//! # 事件公共定义模块
//!
//! 定义所有事件类型共享的基础字段和枚举，包括：
//! - `Sender`：消息发送者信息结构体
//! - `Anonymous`：匿名消息发送者信息结构体
//! - `FileInfo`：群文件上传时的文件信息结构体
//! - `BotStatus`：心跳元事件中的机器人状态结构体

// 引入 serde 的序列化/反序列化派生宏
use serde::{Deserialize, Serialize};

/// 消息发送者信息
///
/// 表示 OneBot 11 协议中消息事件携带的发送者信息。
/// 在私聊消息和群聊消息事件中均会出现。
/// 所有字段均为可选，因为不同场景下服务端返回的字段可能不完整。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sender {
    /// 发送者 QQ 号（可选，某些情况下可能缺失）
    #[serde(default)]
    pub user_id: Option<i64>,

    /// 发送者昵称（可选）
    #[serde(default)]
    pub nickname: Option<String>,

    /// 发送者性别：可选值为 "male"（男）、"female"（女）、"unknown"（未知）
    #[serde(default)]
    pub sex: Option<String>,

    /// 发送者年龄（可选）
    #[serde(default)]
    pub age: Option<i32>,

    /// 群名片/备注（可选，仅在群消息事件中出现）
    #[serde(default)]
    pub card: Option<String>,

    /// 地区（可选）
    #[serde(default)]
    pub area: Option<String>,

    /// 成员等级（可选，仅在群消息事件中出现）
    #[serde(default)]
    pub level: Option<String>,

    /// 群角色：可选值为 "owner"（群主）、"admin"（管理员）、"member"（普通成员）
    #[serde(default)]
    pub role: Option<String>,

    /// 群专属头衔（可选，仅在群消息事件中出现）
    #[serde(default)]
    pub title: Option<String>,
}

/// 匿名消息发送者信息
///
/// 表示群消息事件中匿名发送者的信息。
/// 仅在群消息 sub_type 为 "anonymous" 时存在。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anonymous {
    /// 匿名用户 ID（由服务端分配的临时标识）
    pub id: i64,

    /// 匿名用户显示名称
    pub name: String,

    /// 匿名用户标识 flag（用于禁言等操作时引用）
    pub flag: String,
}

/// 群文件上传信息
///
/// 表示群文件上传通知事件中携带的文件详情。
/// 包含文件的唯一标识、名称、大小和总线 ID。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// 文件 ID（服务端分配的唯一标识）
    pub id: String,

    /// 文件名称（含扩展名）
    pub name: String,

    /// 文件大小（字节数）
    pub size: i64,

    /// 文件总线 ID（用于下载等操作）
    pub busid: i64,
}

/// 机器人状态信息
///
/// 表示心跳元事件中携带的机器人运行状态。
/// 用于监控机器人是否在线及运行是否正常。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotStatus {
    /// 机器人是否在线（true 表示在线，false 表示离线）
    pub online: bool,

    /// 机器人运行状态是否良好（true 表示正常，false 表示异常）
    pub good: bool,
}
