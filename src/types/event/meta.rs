//! # 元事件模块
//!
//! 定义 OneBot 11 协议中的元事件数据结构，包括：
//! - `LifecycleEvent`：生命周期事件（连接建立、启用、禁用）
//! - `HeartbeatEvent`：心跳事件（定期发送，包含机器人状态信息）

// 引入 serde 的序列化/反序列化派生宏
use serde::{Deserialize, Serialize};

// 引入事件公共定义中的机器人状态结构体
use crate::types::event::shared::BotStatus;

/// 生命周期元事件
///
/// 在 OneBot 实现的生命周期关键节点触发。
/// 通常在 WebSocket 连接建立时收到 sub_type 为 "connect" 的事件。
///
/// ## JSON 示例
///
/// ```json
/// {
///     "time": 1700000000,
///     "self_id": 123456789,
///     "post_type": "meta_event",
///     "meta_event_type": "lifecycle",
///     "sub_type": "connect"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleEvent {
    /// 事件发生的 Unix 时间戳（秒级精度）
    pub time: i64,

    /// 收到事件的机器人 QQ 号
    pub self_id: i64,

    /// 上报类型，固定为 "meta_event"
    pub post_type: String,

    /// 元事件类型，固定为 "lifecycle"
    pub meta_event_type: String,

    /// 生命周期子类型：
    /// - "enable"：OneBot 实现启用
    /// - "disable"：OneBot 实现禁用
    /// - "connect"：WebSocket 连接成功建立
    pub sub_type: String,
}

/// 心跳元事件
///
/// OneBot 实现定期发送的心跳事件，用于监控连接状态和机器人运行状况。
/// 包含机器人当前状态信息和心跳间隔时间。
///
/// ## JSON 示例
///
/// ```json
/// {
///     "time": 1700000000,
///     "self_id": 123456789,
///     "post_type": "meta_event",
///     "meta_event_type": "heartbeat",
///     "status": {"online": true, "good": true},
///     "interval": 5000
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatEvent {
    /// 事件发生的 Unix 时间戳（秒级精度）
    pub time: i64,

    /// 收到事件的机器人 QQ 号
    pub self_id: i64,

    /// 上报类型，固定为 "meta_event"
    pub post_type: String,

    /// 元事件类型，固定为 "heartbeat"
    pub meta_event_type: String,

    /// 机器人当前状态（包含在线状态和运行状况）
    pub status: BotStatus,

    /// 心跳间隔时间（毫秒），表示距离下次心跳事件的时间间隔
    pub interval: i64,
}
