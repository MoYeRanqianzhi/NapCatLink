//! # 客户端模块
//!
//! 提供面向用户的高层 NapCatLink 客户端 API，整合以下功能：
//! - WebSocket 连接管理（自动重连、心跳）
//! - 事件监听和处理
//! - OneBot 11 标准 API 调用
//! - NapCat 扩展 API 调用
//!
//! 这是用户与 NapCatLink SDK 交互的主要入口点。

// 引入 Arc 智能指针，用于在多个异步任务之间共享配置和事件总线
use std::sync::Arc;

// 引入原子变量，用于无锁方式存储和读取连接状态
use std::sync::atomic::{AtomicU8, Ordering};

// 引入 tokio 的 mpsc channel，用于接收连接通知
use tokio::sync::mpsc;

// 引入 tokio 的 JoinHandle，用于管理后台通知处理任务的生命周期
use tokio::task::JoinHandle;

// 引入 serde_json::Value，用于 call_api 方法的动态 JSON 参数
use serde_json::Value;

// 引入 SDK 配置类型
use crate::config::NapLinkConfig;

// 引入 SDK 错误类型和 Result 类型别名
use crate::error::Result;

// 引入连接句柄和连接通知，用于与 ConnectionActor 通信
use crate::connection::{ConnectionHandle, ConnectionNotification, ConnectionState};

// 引入事件系统组件
use crate::event::{EventBus, EventSubscription, EventRouter};

// 引入 API 组件
use crate::api::{ApiClient, Dispatcher, OneBotApi};

/// NapLink 客户端 — 与 NapCatQQ 交互的主入口
///
/// 使用 Builder 模式创建实例，然后调用 `connect()` 连接到服务器。
/// 内部整合了连接管理、事件分发、API 调用等所有功能模块。
///
/// # 示例
/// ```rust,no_run
/// use napcat_link::NapLink;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = NapLink::builder("ws://127.0.0.1:3001")
///         .token("your_token")
///         .build()?;
///     client.connect().await?;
///     Ok(())
/// }
/// ```
pub struct NapLink {
    /// 完整配置（通过 Arc 共享，因为多个组件需要引用配置）
    config: Arc<NapLinkConfig>,
    /// 连接句柄（用于发送连接/断开/发送命令给 ConnectionActor）
    conn_handle: ConnectionHandle,
    /// 事件总线（通过 Arc 共享，用于事件发布和订阅）
    event_bus: Arc<EventBus>,
    /// API 客户端（用于发送 API 请求和管理请求-响应配对）
    api_client: ApiClient,
    /// 聚合 API（包含所有 API 模块的统一入口）
    onebot_api: OneBotApi,
    /// 消息分发器（区分 API 响应和事件，分别路由到对应处理器）
    _dispatcher: Arc<Dispatcher>,
    /// 连接状态原子变量（0=Disconnected, 1=Connecting, 2=Connected, 3=Reconnecting）
    state: Arc<AtomicU8>,
    /// 后台通知处理任务句柄（保持 task 存活，Drop 时自动取消）
    _notification_task: JoinHandle<()>,
}

impl NapLink {
    /// 创建配置构建器 — 构建 NapLink 客户端的起始入口
    ///
    /// # 参数
    ///
    /// - `url`: WebSocket 服务器地址（例如 "ws://127.0.0.1:3001"）
    ///
    /// # 返回值
    ///
    /// 返回一个新的 `NapLinkBuilder` 实例，支持链式配置
    pub fn builder(url: &str) -> NapLinkBuilder {
        // 创建构建器实例
        NapLinkBuilder::new(url)
    }

    /// 连接到 NapCat 服务器
    ///
    /// 向 ConnectionActor 发送连接命令，等待连接结果。
    /// 连接成功后，心跳检测会自动启动。
    ///
    /// # 返回值
    ///
    /// - `Ok(())`: 连接成功
    /// - `Err(NapLinkError)`: 连接失败（超时、拒绝等）
    pub async fn connect(&self) -> Result<()> {
        // 记录连接开始日志
        tracing::info!("NapLink: 开始连接...");
        // 委托给连接句柄发送连接命令
        self.conn_handle.connect().await
    }

    /// 断开连接
    ///
    /// 向 ConnectionActor 发送断开命令，清理 API 客户端状态。
    /// 断开后所有待处理的 API 请求都会收到错误。
    pub fn disconnect(&self) {
        // 记录断开日志
        tracing::info!("NapLink: 断开连接...");
        // 发送断开命令给 Actor
        self.conn_handle.disconnect();
        // 清理所有待处理的 API 请求
        self.api_client.destroy();
    }

    /// 获取当前连接状态
    ///
    /// 通过原子变量无锁读取当前状态，映射为 `ConnectionState` 枚举值。
    ///
    /// # 返回值
    ///
    /// 返回当前的 `ConnectionState`
    pub fn state(&self) -> ConnectionState {
        // 从原子变量读取状态值并映射为枚举
        match self.state.load(Ordering::Relaxed) {
            // 1 = 连接中
            1 => ConnectionState::Connecting,
            // 2 = 已连接
            2 => ConnectionState::Connected,
            // 3 = 重连中
            3 => ConnectionState::Reconnecting,
            // 其他值（包括 0）= 已断开
            _ => ConnectionState::Disconnected,
        }
    }

    /// 检查是否已连接
    ///
    /// # 返回值
    ///
    /// 当连接状态为 `Connected` 时返回 `true`，否则返回 `false`
    pub fn is_connected(&self) -> bool {
        // 比较当前状态是否为已连接
        self.state() == ConnectionState::Connected
    }

    /// 订阅事件
    ///
    /// 创建一个新的事件订阅，可以接收连接后产生的所有事件。
    /// 订阅只能接收到创建之后发布的事件。
    ///
    /// # 返回值
    ///
    /// 返回一个 `EventSubscription` 实例，用于异步接收事件
    pub fn subscribe(&self) -> EventSubscription {
        // 通过事件总线创建新的订阅
        self.event_bus.subscribe()
    }

    /// 获取 OneBot API 引用
    ///
    /// 返回聚合了所有 API 模块的 `OneBotApi` 引用，
    /// 通过该引用可以访问消息、群组、账号等所有 API。
    ///
    /// # 返回值
    ///
    /// 返回 `&OneBotApi` 引用
    pub fn api(&self) -> &OneBotApi {
        // 返回 onebot_api 的引用
        &self.onebot_api
    }

    /// 调用自定义 API — 直接发送任意 API 请求
    ///
    /// 当内置的 API 方法不满足需求时，可以使用此方法直接调用自定义 action。
    ///
    /// # 参数
    ///
    /// - `method`: API 动作名称（例如 "custom_action"）
    /// - `params`: API 请求参数的 JSON 值
    ///
    /// # 返回值
    ///
    /// - `Ok(Value)`: API 响应中的 `data` 字段
    /// - `Err(NapLinkError)`: 调用失败
    pub async fn call_api(&self, method: &str, params: Value) -> Result<Value> {
        // 委托给 API 客户端发送请求
        self.api_client.call(method, params).await
    }

    /// 获取配置引用
    ///
    /// 返回客户端创建时使用的完整配置。
    ///
    /// # 返回值
    ///
    /// 返回 `&NapLinkConfig` 引用
    pub fn config(&self) -> &NapLinkConfig {
        // 通过 Arc 解引用返回配置引用
        &self.config
    }

    /// 通知处理循环 — 在独立 task 中运行
    ///
    /// 持续接收 ConnectionActor 发出的通知，并根据通知类型执行相应操作：
    /// - `StateChanged`: 更新原子状态变量，通过事件总线发布状态变更事件
    /// - `Message`: 将 WebSocket 消息转发给 Dispatcher 进行分发
    /// - `ConnectionLost`: 发布连接丢失事件
    /// - `ConnectionRestored`: 发布连接恢复事件
    ///
    /// # 参数
    ///
    /// - `rx`: 通知接收端（从 ConnectionActor 接收通知）
    /// - `state`: 原子状态变量（与 NapLink 共享）
    /// - `event_bus`: 事件总线（与 NapLink 共享）
    /// - `dispatcher`: 消息分发器（与 NapLink 共享）
    async fn notification_loop(
        mut rx: mpsc::Receiver<ConnectionNotification>,
        state: Arc<AtomicU8>,
        event_bus: Arc<EventBus>,
        dispatcher: Arc<Dispatcher>,
    ) {
        // 持续接收通知直到 channel 关闭
        while let Some(notification) = rx.recv().await {
            // 根据通知类型分别处理
            match notification {
                // 连接状态发生变化
                ConnectionNotification::StateChanged(new_state) => {
                    // 将状态枚举映射为原子整数值
                    let state_val = match new_state {
                        // Disconnected = 0
                        ConnectionState::Disconnected => 0,
                        // Connecting = 1
                        ConnectionState::Connecting => 1,
                        // Connected = 2
                        ConnectionState::Connected => 2,
                        // Reconnecting = 3
                        ConnectionState::Reconnecting => 3,
                    };
                    // 原子更新状态值
                    state.store(state_val, Ordering::Relaxed);

                    // 通过事件总线发布通用状态变更事件
                    event_bus.publish("state_change", serde_json::json!({
                        "state": format!("{}", new_state),
                    }));

                    // 根据具体状态发布特定事件
                    match new_state {
                        // 已连接 — 发布 connect 事件
                        ConnectionState::Connected => {
                            event_bus.publish("connect", serde_json::json!({}));
                        }
                        // 已断开 — 发布 disconnect 事件
                        ConnectionState::Disconnected => {
                            event_bus.publish("disconnect", serde_json::json!({}));
                        }
                        // 重连中 — 发布 reconnecting 事件
                        ConnectionState::Reconnecting => {
                            event_bus.publish("reconnecting", serde_json::json!({}));
                        }
                        // 连接中 — 无需额外事件
                        ConnectionState::Connecting => {}
                    }
                }
                // 收到 WebSocket 消息
                ConnectionNotification::Message(msg) => {
                    // 将消息交给 Dispatcher 分发（区分 API 响应和事件）
                    dispatcher.dispatch(&msg);
                }
                // 连接彻底丢失（达到最大重连次数）
                ConnectionNotification::ConnectionLost { attempts } => {
                    // 发布连接丢失事件，携带时间戳和已尝试次数
                    event_bus.publish("connection:lost", serde_json::json!({
                        "timestamp": crate::api::client::now_ms(),
                        "attempts": attempts,
                    }));
                }
                // 连接恢复（重连成功）
                ConnectionNotification::ConnectionRestored { timestamp } => {
                    // 发布连接恢复事件，携带恢复时间戳
                    event_bus.publish("connection:restored", serde_json::json!({
                        "timestamp": timestamp,
                    }));
                }
            }
        }
    }
}

/// 手动实现 Debug trait — 因为 JoinHandle 和部分内部类型不支持 derive(Debug)
impl std::fmt::Debug for NapLink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 输出结构体名称和关键状态信息
        f.debug_struct("NapLink")
            // 显示 WebSocket 连接 URL
            .field("url", &self.config.connection.url)
            // 显示当前连接状态
            .field("state", &self.state())
            .finish()
    }
}

impl Drop for NapLink {
    /// 析构时清理资源
    ///
    /// 断开 WebSocket 连接并清理所有待处理的 API 请求，
    /// 确保不会有资源泄漏。
    fn drop(&mut self) {
        // 发送断开命令给 ConnectionActor
        self.conn_handle.disconnect();
        // 清理所有待处理的 API 请求
        self.api_client.destroy();
    }
}

/// NapLink 构建器 — 链式配置并创建 NapLink 客户端实例
///
/// 封装 `NapLinkConfigBuilder`，提供简洁的 API 来配置所有参数。
/// 仅 `url` 为必填字段，其余参数均有合理的默认值。
///
/// # 用法示例
///
/// ```rust,no_run
/// use napcat_link::NapLink;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = NapLink::builder("ws://127.0.0.1:3001")
///     .token("your_token")
///     .connection_timeout_ms(10000)
///     .reconnect_enabled(true)
///     .build()?;
/// # Ok(())
/// # }
/// ```
pub struct NapLinkBuilder {
    /// 内部配置构建器，封装所有配置参数
    config_builder: crate::config::NapLinkConfigBuilder,
}

impl NapLinkBuilder {
    /// 创建新的构建器实例
    ///
    /// # 参数
    ///
    /// - `url`: WebSocket 服务器地址
    fn new(url: &str) -> Self {
        Self {
            // 使用 NapLinkConfig::builder 创建内部配置构建器
            config_builder: NapLinkConfig::builder(url),
        }
    }

    /// 设置认证 Token
    ///
    /// # 参数
    ///
    /// - `token`: 用于 WebSocket 连接鉴权的 Token 字符串
    pub fn token(mut self, token: &str) -> Self {
        // 代理到内部配置构建器
        self.config_builder = self.config_builder.token(token);
        // 返回自身以支持链式调用
        self
    }

    /// 设置连接超时时间
    ///
    /// # 参数
    ///
    /// - `ms`: 连接超时时间（毫秒），默认 30000ms
    pub fn connection_timeout_ms(mut self, ms: u64) -> Self {
        // 代理到内部配置构建器
        self.config_builder = self.config_builder.connection_timeout_ms(ms);
        // 返回自身以支持链式调用
        self
    }

    /// 设置心跳 Ping 发送间隔
    ///
    /// # 参数
    ///
    /// - `ms`: 心跳间隔（毫秒），默认 30000ms
    pub fn ping_interval_ms(mut self, ms: u64) -> Self {
        // 代理到内部配置构建器
        self.config_builder = self.config_builder.ping_interval_ms(ms);
        // 返回自身以支持链式调用
        self
    }

    /// 设置是否启用自动重连
    ///
    /// # 参数
    ///
    /// - `enabled`: true 启用，false 禁用，默认 true
    pub fn reconnect_enabled(mut self, enabled: bool) -> Self {
        // 代理到内部配置构建器
        self.config_builder = self.config_builder.reconnect_enabled(enabled);
        // 返回自身以支持链式调用
        self
    }

    /// 设置最大重连尝试次数
    ///
    /// # 参数
    ///
    /// - `attempts`: 最大重连次数，默认 5 次
    pub fn reconnect_max_attempts(mut self, attempts: u32) -> Self {
        // 代理到内部配置构建器
        self.config_builder = self.config_builder.reconnect_max_attempts(attempts);
        // 返回自身以支持链式调用
        self
    }

    /// 设置退避策略的初始延迟时间
    ///
    /// # 参数
    ///
    /// - `ms`: 初始退避时间（毫秒），默认 5000ms
    pub fn backoff_initial_ms(mut self, ms: u64) -> Self {
        // 代理到内部配置构建器
        self.config_builder = self.config_builder.backoff_initial_ms(ms);
        // 返回自身以支持链式调用
        self
    }

    /// 设置退避策略的最大延迟时间
    ///
    /// # 参数
    ///
    /// - `ms`: 最大退避时间（毫秒），默认 60000ms
    pub fn backoff_max_ms(mut self, ms: u64) -> Self {
        // 代理到内部配置构建器
        self.config_builder = self.config_builder.backoff_max_ms(ms);
        // 返回自身以支持链式调用
        self
    }

    /// 设置退避策略的乘数因子
    ///
    /// # 参数
    ///
    /// - `multiplier`: 退避乘数，默认 2.0
    pub fn backoff_multiplier(mut self, multiplier: f64) -> Self {
        // 代理到内部配置构建器
        self.config_builder = self.config_builder.backoff_multiplier(multiplier);
        // 返回自身以支持链式调用
        self
    }

    /// 设置日志级别
    ///
    /// # 参数
    ///
    /// - `level`: 日志输出级别，默认 `LogLevel::Info`
    pub fn log_level(mut self, level: crate::config::LogLevel) -> Self {
        // 代理到内部配置构建器
        self.config_builder = self.config_builder.log_level(level);
        // 返回自身以支持链式调用
        self
    }

    /// 设置 API 调用超时时间
    ///
    /// # 参数
    ///
    /// - `ms`: API 超时时间（毫秒），默认 15000ms
    pub fn api_timeout_ms(mut self, ms: u64) -> Self {
        // 代理到内部配置构建器
        self.config_builder = self.config_builder.api_timeout_ms(ms);
        // 返回自身以支持链式调用
        self
    }

    /// 设置 API 调用重试次数
    ///
    /// # 参数
    ///
    /// - `retries`: 重试次数，默认 2 次
    pub fn api_retries(mut self, retries: u32) -> Self {
        // 代理到内部配置构建器
        self.config_builder = self.config_builder.api_retries(retries);
        // 返回自身以支持链式调用
        self
    }

    /// 构建 NapLink 实例
    ///
    /// 验证配置参数，创建所有内部组件，并启动后台通知处理任务。
    /// 创建后客户端处于未连接状态，需要调用 `connect()` 建立连接。
    ///
    /// # 返回值
    ///
    /// - `Ok(NapLink)`: 创建成功
    /// - `Err(NapLinkError)`: 配置验证失败
    pub fn build(self) -> Result<NapLink> {
        // 步骤 1：构建并验证配置
        let config = Arc::new(self.config_builder.build()?);

        // 步骤 2：创建事件总线（容量 256，足以缓冲高频事件）
        let event_bus = Arc::new(EventBus::new(256));

        // 步骤 3：创建事件路由器（持有事件总线的 Arc 引用）
        let event_router = EventRouter::new(event_bus.clone());

        // 步骤 4：创建通知 channel 并启动 ConnectionActor
        let (notification_tx, notification_rx) = mpsc::channel(256);
        // 通过 ConnectionHandle::new 内部启动 Actor，返回连接控制句柄
        let conn_handle = ConnectionHandle::new(
            (*config).clone(),
            notification_tx,
        );

        // 步骤 5：创建 API 客户端（持有连接句柄和配置的 Arc 引用）
        let api_client = ApiClient::new(conn_handle.clone(), config.clone());

        // 步骤 6：创建消息分发器（持有 API 客户端和事件路由器）
        let dispatcher = Arc::new(Dispatcher::new(api_client.clone(), event_router));

        // 步骤 7：创建 OneBotApi 聚合器（持有 API 客户端的 Clone）
        let onebot_api = OneBotApi::new(api_client.clone());

        // 步骤 8：初始化连接状态为 Disconnected（0）
        let state = Arc::new(AtomicU8::new(0));

        // 步骤 9：启动后台通知处理任务
        let notification_task = {
            // Clone 共享资源用于 task
            let state = state.clone();
            let event_bus = event_bus.clone();
            let dispatcher = dispatcher.clone();
            // 在独立 tokio task 中运行通知处理循环
            tokio::spawn(async move {
                NapLink::notification_loop(notification_rx, state, event_bus, dispatcher).await;
            })
        };

        // 记录初始化完成日志
        tracing::info!("NapLink 客户端已初始化");

        // 组装并返回 NapLink 实例
        Ok(NapLink {
            config,
            conn_handle,
            event_bus,
            api_client,
            onebot_api,
            _dispatcher: dispatcher,
            state,
            _notification_task: notification_task,
        })
    }
}
