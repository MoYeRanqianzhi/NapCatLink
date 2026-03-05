//! # 事件总线模块
//!
//! 基于 `tokio::sync::broadcast` channel 实现发布-订阅模式的事件分发中心。
//!
//! 核心功能：
//! - 事件发布：将事件广播给所有订阅者
//! - 事件订阅：创建订阅者以接收事件
//! - 事件过滤：支持按前缀或精确名称过滤事件

// 引入 tokio 的 broadcast channel，用于实现多生产者多消费者的广播通信
use tokio::sync::broadcast;

// 引入 serde_json::Value，用于存储事件的原始 JSON 数据
use serde_json::Value;

/// 事件数据结构 — 封装单条事件的名称和原始 JSON 负载
///
/// 每个事件由一个层级名称（如 `"message.group.normal"`）和对应的 JSON 数据组成。
/// 实现了 `Clone` 和 `Debug`，以便在 broadcast channel 中传递和调试输出。
#[derive(Debug, Clone)]
pub struct EventData {
    /// 事件名称，使用点号分隔的层级格式（如 "message.group.normal"）
    pub name: String,
    /// 事件的原始 JSON 数据，保留完整的 OneBot 11 事件字段
    pub data: Value,
}

/// 事件总线 — 基于 broadcast channel 的发布/订阅系统
///
/// 事件总线是事件系统的核心组件，负责：
/// - 接收事件发布请求，将事件广播给所有已注册的订阅者
/// - 管理 broadcast channel 的发送端
/// - 创建新的订阅（接收端）
///
/// # 用法示例
///
/// ```rust
/// use napcat_link::event::bus::EventBus;
/// use serde_json::json;
///
/// let bus = EventBus::new(256);
/// let mut sub = bus.subscribe();
/// bus.publish("message.group", json!({"text": "hello"}));
/// ```
pub struct EventBus {
    /// broadcast channel 的发送端，所有订阅者共享同一个 channel
    tx: broadcast::Sender<EventData>,
}

impl EventBus {
    /// 创建新的事件总线实例
    ///
    /// # 参数
    /// - `capacity`：broadcast channel 的缓冲区大小，决定了在没有订阅者消费时可以缓存多少条消息。
    ///   推荐值为 256，对于高频事件场景可以适当增大。
    ///
    /// # 返回值
    /// 返回一个新的 `EventBus` 实例
    pub fn new(capacity: usize) -> Self {
        // 创建 broadcast channel，返回发送端和一个初始接收端（初始接收端被丢弃）
        let (tx, _) = broadcast::channel(capacity);
        // 构造并返回 EventBus，仅保留发送端
        Self { tx }
    }

    /// 发布事件到所有当前活跃的订阅者
    ///
    /// 将事件数据封装为 `EventData` 并通过 broadcast channel 发送。
    /// 如果当前没有订阅者，消息会被静默丢弃（不会产生错误）。
    ///
    /// # 参数
    /// - `name`：事件名称，支持任何可以转换为 `String` 的类型
    /// - `data`：事件的 JSON 数据
    pub fn publish(&self, name: impl Into<String>, data: Value) {
        // 构造事件数据并通过 broadcast channel 发送
        // 使用 `let _ =` 忽略发送结果，因为没有订阅者时会返回错误，这是预期行为
        let _ = self.tx.send(EventData {
            // 将事件名称转换为 String
            name: name.into(),
            // 直接传递 JSON 数据
            data,
        });
    }

    /// 创建新的事件订阅
    ///
    /// 每次调用都会创建一个独立的订阅者，该订阅者只能接收到创建之后发布的事件。
    /// 创建之前已经发布的事件不会被接收到。
    ///
    /// # 返回值
    /// 返回一个 `EventSubscription` 实例，可用于接收和过滤事件
    pub fn subscribe(&self) -> EventSubscription {
        // 通过发送端创建新的接收端，封装为 EventSubscription
        EventSubscription {
            rx: self.tx.subscribe(),
        }
    }
}

/// 事件订阅 — 从事件总线接收并过滤事件
///
/// 订阅者通过 broadcast channel 的接收端接收事件，支持三种接收模式：
/// - `recv()`：接收所有事件
/// - `recv_filter()`：按事件名前缀过滤接收
/// - `recv_exact()`：按精确事件名接收
pub struct EventSubscription {
    /// broadcast channel 的接收端，用于从事件总线接收事件数据
    rx: broadcast::Receiver<EventData>,
}

impl EventSubscription {
    /// 接收下一个事件（异步阻塞等待）
    ///
    /// 持续等待直到接收到一个事件或 channel 关闭。
    /// 如果订阅者消费速度过慢导致消息滞后（lagged），会自动跳过丢失的消息并继续接收。
    ///
    /// # 返回值
    /// - `Some(EventData)`：成功接收到一个事件
    /// - `None`：channel 已关闭（所有发送端都已被丢弃）
    pub async fn recv(&mut self) -> Option<EventData> {
        // 循环接收，处理可能出现的滞后情况
        loop {
            // 异步等待下一条消息
            match self.rx.recv().await {
                // 成功接收到事件数据，直接返回
                Ok(data) => return Some(data),
                // 订阅者消费速度跟不上发布速度，部分消息被跳过
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    // 记录警告日志，标明跳过的消息数量
                    tracing::warn!("事件订阅者滞后，跳过 {} 条消息", n);
                    // 继续循环，尝试接收后续消息
                    continue;
                }
                // channel 已关闭，所有发送端都已被丢弃
                Err(broadcast::error::RecvError::Closed) => return None,
            }
        }
    }

    /// 接收下一个匹配指定前缀的事件（异步阻塞等待）
    ///
    /// 持续接收事件，跳过名称不以指定前缀开头的事件，直到找到匹配的事件或 channel 关闭。
    ///
    /// # 参数
    /// - `prefix`：事件名称前缀，例如 `"message"` 会匹配 `"message"`、`"message.group"`、`"message.group.normal"` 等
    ///
    /// # 返回值
    /// - `Some(EventData)`：接收到一个名称以指定前缀开头的事件
    /// - `None`：channel 已关闭
    pub async fn recv_filter(&mut self, prefix: &str) -> Option<EventData> {
        // 循环接收并过滤
        loop {
            // 调用 recv() 获取下一个事件
            match self.recv().await {
                // 接收到事件且名称精确匹配前缀，或名称以 "prefix." 开头（层级匹配），返回匹配的事件
                Some(data) if data.name == prefix || data.name.starts_with(&format!("{}.", prefix)) => return Some(data),
                // 接收到事件但名称不匹配，跳过继续等待下一个
                Some(_) => continue,
                // channel 已关闭，返回 None
                None => return None,
            }
        }
    }

    /// 接收下一个精确匹配指定名称的事件（异步阻塞等待）
    ///
    /// 持续接收事件，跳过名称不完全匹配的事件，直到找到精确匹配的事件或 channel 关闭。
    ///
    /// # 参数
    /// - `name`：要匹配的完整事件名称
    ///
    /// # 返回值
    /// - `Some(EventData)`：接收到一个名称精确匹配的事件
    /// - `None`：channel 已关闭
    pub async fn recv_exact(&mut self, name: &str) -> Option<EventData> {
        // 循环接收并精确匹配
        loop {
            // 调用 recv() 获取下一个事件
            match self.recv().await {
                // 接收到事件且名称完全匹配，返回该事件
                Some(data) if data.name == name => return Some(data),
                // 接收到事件但名称不匹配，跳过继续等待
                Some(_) => continue,
                // channel 已关闭，返回 None
                None => return None,
            }
        }
    }
}
