//! # 通知事件模块
//!
//! 定义 OneBot 11 协议中的通知事件数据结构，包括：
//! - `GroupRecallNotice`：群消息撤回通知
//! - `FriendRecallNotice`：好友消息撤回通知
//! - `GroupUploadNotice`：群文件上传通知
//! - `GroupAdminNotice`：群管理员变动通知
//! - `GroupDecreaseNotice`：群成员减少通知
//! - `GroupIncreaseNotice`：群成员增加通知
//! - `FriendAddNotice`：好友添加通知
//! - `PokeNotice`：戳一戳通知
//! - `GroupGrayTipNotice`：群灰色提示条通知

// 引入 serde 的序列化/反序列化派生宏
use serde::{Deserialize, Serialize};

// 引入事件公共定义中的文件信息结构体
use crate::types::event::shared::FileInfo;

/// 群消息撤回通知事件
///
/// 当群内某条消息被撤回时触发。
/// 包含操作者信息（谁执行了撤回）和被撤回消息的发送者信息。
///
/// ## JSON 示例
///
/// ```json
/// {
///     "time": 1700000000,
///     "self_id": 123456789,
///     "post_type": "notice",
///     "notice_type": "group_recall",
///     "group_id": 100200300,
///     "user_id": 987654321,
///     "operator_id": 111222333,
///     "message_id": 1001
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupRecallNotice {
    /// 事件发生的 Unix 时间戳（秒级精度）
    pub time: i64,

    /// 收到事件的机器人 QQ 号
    pub self_id: i64,

    /// 上报类型，固定为 "notice"
    pub post_type: String,

    /// 通知类型，固定为 "group_recall"
    pub notice_type: String,

    /// 群号
    pub group_id: i64,

    /// 被撤回消息的发送者 QQ 号
    pub user_id: i64,

    /// 执行撤回操作的用户 QQ 号（可以是管理员或消息发送者自己）
    pub operator_id: i64,

    /// 被撤回消息的 ID
    pub message_id: i64,
}

/// 好友消息撤回通知事件
///
/// 当好友撤回了发送给你的消息时触发。
///
/// ## JSON 示例
///
/// ```json
/// {
///     "time": 1700000000,
///     "self_id": 123456789,
///     "post_type": "notice",
///     "notice_type": "friend_recall",
///     "user_id": 987654321,
///     "message_id": 2001
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendRecallNotice {
    /// 事件发生的 Unix 时间戳（秒级精度）
    pub time: i64,

    /// 收到事件的机器人 QQ 号
    pub self_id: i64,

    /// 上报类型，固定为 "notice"
    pub post_type: String,

    /// 通知类型，固定为 "friend_recall"
    pub notice_type: String,

    /// 撤回消息的好友 QQ 号
    pub user_id: i64,

    /// 被撤回消息的 ID
    pub message_id: i64,
}

/// 群文件上传通知事件
///
/// 当群内有成员上传文件时触发。
/// 包含上传者信息和文件详情。
///
/// ## JSON 示例
///
/// ```json
/// {
///     "time": 1700000000,
///     "self_id": 123456789,
///     "post_type": "notice",
///     "notice_type": "group_upload",
///     "group_id": 100200300,
///     "user_id": 987654321,
///     "file": {"id": "abc", "name": "doc.pdf", "size": 1024, "busid": 102}
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupUploadNotice {
    /// 事件发生的 Unix 时间戳（秒级精度）
    pub time: i64,

    /// 收到事件的机器人 QQ 号
    pub self_id: i64,

    /// 上报类型，固定为 "notice"
    pub post_type: String,

    /// 通知类型，固定为 "group_upload"
    pub notice_type: String,

    /// 群号
    pub group_id: i64,

    /// 上传文件的用户 QQ 号
    pub user_id: i64,

    /// 上传的文件信息（包含文件 ID、名称、大小、总线 ID）
    pub file: FileInfo,
}

/// 群管理员变动通知事件
///
/// 当群内有成员被设为管理员或被取消管理员时触发。
///
/// ## JSON 示例
///
/// ```json
/// {
///     "time": 1700000000,
///     "self_id": 123456789,
///     "post_type": "notice",
///     "notice_type": "group_admin",
///     "sub_type": "set",
///     "group_id": 100200300,
///     "user_id": 987654321
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupAdminNotice {
    /// 事件发生的 Unix 时间戳（秒级精度）
    pub time: i64,

    /// 收到事件的机器人 QQ 号
    pub self_id: i64,

    /// 上报类型，固定为 "notice"
    pub post_type: String,

    /// 通知类型，固定为 "group_admin"
    pub notice_type: String,

    /// 变动子类型：
    /// - "set"：被设置为管理员
    /// - "unset"：被取消管理员
    pub sub_type: String,

    /// 群号
    pub group_id: i64,

    /// 被变动的成员 QQ 号
    pub user_id: i64,
}

/// 群成员减少通知事件
///
/// 当群内有成员离开（主动退群、被踢出、自己被踢出）时触发。
///
/// ## JSON 示例
///
/// ```json
/// {
///     "time": 1700000000,
///     "self_id": 123456789,
///     "post_type": "notice",
///     "notice_type": "group_decrease",
///     "sub_type": "kick",
///     "group_id": 100200300,
///     "operator_id": 111222333,
///     "user_id": 987654321
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupDecreaseNotice {
    /// 事件发生的 Unix 时间戳（秒级精度）
    pub time: i64,

    /// 收到事件的机器人 QQ 号
    pub self_id: i64,

    /// 上报类型，固定为 "notice"
    pub post_type: String,

    /// 通知类型，固定为 "group_decrease"
    pub notice_type: String,

    /// 成员减少子类型：
    /// - "leave"：成员主动退群
    /// - "kick"：成员被管理员踢出
    /// - "kick_me"：机器人自己被踢出
    pub sub_type: String,

    /// 群号
    pub group_id: i64,

    /// 执行操作的用户 QQ 号（踢人时为管理员，退群时为离开者自己）
    pub operator_id: i64,

    /// 离开群的成员 QQ 号
    pub user_id: i64,
}

/// 群成员增加通知事件
///
/// 当群内有新成员加入（管理员审批通过或被其他成员邀请）时触发。
///
/// ## JSON 示例
///
/// ```json
/// {
///     "time": 1700000000,
///     "self_id": 123456789,
///     "post_type": "notice",
///     "notice_type": "group_increase",
///     "sub_type": "approve",
///     "group_id": 100200300,
///     "operator_id": 111222333,
///     "user_id": 987654321
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupIncreaseNotice {
    /// 事件发生的 Unix 时间戳（秒级精度）
    pub time: i64,

    /// 收到事件的机器人 QQ 号
    pub self_id: i64,

    /// 上报类型，固定为 "notice"
    pub post_type: String,

    /// 通知类型，固定为 "group_increase"
    pub notice_type: String,

    /// 成员增加子类型：
    /// - "approve"：管理员审批通过加群申请
    /// - "invite"：被其他成员邀请入群
    pub sub_type: String,

    /// 群号
    pub group_id: i64,

    /// 操作者 QQ 号（审批时为管理员，邀请时为邀请者）
    pub operator_id: i64,

    /// 新加入群的成员 QQ 号
    pub user_id: i64,
}

/// 好友添加通知事件
///
/// 当有新好友添加成功时触发。
///
/// ## JSON 示例
///
/// ```json
/// {
///     "time": 1700000000,
///     "self_id": 123456789,
///     "post_type": "notice",
///     "notice_type": "friend_add",
///     "user_id": 987654321
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendAddNotice {
    /// 事件发生的 Unix 时间戳（秒级精度）
    pub time: i64,

    /// 收到事件的机器人 QQ 号
    pub self_id: i64,

    /// 上报类型，固定为 "notice"
    pub post_type: String,

    /// 通知类型，固定为 "friend_add"
    pub notice_type: String,

    /// 新添加的好友 QQ 号
    pub user_id: i64,
}

/// 戳一戳通知事件
///
/// 当有人在群内或私聊中发起戳一戳（窗口抖动）时触发。
/// group_id 为可选字段，存在时表示群内戳一戳，不存在时表示私聊戳一戳。
///
/// ## JSON 示例
///
/// ```json
/// {
///     "time": 1700000000,
///     "self_id": 123456789,
///     "post_type": "notice",
///     "notice_type": "notify",
///     "sub_type": "poke",
///     "group_id": 100200300,
///     "user_id": 987654321,
///     "target_id": 123456789
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokeNotice {
    /// 事件发生的 Unix 时间戳（秒级精度）
    pub time: i64,

    /// 收到事件的机器人 QQ 号
    pub self_id: i64,

    /// 上报类型，固定为 "notice"
    pub post_type: String,

    /// 通知类型，固定为 "notify"
    pub notice_type: String,

    /// 通知子类型，固定为 "poke"（戳一戳）
    pub sub_type: String,

    /// 群号（可选，存在时表示群内戳一戳，不存在时表示私聊戳一戳）
    #[serde(default)]
    pub group_id: Option<i64>,

    /// 发起戳一戳的用户 QQ 号
    pub user_id: i64,

    /// 被戳的目标用户 QQ 号
    pub target_id: i64,

    /// 发送者 QQ 号（可选，某些实现中可能额外提供）
    #[serde(default)]
    pub sender_id: Option<i64>,
}

/// 群灰色提示条通知事件
///
/// 群聊中的灰色系统提示消息（如成员入群提示、群公告变更等）。
/// 这是 NapCat 扩展的通知类型，notice_type 为 "notify"，sub_type 为 "gray_tip"。
///
/// ## JSON 示例
///
/// ```json
/// {
///     "time": 1700000000,
///     "self_id": 123456789,
///     "post_type": "notice",
///     "notice_type": "notify",
///     "sub_type": "gray_tip",
///     "group_id": 100200300,
///     "user_id": 987654321,
///     "content": "欢迎新成员加入群聊",
///     "message_id": 1001,
///     "busi_id": "10",
///     "raw_info": null
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupGrayTipNotice {
    /// 事件发生的 Unix 时间戳（秒级精度）
    pub time: i64,

    /// 收到事件的机器人 QQ 号
    pub self_id: i64,

    /// 上报类型，固定为 "notice"
    pub post_type: String,

    /// 通知类型，固定为 "notify"
    pub notice_type: String,

    /// 通知子类型，固定为 "gray_tip"
    pub sub_type: String,

    /// 群号
    pub group_id: i64,

    /// 触发灰条提示的用户 QQ 号
    pub user_id: i64,

    /// 灰条提示内容
    pub content: String,

    /// 关联的消息 ID（可选，某些灰条提示可能关联特定消息）
    #[serde(default)]
    pub message_id: Option<i64>,

    /// 业务 ID（可选，用于标识灰条提示的业务类型）
    #[serde(default)]
    pub busi_id: Option<String>,

    /// 原始信息（可选，包含灰条提示的原始 JSON 数据）
    #[serde(default)]
    pub raw_info: Option<serde_json::Value>,
}
