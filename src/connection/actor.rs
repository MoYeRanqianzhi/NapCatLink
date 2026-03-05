//! # 连接 Actor 模块
//!
//! 实现连接管理的核心 Actor 模式，包括：
//! - WebSocket 连接的建立和关闭
//! - 消息的读取（接收事件）和写入（发送 API 请求）
//! - 将接收到的原始消息分发到事件总线
//! - 协调心跳检测和重连逻辑
//!
//! # 架构设计
//!
//! ConnectionActor 运行在独立的 tokio task 中，通过 channel 与外部通信：
//! - 外部通过 `ConnectionHandle` 发送命令（Connect / Disconnect / Send）
//! - Actor 通过 `ConnectionNotification` channel 向外部发送通知（状态变化 / 收到消息 / 连接丢失）

// 引入 tokio 的 mpsc 和 oneshot channel
use tokio::sync::{mpsc, oneshot};
// 引入 tokio-tungstenite 的 WebSocket 消息类型
use tokio_tungstenite::tungstenite::Message as WsMessage;
// 引入 futures-util 的 Stream/Sink 扩展 trait 和拆分类型
use futures_util::{SinkExt, StreamExt, stream::SplitSink, stream::SplitStream};
// 引入 tracing 日志宏
use tracing::{debug, error, info, warn};
// 引入 tokio 的时间工具
use tokio::time::Duration;

// 引入本库的配置类型
use crate::config::NapLinkConfig;
// 引入本库的错误类型
use crate::error::{NapLinkError, Result};
// 引入连接状态枚举
use super::state::ConnectionState;
// 引入心跳服务及其消息类型
use super::heartbeat::{HeartbeatNotification, HeartbeatService};
// 引入重连服务
use super::reconnect::ReconnectService;

/// WebSocket 流类型别名 — tokio-tungstenite 连接后的完整流类型
///
/// `MaybeTlsStream` 表示连接可能是普通 TCP 或 TLS 加密连接。
type WsStream = tokio_tungstenite::WebSocketStream<
    tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
>;

/// 发送给 ConnectionActor 的命令枚举
///
/// 外部（NapLinkClient）通过 ConnectionHandle 向 Actor 发送这些命令。
#[derive(Debug)]
pub enum ConnectionCommand {
    /// 建立 WebSocket 连接
    ///
    /// `reply` 用于将连接结果通知回调用方（oneshot 单次响应）。
    Connect {
        /// 连接结果回调 — Ok(()) 连接成功，Err 连接失败
        reply: oneshot::Sender<Result<()>>,
    },
    /// 断开 WebSocket 连接
    Disconnect,
    /// 发送 WebSocket 文本消息
    Send {
        /// 要发送的 JSON 字符串
        payload: String,
    },
}

/// ConnectionActor 发给外部的通知枚举
///
/// Actor 通过 notification channel 向外部发送这些通知，
/// 外部（NapLinkClient）据此更新状态、处理消息、触发事件等。
#[derive(Debug)]
pub enum ConnectionNotification {
    /// 连接状态发生变化
    StateChanged(ConnectionState),
    /// 收到 WebSocket 文本消息（JSON 字符串）
    Message(String),
    /// 连接彻底丢失 — 已达到最大重连次数，无法恢复
    ConnectionLost {
        /// 已尝试的重连次数
        attempts: u32,
    },
    /// 连接恢复 — 重连成功后发送此通知
    ConnectionRestored {
        /// 恢复时的 UNIX 时间戳（毫秒）
        timestamp: u64,
    },
}

/// 构建 WebSocket 连接 URL
///
/// 根据配置中的基础 URL 和可选的 Token，构建最终的 WebSocket 连接地址。
/// Token 以 URL 查询参数 `access_token` 的形式附加。
///
/// # 参数
///
/// - `config`: NapLink 完整配置
///
/// # 返回值
///
/// 返回构建好的 WebSocket URL 字符串。
///
/// # 示例
///
/// - 无 Token: `ws://127.0.0.1:3001` -> `ws://127.0.0.1:3001`
/// - 有 Token: `ws://127.0.0.1:3001` -> `ws://127.0.0.1:3001?access_token=xxx`
/// - URL 已有参数: `ws://127.0.0.1:3001?foo=bar` -> `ws://127.0.0.1:3001?foo=bar&access_token=xxx`
pub fn build_websocket_url(config: &NapLinkConfig) -> String {
    // 获取配置中的基础 URL
    let url = &config.connection.url;
    // 根据 Token 是否存在决定 URL 拼接方式
    match &config.connection.token {
        // Token 存在且非空 — 附加 access_token 查询参数
        Some(token) if !token.is_empty() => {
            // 检查 URL 是否已包含查询参数（即包含 '?'）
            if url.contains('?') {
                // 已有查询参数，使用 '&' 连接
                format!("{}&access_token={}", url, token)
            } else {
                // 无查询参数，使用 '?' 开始
                format!("{}?access_token={}", url, token)
            }
        }
        // Token 不存在或为空 — 直接返回原始 URL
        _ => url.clone(),
    }
}

/// ConnectionActor — 运行在独立 tokio task 中的连接管理器
///
/// 核心职责：
/// 1. 管理 WebSocket 连接的建立和关闭
/// 2. 转发 WebSocket 消息到外部（通过 notification channel）
/// 3. 协调心跳检测机制
/// 4. 在连接断开时执行自动重连策略
///
/// # Actor 模式
///
/// Actor 通过 channel 与外部通信，不共享可变状态：
/// - 输入：cmd_rx 接收外部命令
/// - 输出：notification_tx 发送通知
struct ConnectionActor {
    /// NapLink 完整配置
    config: NapLinkConfig,
    /// 当前连接状态
    state: ConnectionState,
    /// WebSocket 写入端（拆分后的 Sink） — None 表示未连接
    ws_writer: Option<SplitSink<WsStream, WsMessage>>,
    /// 心跳服务实例 — None 表示心跳未启动
    heartbeat: Option<HeartbeatService>,
    /// 重连服务 — 管理重连策略和尝试次数
    reconnect_svc: ReconnectService,
    /// 标记是否正在从重连中恢复（用于发送 ConnectionRestored 通知）
    was_reconnecting: bool,
    /// 通知外部的 channel 发送端
    notification_tx: mpsc::Sender<ConnectionNotification>,
    /// 命令接收端 — 接收来自 ConnectionHandle 的命令
    cmd_rx: mpsc::Receiver<ConnectionCommand>,
    /// 心跳通知接收端 — 接收来自 HeartbeatService 的通知
    heartbeat_notification_rx: Option<mpsc::Receiver<HeartbeatNotification>>,
    /// 上次成功连接的时间戳（用于检测"连接后立即断开"的模式）
    last_connected_at: Option<std::time::Instant>,
    /// 连续快速断开次数（连接后 5 秒内就断开）
    rapid_close_count: u32,
    /// 最近收到的 Close 帧的 close code（用于判断是否应该重连）
    last_close_code: Option<u16>,
}

impl ConnectionActor {
    /// 生成（spawn）一个 ConnectionActor 并返回其控制句柄
    ///
    /// 此方法创建命令 channel，构造 Actor 实例，
    /// 然后在独立的 tokio task 中运行 Actor 的事件循环。
    ///
    /// # 参数
    ///
    /// - `config`: NapLink 完整配置
    /// - `notification_tx`: 通知 channel 的发送端（由外部持有接收端）
    ///
    /// # 返回值
    ///
    /// 返回 `ConnectionHandle`，外部通过它向 Actor 发送命令。
    pub fn spawn(
        config: NapLinkConfig,
        notification_tx: mpsc::Sender<ConnectionNotification>,
    ) -> ConnectionHandle {
        // 创建命令 channel，缓冲区 32 条消息
        let (cmd_tx, cmd_rx) = mpsc::channel::<ConnectionCommand>(32);

        // 从配置中克隆重连配置，创建重连服务
        let reconnect_svc = ReconnectService::new(config.reconnect.clone());

        // 构造 Actor 实例
        let mut actor = Self {
            // 保存配置
            config,
            // 初始状态为断开连接
            state: ConnectionState::Disconnected,
            // 初始无 WebSocket 写入端
            ws_writer: None,
            // 初始无心跳服务
            heartbeat: None,
            // 初始化重连服务
            reconnect_svc,
            // 初始不是重连状态
            was_reconnecting: false,
            // 保存通知发送端
            notification_tx,
            // 保存命令接收端
            cmd_rx,
            // 初始无心跳通知接收端
            heartbeat_notification_rx: None,
            // 初始无连接时间戳
            last_connected_at: None,
            // 初始快速断开计数为 0
            rapid_close_count: 0,
            // 初始无 close code
            last_close_code: None,
        };

        // 在独立的 tokio task 中运行 Actor 事件循环
        tokio::spawn(async move {
            actor.run().await;
        });

        // 返回连接控制句柄
        ConnectionHandle { cmd_tx }
    }

    /// Actor 主事件循环
    ///
    /// 使用 `tokio::select!` 同时监听多个事件源：
    /// 1. 外部命令（Connect / Disconnect / Send）
    /// 2. WebSocket 读取消息（当已连接时）
    /// 3. 心跳通知（当心跳服务运行时）
    ///
    /// 循环持续运行直到命令 channel 关闭。
    async fn run(&mut self) {
        // 输出信息日志，标记 Actor 启动
        info!("ConnectionActor: 事件循环已启动");

        // WebSocket 读取端（拆分后的 Stream） — 需要在循环外声明以保持生命周期
        let mut ws_reader: Option<SplitStream<WsStream>> = None;

        // Actor 主事件循环
        loop {
            // 使用 select! 同时等待多个异步事件
            tokio::select! {
                // 监听外部命令
                cmd = self.cmd_rx.recv() => {
                    match cmd {
                        // 收到连接命令
                        Some(ConnectionCommand::Connect { reply }) => {
                            // 执行连接逻辑，获取可能的 WebSocket 读取端
                            let reader = self.handle_connect(reply).await;
                            // 如果连接成功，保存读取端
                            ws_reader = reader;
                        }
                        // 收到断开命令
                        Some(ConnectionCommand::Disconnect) => {
                            // 执行断开逻辑
                            self.handle_disconnect();
                            // 发送状态变化通知到外部监听者
                            let _ = self.notification_tx.send(
                                ConnectionNotification::StateChanged(ConnectionState::Disconnected)
                            ).await;
                            // 清空 WebSocket 读取端
                            ws_reader = None;
                        }
                        // 收到发送消息命令
                        Some(ConnectionCommand::Send { payload }) => {
                            // 将消息写入 WebSocket
                            self.handle_send(payload).await;
                        }
                        // 命令 channel 关闭 — 外部已释放所有 ConnectionHandle
                        None => {
                            info!("ConnectionActor: 命令 channel 已关闭，退出事件循环");
                            // 执行清理并退出
                            self.handle_disconnect();
                            // 发送状态变化通知到外部监听者
                            let _ = self.notification_tx.send(
                                ConnectionNotification::StateChanged(ConnectionState::Disconnected)
                            ).await;
                            break;
                        }
                    }
                }

                // 监听 WebSocket 读取消息（仅在有读取端时）
                ws_msg = async {
                    match ws_reader.as_mut() {
                        // 有读取端：等待下一条消息
                        Some(reader) => reader.next().await,
                        // 无读取端：永远挂起（不会触发此分支）
                        None => std::future::pending().await,
                    }
                } => {
                    match ws_msg {
                        // 收到 WebSocket 消息
                        Some(Ok(msg)) => {
                            // 处理收到的消息（分发文本消息、记录 pong 等）
                            self.handle_ws_message(msg).await;
                        }
                        // WebSocket 读取错误
                        Some(Err(e)) => {
                            warn!("WebSocket 读取错误: {}", e);
                            // 触发重连流程（handle_ws_close 内部会清理旧连接资源）
                            ws_reader = self.handle_ws_close().await;
                        }
                        // WebSocket 流结束（连接关闭）
                        None => {
                            info!("WebSocket 连接已关闭");
                            // 触发重连流程（handle_ws_close 内部会清理旧连接资源）
                            ws_reader = self.handle_ws_close().await;
                        }
                    }
                }

                // 监听心跳通知（仅在有心跳通知接收端时）
                hb_notif = async {
                    match self.heartbeat_notification_rx.as_mut() {
                        // 有心跳通知接收端：等待下一条通知
                        Some(rx) => rx.recv().await,
                        // 无接收端：永远挂起
                        None => std::future::pending().await,
                    }
                } => {
                    match hb_notif {
                        // 需要发送 ping
                        Some(HeartbeatNotification::SendPing(payload)) => {
                            debug!("心跳: 发送 ping");
                            // 通过 WebSocket 发送 ping payload
                            self.handle_send(payload).await;
                        }
                        // 心跳超时
                        Some(HeartbeatNotification::Timeout) => {
                            warn!("心跳超时: 连接可能已断开");
                            // 处理心跳超时（断开并尝试重连，内部会清理旧连接资源）
                            ws_reader = self.handle_heartbeat_timeout().await;
                        }
                        // 心跳通知 channel 关闭
                        None => {
                            debug!("心跳通知 channel 已关闭");
                            // 清空心跳通知接收端
                            self.heartbeat_notification_rx = None;
                        }
                    }
                }
            }
        }

        // 输出信息日志，标记 Actor 退出
        info!("ConnectionActor: 事件循环已退出");
    }

    /// 处理连接命令 — 建立 WebSocket 连接
    ///
    /// 步骤：
    /// 1. 构建带 Token 的 WebSocket URL
    /// 2. 在超时时间内尝试建立连接
    /// 3. 拆分 WebSocket 流为读写两端
    /// 4. 启动心跳服务
    /// 5. 通知外部状态变为 Connected
    ///
    /// # 参数
    ///
    /// - `reply`: 连接结果回调 channel
    ///
    /// # 返回值
    ///
    /// 连接成功返回 `Some(SplitStream)`，失败返回 `None`。
    async fn handle_connect(
        &mut self,
        reply: oneshot::Sender<Result<()>>,
    ) -> Option<SplitStream<WsStream>> {
        // 防止重复连接 — 如果已在连接中或已连接，直接返回错误
        if self.state == ConnectionState::Connecting || self.state == ConnectionState::Connected {
            let _ = reply.send(Err(NapLinkError::Connection(
                "已在连接中或已连接，请勿重复调用 connect()".to_string()
            )));
            return None;
        }

        // 更新状态为"连接中"并通知外部
        self.set_state(ConnectionState::Connecting).await;

        // 构建 WebSocket 连接 URL（带 Token）
        let url = build_websocket_url(&self.config);
        info!("正在连接到 WebSocket: {}", url);

        // 获取连接超时时间
        let timeout_duration = Duration::from_millis(self.config.connection.timeout_ms);

        // 在超时时间内尝试建立 WebSocket 连接
        let connect_result = tokio::time::timeout(
            timeout_duration,
            tokio_tungstenite::connect_async(&url),
        )
        .await;

        // 处理连接结果
        match connect_result {
            // 连接超时
            Err(_elapsed) => {
                let err_msg = format!(
                    "连接超时: 在 {}ms 内未能建立连接",
                    self.config.connection.timeout_ms
                );
                error!("{}", err_msg);
                // 更新状态为断开
                self.set_state(ConnectionState::Disconnected).await;
                // 通知调用方连接失败
                let _ = reply.send(Err(NapLinkError::Connection(err_msg)));
                // 返回 None 表示无读取端
                None
            }
            // 连接失败
            Ok(Err(e)) => {
                let err_msg = format!("WebSocket 连接失败: {}", e);
                error!("{}", err_msg);
                // 更新状态为断开
                self.set_state(ConnectionState::Disconnected).await;
                // 通知调用方连接失败
                let _ = reply.send(Err(NapLinkError::Connection(err_msg)));
                // 返回 None
                None
            }
            // 连接成功
            Ok(Ok((ws_stream, _response))) => {
                info!("WebSocket 连接成功");

                // 将 WebSocket 流拆分为读取端和写入端
                let (writer, reader) = ws_stream.split();
                // 保存写入端到 Actor
                self.ws_writer = Some(writer);

                // 启动心跳服务
                self.start_heartbeat();

                // 记录连接成功时间，用于快速断开检测
                self.last_connected_at = Some(std::time::Instant::now());
                // 首次连接重置快速断开计数
                self.rapid_close_count = 0;
                // 清除上次的 close code
                self.last_close_code = None;

                // 更新状态为已连接
                self.set_state(ConnectionState::Connected).await;

                // 如果是从重连中恢复，发送 ConnectionRestored 通知
                if self.was_reconnecting {
                    // 获取当前时间戳（毫秒）
                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    // 发送连接恢复通知
                    let _ = self
                        .notification_tx
                        .send(ConnectionNotification::ConnectionRestored { timestamp })
                        .await;
                    // 重置重连标记
                    self.was_reconnecting = false;
                }

                // 重连成功后重置重连计数器
                self.reconnect_svc.reset();

                // 通知调用方连接成功
                let _ = reply.send(Ok(()));

                // 返回 WebSocket 读取端
                Some(reader)
            }
        }
    }

    /// 处理断开连接命令
    ///
    /// 清理所有连接相关资源：
    /// 1. 停止心跳服务
    /// 2. 关闭 WebSocket 写入端
    /// 3. 清空心跳通知接收端
    fn handle_disconnect(&mut self) {
        info!("正在断开 WebSocket 连接");

        // 停止心跳服务（如果存在）
        if let Some(hb) = self.heartbeat.take() {
            hb.stop();
        }

        // 释放 WebSocket 写入端（Drop 时会自动关闭）
        self.ws_writer = None;

        // 清空心跳通知接收端
        self.heartbeat_notification_rx = None;

        // 注意：这里不使用 set_state 异步方法，因为此函数是同步的
        // 状态更新将在调用处处理
        self.state = ConnectionState::Disconnected;

        info!("WebSocket 连接已断开");
    }

    /// 处理发送消息命令 — 将文本消息写入 WebSocket
    ///
    /// # 参数
    ///
    /// - `payload`: 要发送的 JSON 字符串
    async fn handle_send(&mut self, payload: String) {
        // 检查 WebSocket 写入端是否存在
        if let Some(writer) = self.ws_writer.as_mut() {
            // 将字符串包装为 WebSocket Text 消息并发送
            if let Err(e) = writer.send(WsMessage::Text(payload.into())).await {
                error!("WebSocket 发送失败: {}", e);
            }
        } else {
            // 无写入端，说明未连接
            warn!("尝试发送消息但 WebSocket 未连接");
        }
    }

    /// 处理收到的 WebSocket 消息
    ///
    /// 根据消息类型分别处理：
    /// - Text: 解析为 JSON 文本，通知外部
    /// - Pong: 记录心跳 pong 响应
    /// - Close: 触发重连流程
    /// - 其他类型: 忽略（Ping 由 tungstenite 自动处理）
    ///
    /// # 参数
    ///
    /// - `msg`: 收到的 WebSocket 消息
    async fn handle_ws_message(&mut self, msg: WsMessage) {
        match msg {
            // 收到文本消息 — 通常是 OneBot 事件或 API 响应的 JSON
            WsMessage::Text(text) => {
                debug!("收到 WebSocket 文本消息: {} 字节", text.len());
                // 收到任何消息都视为心跳 pong（说明连接存活）
                if let Some(hb) = &self.heartbeat {
                    hb.record_pong().await;
                }
                // 通知外部收到消息
                let _ = self
                    .notification_tx
                    .send(ConnectionNotification::Message(text.to_string()))
                    .await;
            }
            // 收到 Pong 帧 — WebSocket 协议层的心跳响应
            WsMessage::Pong(_) => {
                debug!("收到 WebSocket Pong 帧");
                // 记录心跳 pong 响应
                if let Some(hb) = &self.heartbeat {
                    hb.record_pong().await;
                }
            }
            // 收到 Close 帧 — 服务端主动关闭连接
            WsMessage::Close(frame) => {
                info!("收到 WebSocket Close 帧: {:?}", frame);
                // 保存 close code，供 handle_ws_close 判断是否需要重连
                self.last_close_code = frame.as_ref().map(|f| f.code.into());
            }
            // 收到 Ping 帧 — tokio-tungstenite 会自动回复 Pong
            WsMessage::Ping(_) => {
                debug!("收到 WebSocket Ping 帧（自动回复 Pong）");
            }
            // 收到二进制消息 — OneBot 11 协议通常不使用二进制，忽略
            WsMessage::Binary(_) => {
                debug!("收到 WebSocket 二进制消息（忽略）");
            }
            // 收到 Frame 类型 — 内部消息，忽略
            WsMessage::Frame(_) => {
                debug!("收到 WebSocket 原始帧（忽略）");
            }
        }
    }

    /// 处理 WebSocket 连接关闭 — 执行重连逻辑
    ///
    /// 当 WebSocket 连接断开时（正常关闭或异常断开），
    /// 根据重连策略决定是否重连以及重连延迟。
    ///
    /// # 返回值
    ///
    /// 重连成功返回 `Some(SplitStream)`，放弃重连返回 `None`。
    async fn handle_ws_close(&mut self) -> Option<SplitStream<WsStream>> {
        // 停止当前心跳服务
        if let Some(hb) = self.heartbeat.take() {
            hb.stop();
        }
        // 释放写入端
        self.ws_writer = None;
        // 清空心跳通知接收端
        self.heartbeat_notification_rx = None;

        // === Close code 检查 ===
        // Close code 1000 表示正常关闭（服务端主动断开），不应重连
        if self.last_close_code == Some(1000) {
            info!("服务端正常关闭连接 (code 1000)，不进行重连");
            self.set_state(ConnectionState::Disconnected).await;
            return None;
        }

        // === 快速断开检测 ===
        // 如果连接后极短时间内就断开（< 5秒），说明服务端在拒绝连接
        // （常见原因：token 验证失败、服务端配置问题）
        // 连续发生 3 次后停止重连，避免无限循环
        const RAPID_CLOSE_THRESHOLD: Duration = Duration::from_secs(5);
        const MAX_RAPID_CLOSES: u32 = 3;

        if let Some(connected_at) = self.last_connected_at {
            let elapsed = connected_at.elapsed();
            if elapsed < RAPID_CLOSE_THRESHOLD {
                self.rapid_close_count += 1;
                warn!(
                    "连接后 {}ms 内即断开（第 {}/{} 次快速断开）",
                    elapsed.as_millis(),
                    self.rapid_close_count,
                    MAX_RAPID_CLOSES
                );

                if self.rapid_close_count >= MAX_RAPID_CLOSES {
                    error!(
                        "连续 {} 次连接后立即被服务端关闭，停止重连。\
                        请检查：1) access_token 是否正确  2) 服务端配置是否正确  3) 服务端日志",
                        self.rapid_close_count
                    );
                    self.set_state(ConnectionState::Disconnected).await;
                    return None;
                }
            } else {
                // 连接持续了足够久才断开，重置快速断开计数
                self.rapid_close_count = 0;
            }
        }

        // 检查是否有剩余的重连机会
        if let Some(delay) = self.reconnect_svc.next_delay() {
            // 标记正在重连
            self.was_reconnecting = true;
            // 更新状态为重连中
            self.set_state(ConnectionState::Reconnecting).await;

            info!(
                "将在 {}ms 后尝试第 {} 次重连（最大 {} 次）",
                delay.as_millis(),
                self.reconnect_svc.current_attempt(),
                self.reconnect_svc.max_attempts()
            );

            // 等待退避延迟
            tokio::time::sleep(delay).await;

            // 尝试重新连接
            return self.attempt_reconnect().await;
        }

        // 没有重连机会 — 通知外部连接彻底丢失
        let attempts = self.reconnect_svc.current_attempt();
        error!("连接丢失: 已达到最大重连次数 {}", attempts);
        // 更新状态为断开
        self.set_state(ConnectionState::Disconnected).await;
        // 发送连接丢失通知
        let _ = self
            .notification_tx
            .send(ConnectionNotification::ConnectionLost { attempts })
            .await;

        // 返回 None 表示无法恢复
        None
    }

    /// 处理心跳超时 — 主动断开并触发重连
    ///
    /// 当心跳服务检测到连续多次未收到 pong 响应时，
    /// 判定连接已实质断开，主动触发重连流程。
    ///
    /// # 返回值
    ///
    /// 重连成功返回 `Some(SplitStream)`，放弃重连返回 `None`。
    async fn handle_heartbeat_timeout(&mut self) -> Option<SplitStream<WsStream>> {
        warn!("心跳超时: 主动断开连接并尝试重连");
        // 复用 WebSocket 关闭处理逻辑
        self.handle_ws_close().await
    }

    /// 尝试重新连接 — 建立新的 WebSocket 连接
    ///
    /// 使用循环（非递归）方式持续尝试重连，直到成功或达到最大重连次数。
    /// 每次失败后根据指数退避策略等待一段时间再重试。
    ///
    /// # 返回值
    ///
    /// 连接成功返回 `Some(SplitStream)`，放弃重连返回 `None`。
    async fn attempt_reconnect(&mut self) -> Option<SplitStream<WsStream>> {
        // 使用循环替代递归，避免 async 递归导致的 future 大小无限增长
        loop {
            // 构建 WebSocket 连接 URL
            let url = build_websocket_url(&self.config);
            info!("重连: 正在连接到 {}", url);

            // 获取连接超时时间
            let timeout_duration = Duration::from_millis(self.config.connection.timeout_ms);

            // 在超时时间内尝试建立连接
            let connect_result = tokio::time::timeout(
                timeout_duration,
                tokio_tungstenite::connect_async(&url),
            )
            .await;

            match connect_result {
                // 连接超时
                Err(_elapsed) => {
                    warn!("重连超时");
                }
                // 连接失败
                Ok(Err(e)) => {
                    warn!("重连失败: {}", e);
                }
                // 连接成功
                Ok(Ok((ws_stream, _response))) => {
                    info!("重连成功");

                    // 拆分 WebSocket 流为读写两端
                    let (writer, reader) = ws_stream.split();
                    // 保存写入端到 Actor
                    self.ws_writer = Some(writer);

                    // 启动心跳服务
                    self.start_heartbeat();

                    // 记录重连成功时间，用于快速断开检测
                    self.last_connected_at = Some(std::time::Instant::now());
                    // 清除上次的 close code
                    self.last_close_code = None;

                    // 更新状态为已连接
                    self.set_state(ConnectionState::Connected).await;

                    // 仅当之前处于重连状态时才发送 ConnectionRestored 通知
                    // 避免在非重连场景（如首次连接）中误发恢复通知
                    if self.was_reconnecting {
                        // 获取当前 UNIX 时间戳（毫秒）
                        let timestamp = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64;
                        // 发送连接恢复通知
                        let _ = self
                            .notification_tx
                            .send(ConnectionNotification::ConnectionRestored { timestamp })
                            .await;
                        // 重置重连标记
                        self.was_reconnecting = false;
                    }

                    // 重置重连计数器
                    self.reconnect_svc.reset();

                    // 返回 WebSocket 读取端
                    return Some(reader);
                }
            }

            // 连接失败 — 检查是否还有重连机会
            if let Some(delay) = self.reconnect_svc.next_delay() {
                info!(
                    "将在 {}ms 后尝试第 {} 次重连",
                    delay.as_millis(),
                    self.reconnect_svc.current_attempt()
                );
                // 等待退避延迟后继续循环重试
                tokio::time::sleep(delay).await;
            } else {
                // 没有更多重连机会 — 通知外部连接彻底丢失
                let attempts = self.reconnect_svc.current_attempt();
                error!("连接丢失: 已达到最大重连次数 {}", attempts);
                // 更新状态为断开
                self.set_state(ConnectionState::Disconnected).await;
                // 发送连接丢失通知
                let _ = self
                    .notification_tx
                    .send(ConnectionNotification::ConnectionLost { attempts })
                    .await;
                // 返回 None 表示放弃重连
                return None;
            }
        }
    }

    /// 启动心跳服务
    ///
    /// 根据配置创建心跳 ping payload（OneBot API 请求 JSON），
    /// 然后启动 HeartbeatService。
    fn start_heartbeat(&mut self) {
        // 如果心跳间隔为 0，表示心跳已禁用，不启动服务
        if self.config.connection.ping_interval_ms == 0 {
            debug!("心跳已禁用 (ping_interval_ms: 0)");
            return;
        }

        // 构建心跳 ping 的 JSON payload（OneBot API 请求格式）
        // 必须包含 echo 字段且以 "heartbeat_" 开头，
        // 这样 Dispatcher 才能识别出心跳响应并忽略它
        //
        // 使用动态时间戳作为 echo 后缀（与 TS 版本 `heartbeat_${Date.now()}` 一致），
        // 确保每次心跳的 echo 值唯一，避免与 API 响应匹配器冲突
        let heartbeat_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let ping_payload = serde_json::json!({
            // 调用的 API 方法名（默认 get_status）
            "action": self.config.connection.heartbeat_action.action,
            // API 参数（默认空对象）
            "params": self.config.connection.heartbeat_action.params,
            // 心跳专用 echo 标识 — 格式: "heartbeat_{时间戳毫秒}"
            // Dispatcher 会跳过以 "heartbeat_" 开头的响应
            "echo": format!("heartbeat_{}", heartbeat_timestamp),
        })
        .to_string();

        // 创建心跳通知 channel
        let (hb_notification_tx, hb_notification_rx) = mpsc::channel::<HeartbeatNotification>(16);

        // 启动心跳服务
        let hb_service = HeartbeatService::start(
            // 心跳间隔（毫秒）
            self.config.connection.ping_interval_ms,
            // ping payload JSON 字符串
            ping_payload,
            // 心跳通知发送端
            hb_notification_tx,
        );

        // 保存心跳服务和通知接收端
        self.heartbeat = Some(hb_service);
        self.heartbeat_notification_rx = Some(hb_notification_rx);
    }

    /// 更新连接状态并通知外部
    ///
    /// # 参数
    ///
    /// - `new_state`: 新的连接状态
    async fn set_state(&mut self, new_state: ConnectionState) {
        // 仅在状态真正发生变化时才通知
        if self.state != new_state {
            // 记录状态变化日志
            info!("连接状态: {} -> {}", self.state, new_state);
            // 更新内部状态
            self.state = new_state;
            // 通知外部状态变化
            let _ = self
                .notification_tx
                .send(ConnectionNotification::StateChanged(new_state))
                .await;
        }
    }
}

/// ConnectionHandle — 连接控制句柄
///
/// 外部（NapLinkClient）通过此句柄向 ConnectionActor 发送命令。
/// 句柄可以被 Clone，支持多个发送者同时持有。
///
/// # 线程安全
///
/// `mpsc::Sender` 是线程安全的，因此 ConnectionHandle 可以跨线程使用。
#[derive(Clone)]
pub struct ConnectionHandle {
    /// 命令发送端 — 向 ConnectionActor 发送 ConnectionCommand
    cmd_tx: mpsc::Sender<ConnectionCommand>,
}

impl ConnectionHandle {
    /// 创建 Actor 并获取句柄
    ///
    /// 这是创建连接管理器的入口方法。
    /// 会 spawn 一个 ConnectionActor task 并返回控制句柄。
    ///
    /// # 参数
    ///
    /// - `config`: NapLink 完整配置
    /// - `notification_tx`: 通知 channel 的发送端
    ///
    /// # 返回值
    ///
    /// 返回 ConnectionHandle 实例。
    pub fn new(
        config: NapLinkConfig,
        notification_tx: mpsc::Sender<ConnectionNotification>,
    ) -> Self {
        // 委托给 ConnectionActor::spawn 创建 Actor 并返回句柄
        ConnectionActor::spawn(config, notification_tx)
    }

    /// 建立 WebSocket 连接
    ///
    /// 向 Actor 发送 Connect 命令，并等待连接结果。
    ///
    /// # 返回值
    ///
    /// - `Ok(())`: 连接成功
    /// - `Err(NapLinkError)`: 连接失败（超时、拒绝等）
    pub async fn connect(&self) -> Result<()> {
        // 创建 oneshot channel 用于接收连接结果
        let (reply_tx, reply_rx) = oneshot::channel();

        // 向 Actor 发送 Connect 命令
        self.cmd_tx
            .send(ConnectionCommand::Connect { reply: reply_tx })
            .await
            .map_err(|_| NapLinkError::Connection("Actor 已停止".to_string()))?;

        // 等待 Actor 回复连接结果
        reply_rx
            .await
            .map_err(|_| NapLinkError::Connection("Actor 未响应".to_string()))?
    }

    /// 断开 WebSocket 连接
    ///
    /// 向 Actor 发送 Disconnect 命令。
    /// 此方法不等待断开完成，是非阻塞的。
    pub fn disconnect(&self) {
        // 使用 try_send 非阻塞发送断开命令（忽略发送失败，可能 Actor 已停止）
        let _ = self.cmd_tx.try_send(ConnectionCommand::Disconnect);
    }

    /// 发送 WebSocket 文本消息
    ///
    /// 向 Actor 发送 Send 命令，将 JSON 字符串通过 WebSocket 发出。
    ///
    /// # 参数
    ///
    /// - `payload`: 要发送的 JSON 字符串
    ///
    /// # 返回值
    ///
    /// - `Ok(())`: 消息已成功提交给 Actor（不保证已实际发送到网络）
    /// - `Err(NapLinkError)`: Actor 已停止，无法接受消息
    pub fn send(&self, payload: String) -> Result<()> {
        // 使用 try_send 非阻塞发送消息命令
        self.cmd_tx
            .try_send(ConnectionCommand::Send { payload })
            .map_err(|_| NapLinkError::Connection("Actor 已停止或消息队列已满".to_string()))
    }
}
