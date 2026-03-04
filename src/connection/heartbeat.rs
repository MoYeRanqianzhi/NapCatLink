//! # 心跳检测模块
//!
//! 实现 WebSocket 连接的心跳保活机制，包括：
//! - 定时发送心跳 ping 消息（通过 OneBot API 请求作为 ping 载荷）
//! - 追踪 pong 响应是否按时到达
//! - 连续多次未收到 pong 时判定连接超时
//! - 通过 channel 与外部 Actor 通信

// 引入 tokio 的 mpsc channel，用于异步消息传递
use tokio::sync::mpsc;
// 引入 tokio 的时间工具，Duration 表示时间长度，interval 创建定时器
use tokio::time::{Duration, interval};
// 引入 tracing 日志宏，用于输出结构化日志
use tracing::{debug, warn};

/// 连续未收到 pong 响应的最大容忍次数
///
/// 超过此次数后，心跳服务会发送 Timeout 通知，
/// 告知外部 Actor 连接可能已经断开。
const MAX_MISSED_PONGS: u32 = 3;

/// 心跳命令枚举 — 从外部（ConnectionActor）发给心跳服务的控制指令
///
/// 外部通过持有的 `mpsc::Sender<HeartbeatCommand>` 向心跳服务发送命令。
#[derive(Debug)]
pub enum HeartbeatCommand {
    /// 记录收到了 pong 响应 — 将未响应计数器重置为 0
    RecordPong,
    /// 停止心跳服务 — 优雅地退出心跳循环
    Stop,
}

/// 心跳通知枚举 — 心跳服务发给外部（ConnectionActor）的通知
///
/// 心跳服务通过持有的 `mpsc::Sender<HeartbeatNotification>` 向外部发送通知。
#[derive(Debug)]
pub enum HeartbeatNotification {
    /// 需要发送 ping — 携带要通过 WebSocket 发送的 JSON payload 字符串
    SendPing(String),
    /// 心跳超时 — 连续 MAX_MISSED_PONGS 次未收到 pong 响应
    Timeout,
}

/// 心跳服务 — 作为独立 tokio task 运行的心跳检测器
///
/// 职责：
/// 1. 定时通知外部发送 ping 消息
/// 2. 追踪 pong 响应的接收情况
/// 3. 超时时通知外部连接可能已断开
///
/// 生命周期由外部 ConnectionActor 管理：
/// - 连接建立时通过 `start()` 创建
/// - 连接断开时通过 `stop()` 销毁
pub struct HeartbeatService {
    /// 命令发送端 — 外部持有此端，用于向心跳 task 发送 RecordPong / Stop 命令
    cmd_tx: mpsc::Sender<HeartbeatCommand>,
    /// 心跳 task 的 JoinHandle — 用于停止时 abort 心跳 task
    task: tokio::task::JoinHandle<()>,
}

impl HeartbeatService {
    /// 启动心跳服务
    ///
    /// 创建一个独立的 tokio task 运行心跳循环。
    ///
    /// # 参数
    ///
    /// - `interval_ms`: 心跳发送间隔（毫秒），每隔此时长发送一次 ping
    /// - `ping_payload`: 要通过 WebSocket 发送的 ping JSON 字符串（通常是 OneBot API 请求）
    /// - `notification_tx`: 通知发送端，心跳服务通过此 channel 向外部发送 SendPing / Timeout 通知
    ///
    /// # 返回值
    ///
    /// 返回 HeartbeatService 实例，外部通过它控制心跳服务的生命周期。
    pub fn start(
        interval_ms: u64,
        ping_payload: String,
        notification_tx: mpsc::Sender<HeartbeatNotification>,
    ) -> Self {
        // 创建命令 channel，缓冲区大小为 8（足够缓冲 RecordPong 和 Stop 命令）
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<HeartbeatCommand>(8);

        // spawn 独立的心跳 tokio task
        let task = tokio::spawn(async move {
            // 创建定时器，按照指定间隔周期性触发
            let mut ticker = interval(Duration::from_millis(interval_ms));
            // 跳过第一次立即触发的 tick（首次 tick 会立即返回）
            ticker.tick().await;

            // 未收到 pong 的连续次数计数器，初始为 0
            let mut missed_pongs: u32 = 0;

            // 心跳主循环
            loop {
                // 使用 select! 同时等待定时器触发和外部命令
                tokio::select! {
                    // 定时器触发 — 该发送 ping 了
                    _ = ticker.tick() => {
                        // 累加未响应计数器（每次 ping 都假设不会收到 pong，直到 RecordPong 重置）
                        missed_pongs += 1;
                        // 输出调试日志，记录当前未响应次数
                        debug!(
                            missed_pongs = missed_pongs,
                            max = MAX_MISSED_PONGS,
                            "心跳 ping: 未响应次数 {}/{}",
                            missed_pongs,
                            MAX_MISSED_PONGS
                        );

                        // 检查是否超过最大容忍次数
                        if missed_pongs >= MAX_MISSED_PONGS {
                            // 超时：连续多次未收到 pong，判定连接已断开
                            warn!(
                                "心跳超时: 连续 {} 次未收到 pong 响应",
                                missed_pongs
                            );
                            // 向外部发送 Timeout 通知
                            let _ = notification_tx.send(HeartbeatNotification::Timeout).await;
                            // 超时后退出心跳循环，等待外部决定是否重连
                            break;
                        }

                        // 未超时：通知外部发送 ping 消息
                        let _ = notification_tx
                            .send(HeartbeatNotification::SendPing(ping_payload.clone()))
                            .await;
                    }

                    // 收到外部命令
                    cmd = cmd_rx.recv() => {
                        match cmd {
                            // 记录收到了 pong 响应 — 重置计数器
                            Some(HeartbeatCommand::RecordPong) => {
                                debug!("心跳: 收到 pong 响应，重置未响应计数");
                                // 重置未响应计数器为 0
                                missed_pongs = 0;
                            }
                            // 收到停止命令 — 优雅退出心跳循环
                            Some(HeartbeatCommand::Stop) => {
                                debug!("心跳服务: 收到停止命令，退出心跳循环");
                                break;
                            }
                            // channel 关闭 — 外部已释放发送端，退出循环
                            None => {
                                debug!("心跳服务: 命令 channel 已关闭，退出心跳循环");
                                break;
                            }
                        }
                    }
                }
            }

            // 心跳循环结束，输出调试日志
            debug!("心跳服务: task 已退出");
        });

        // 返回心跳服务实例
        Self { cmd_tx, task }
    }

    /// 记录收到 pong 响应
    ///
    /// 当外部 Actor 收到任何来自服务端的响应消息时调用此方法，
    /// 将未响应计数器重置为 0，表明连接仍然存活。
    pub async fn record_pong(&self) {
        // 向心跳 task 发送 RecordPong 命令（忽略发送失败，task 可能已退出）
        let _ = self.cmd_tx.send(HeartbeatCommand::RecordPong).await;
    }

    /// 停止心跳服务
    ///
    /// 先尝试发送 Stop 命令（优雅停止），然后 abort 心跳 task（强制停止）。
    /// 此方法会消费 self，确保停止后不会再使用该服务。
    pub fn stop(self) {
        // 尝试发送停止命令（使用 try_send 避免异步等待）
        let _ = self.cmd_tx.try_send(HeartbeatCommand::Stop);
        // abort 心跳 task，确保 task 立即结束
        self.task.abort();
    }
}
