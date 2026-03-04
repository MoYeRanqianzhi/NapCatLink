//! # 请求事件模块
//!
//! 定义 OneBot 11 协议中的请求事件数据结构，包括：
//! - `FriendRequestEvent`：好友添加请求事件
//! - `GroupRequestEvent`：群添加/邀请请求事件

// 引入 serde 的序列化/反序列化派生宏
use serde::{Deserialize, Serialize};

/// 好友添加请求事件
///
/// 当收到其他用户发来的好友添加请求时触发。
/// 可通过 `flag` 字段配合 API 进行同意或拒绝操作。
///
/// ## JSON 示例
///
/// ```json
/// {
///     "time": 1700000000,
///     "self_id": 123456789,
///     "post_type": "request",
///     "request_type": "friend",
///     "user_id": 987654321,
///     "comment": "我是你的朋友",
///     "flag": "flag_abc123"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendRequestEvent {
    /// 事件发生的 Unix 时间戳（秒级精度）
    pub time: i64,

    /// 收到事件的机器人 QQ 号
    pub self_id: i64,

    /// 上报类型，固定为 "request"
    pub post_type: String,

    /// 请求类型，固定为 "friend"（好友添加请求）
    pub request_type: String,

    /// 请求发起者的 QQ 号
    pub user_id: i64,

    /// 验证信息（请求附带的留言内容）
    pub comment: String,

    /// 请求标识 flag（用于调用 API 处理请求时引用此请求）
    pub flag: String,
}

/// 群添加/邀请请求事件
///
/// 当收到加群请求或被邀请入群时触发。
/// 可通过 `flag` 字段配合 API 进行同意或拒绝操作。
///
/// ## JSON 示例
///
/// ```json
/// {
///     "time": 1700000000,
///     "self_id": 123456789,
///     "post_type": "request",
///     "request_type": "group",
///     "sub_type": "add",
///     "group_id": 100200300,
///     "user_id": 987654321,
///     "comment": "请让我加入群聊",
///     "flag": "flag_xyz789"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupRequestEvent {
    /// 事件发生的 Unix 时间戳（秒级精度）
    pub time: i64,

    /// 收到事件的机器人 QQ 号
    pub self_id: i64,

    /// 上报类型，固定为 "request"
    pub post_type: String,

    /// 请求类型，固定为 "group"（群相关请求）
    pub request_type: String,

    /// 请求子类型：
    /// - "add"：用户主动申请加群
    /// - "invite"：机器人被邀请加入某群
    pub sub_type: String,

    /// 群号
    pub group_id: i64,

    /// 请求发起者的 QQ 号（加群请求时为申请者，邀请时为邀请者）
    pub user_id: i64,

    /// 验证信息（加群请求附带的留言内容）
    pub comment: String,

    /// 请求标识 flag（用于调用 API 处理请求时引用此请求）
    pub flag: String,
}
