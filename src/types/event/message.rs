//! # 消息事件模块
//!
//! 定义 OneBot 11 协议中的消息事件数据结构，包括：
//! - `PrivateMessageEvent`：私聊消息事件（好友、群临时会话、其他来源）
//! - `GroupMessageEvent`：群聊消息事件（普通消息、匿名消息、系统通知）

// 引入 serde 的序列化/反序列化派生宏
use serde::{Deserialize, Serialize};

// 引入消息段类型定义（用于消息内容字段）
use crate::types::message::MessageSegment;
// 引入事件公共定义中的发送者信息和匿名信息
use crate::types::event::shared::{Anonymous, Sender};

/// 私聊消息事件
///
/// 表示 OneBot 11 协议中收到的私聊消息。
/// 可能来源于好友消息、群临时会话消息或其他来源。
///
/// ## JSON 示例
///
/// ```json
/// {
///     "time": 1700000000,
///     "self_id": 123456789,
///     "post_type": "message",
///     "message_type": "private",
///     "sub_type": "friend",
///     "message_id": 1001,
///     "user_id": 987654321,
///     "message": [{"type": "text", "data": {"text": "你好"}}],
///     "raw_message": "你好",
///     "font": 0,
///     "sender": {"user_id": 987654321, "nickname": "测试用户"}
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateMessageEvent {
    /// 事件发生的 Unix 时间戳（秒级精度）
    pub time: i64,

    /// 收到事件的机器人 QQ 号
    pub self_id: i64,

    /// 上报类型，固定为 "message"
    pub post_type: String,

    /// 消息类型，固定为 "private"（私聊）
    pub message_type: String,

    /// 消息子类型：
    /// - "friend"：好友消息
    /// - "group"：群临时会话消息
    /// - "other"：其他来源的私聊消息
    pub sub_type: String,

    /// 消息 ID（服务端分配的唯一标识，用于撤回、回复等操作）
    pub message_id: i64,

    /// 发送者 QQ 号
    pub user_id: i64,

    /// 消息内容（消息段数组，结构化的消息表示）
    pub message: Vec<MessageSegment>,

    /// 原始消息内容（CQ 码格式的纯文本表示）
    pub raw_message: String,

    /// 字体 ID（通常为 0，保留字段）
    pub font: i32,

    /// 发送者信息（包含昵称、性别、年龄等）
    pub sender: Sender,

    /// 目标 QQ 号（可选，在某些场景下表示消息的目标接收者）
    #[serde(default)]
    pub target_id: Option<i64>,
}

/// 群聊消息事件
///
/// 表示 OneBot 11 协议中收到的群聊消息。
/// 可能是普通群消息、匿名消息或系统通知。
///
/// ## JSON 示例
///
/// ```json
/// {
///     "time": 1700000000,
///     "self_id": 123456789,
///     "post_type": "message",
///     "message_type": "group",
///     "sub_type": "normal",
///     "message_id": 2001,
///     "group_id": 100200300,
///     "user_id": 987654321,
///     "message": [{"type": "text", "data": {"text": "大家好"}}],
///     "raw_message": "大家好",
///     "font": 0,
///     "sender": {"user_id": 987654321, "nickname": "群成员", "role": "member"}
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMessageEvent {
    /// 事件发生的 Unix 时间戳（秒级精度）
    pub time: i64,

    /// 收到事件的机器人 QQ 号
    pub self_id: i64,

    /// 上报类型，固定为 "message"
    pub post_type: String,

    /// 消息类型，固定为 "group"（群消息）
    pub message_type: String,

    /// 消息子类型：
    /// - "normal"：普通群消息
    /// - "anonymous"：匿名消息
    /// - "notice"：系统提示（如管理员操作提示等）
    pub sub_type: String,

    /// 消息 ID（服务端分配的唯一标识，用于撤回、回复等操作）
    pub message_id: i64,

    /// 群号
    pub group_id: i64,

    /// 发送者 QQ 号
    pub user_id: i64,

    /// 消息内容（消息段数组，结构化的消息表示）
    pub message: Vec<MessageSegment>,

    /// 原始消息内容（CQ 码格式的纯文本表示）
    pub raw_message: String,

    /// 字体 ID（通常为 0，保留字段）
    pub font: i32,

    /// 发送者信息（包含昵称、群名片、群角色等）
    pub sender: Sender,

    /// 匿名信息（可选，仅当 sub_type 为 "anonymous" 时存在）
    #[serde(default)]
    pub anonymous: Option<Anonymous>,
}
