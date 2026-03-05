//! # 群组 API 模块
//!
//! 封装 OneBot 11 群组管理相关的 API 调用，包括：
//! - 群禁言（set_group_ban / set_group_whole_ban）
//! - 群踢人（set_group_kick）
//! - 退出群组（set_group_leave）
//! - 设置群名片（set_group_card）
//! - 设置群名（set_group_name）
//! - 设置群管理员（set_group_admin）
//! - 匿名用户禁言（set_group_anonymous_ban）
//! - 设置群专属头衔（set_group_special_title）
//! - 点赞（send_like）

// 引入 serde_json 的 json! 宏和 Value 类型，用于构建 API 请求参数
use serde_json::{Value, json};

// 引入 API 客户端，用于发送 API 请求
use super::client::ApiClient;

// 引入 SDK 的 Result 类型别名
use crate::error::Result;

/// 群组 API — 封装所有群组管理相关的 OneBot 11 API 调用
///
/// 通过持有 `ApiClient` 的克隆实例来发送请求。
/// `ApiClient` 内部使用 `Arc` 共享状态，Clone 成本极低。
#[derive(Clone)]
pub struct GroupApi {
    /// API 客户端实例（内部通过 Arc 共享状态，Clone 成本低）
    client: ApiClient,
}

impl GroupApi {
    /// 创建新的群组 API 实例
    ///
    /// # 参数
    ///
    /// - `client`: API 客户端实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的 `GroupApi` 实例
    pub fn new(client: ApiClient) -> Self {
        // 保存 API 客户端引用
        Self { client }
    }

    /// 群组禁言指定成员
    ///
    /// 对应 OneBot action: `set_group_ban`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    /// - `user_id`: 要禁言的成员 QQ 号
    /// - `duration`: 禁言时长（秒），默认 1800（30 分钟），0 表示解除禁言
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn set_group_ban(
        &self,
        group_id: i64,
        user_id: i64,
        duration: Option<i64>,
    ) -> Result<Value> {
        // 调用 set_group_ban action，duration 默认 1800 秒
        self.client.call("set_group_ban", json!({
            "group_id": group_id,
            "user_id": user_id,
            "duration": duration.unwrap_or(1800),
        })).await
    }

    /// 解除群组成员禁言
    ///
    /// 对应 OneBot action: `set_group_ban`（duration = 0 表示解除禁言）
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    /// - `user_id`: 要解除禁言的成员 QQ 号
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn unset_group_ban(
        &self,
        group_id: i64,
        user_id: i64,
    ) -> Result<Value> {
        // 调用 set_group_ban action，duration = 0 表示解除禁言
        self.client.call("set_group_ban", json!({
            "group_id": group_id,
            "user_id": user_id,
            "duration": 0,
        })).await
    }

    /// 群组全员禁言
    ///
    /// 对应 OneBot action: `set_group_whole_ban`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    /// - `enable`: 是否开启全员禁言（默认 true）
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn set_group_whole_ban(
        &self,
        group_id: i64,
        enable: Option<bool>,
    ) -> Result<Value> {
        // 调用 set_group_whole_ban action，enable 默认 true
        self.client.call("set_group_whole_ban", json!({
            "group_id": group_id,
            "enable": enable.unwrap_or(true),
        })).await
    }

    /// 群组踢人
    ///
    /// 对应 OneBot action: `set_group_kick`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    /// - `user_id`: 要踢出的成员 QQ 号
    /// - `reject_add_request`: 是否拒绝此人再次加群（默认 false）
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn set_group_kick(
        &self,
        group_id: i64,
        user_id: i64,
        reject_add_request: Option<bool>,
    ) -> Result<Value> {
        // 调用 set_group_kick action，reject_add_request 默认 false
        self.client.call("set_group_kick", json!({
            "group_id": group_id,
            "user_id": user_id,
            "reject_add_request": reject_add_request.unwrap_or(false),
        })).await
    }

    /// 退出群组
    ///
    /// 对应 OneBot action: `set_group_leave`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    /// - `is_dismiss`: 是否解散群（仅群主可用，默认 false）
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn set_group_leave(
        &self,
        group_id: i64,
        is_dismiss: Option<bool>,
    ) -> Result<Value> {
        // 调用 set_group_leave action，is_dismiss 默认 false
        self.client.call("set_group_leave", json!({
            "group_id": group_id,
            "is_dismiss": is_dismiss.unwrap_or(false),
        })).await
    }

    /// 设置群名片（群备注）
    ///
    /// 对应 OneBot action: `set_group_card`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    /// - `user_id`: 要设置群名片的成员 QQ 号
    /// - `card`: 群名片内容（空字符串表示删除群名片）
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn set_group_card(
        &self,
        group_id: i64,
        user_id: i64,
        card: &str,
    ) -> Result<Value> {
        // 调用 set_group_card action，传入群号、用户 ID 和名片内容
        self.client.call("set_group_card", json!({
            "group_id": group_id,
            "user_id": user_id,
            "card": card,
        })).await
    }

    /// 设置群名
    ///
    /// 对应 OneBot action: `set_group_name`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    /// - `group_name`: 新的群名
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn set_group_name(
        &self,
        group_id: i64,
        group_name: &str,
    ) -> Result<Value> {
        // 调用 set_group_name action，传入群号和新群名
        self.client.call("set_group_name", json!({
            "group_id": group_id,
            "group_name": group_name,
        })).await
    }

    /// 设置群管理员
    ///
    /// 对应 OneBot action: `set_group_admin`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    /// - `user_id`: 要设置/取消管理员的成员 QQ 号
    /// - `enable`: 是否设为管理员（默认 true，false 为取消管理员）
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn set_group_admin(
        &self,
        group_id: i64,
        user_id: i64,
        enable: Option<bool>,
    ) -> Result<Value> {
        // 调用 set_group_admin action，enable 默认 true
        self.client.call("set_group_admin", json!({
            "group_id": group_id,
            "user_id": user_id,
            "enable": enable.unwrap_or(true),
        })).await
    }

    /// 对匿名用户禁言
    ///
    /// 对应 OneBot action: `set_group_anonymous_ban`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    /// - `anonymous_flag`: 匿名用户的 flag（从匿名消息事件中获取）
    /// - `duration`: 禁言时长（秒），默认 1800（30 分钟），0 表示解除禁言
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn set_group_anonymous_ban(
        &self,
        group_id: i64,
        anonymous_flag: &str,
        duration: Option<i64>,
    ) -> Result<Value> {
        // 调用 set_group_anonymous_ban action，duration 默认 1800 秒（30 分钟）
        self.client.call("set_group_anonymous_ban", json!({
            "group_id": group_id,
            "anonymous_flag": anonymous_flag,
            "duration": duration.unwrap_or(1800),
        })).await
    }

    /// 设置群专属头衔
    ///
    /// 对应 OneBot action: `set_group_special_title`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    /// - `user_id`: 要设置头衔的成员 QQ 号
    /// - `special_title`: 专属头衔内容（空字符串表示删除头衔）
    /// - `duration`: 头衔有效期（秒，-1 表示永久，部分服务端可能不支持此参数）
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn set_group_special_title(
        &self,
        group_id: i64,
        user_id: i64,
        special_title: &str,
        duration: Option<i64>,
    ) -> Result<Value> {
        // 调用 set_group_special_title action，duration 默认 -1（永久）
        self.client.call("set_group_special_title", json!({
            "group_id": group_id,
            "user_id": user_id,
            "special_title": special_title,
            "duration": duration.unwrap_or(-1),
        })).await
    }

    /// 发送好友赞（点赞）
    ///
    /// 对应 OneBot action: `send_like`
    ///
    /// # 参数
    ///
    /// - `user_id`: 目标用户 QQ 号
    /// - `times`: 点赞次数（默认 1，每个好友每天最多 10 次）
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn send_like(
        &self,
        user_id: i64,
        times: Option<i32>,
    ) -> Result<Value> {
        // 调用 send_like action，times 默认 1
        self.client.call("send_like", json!({
            "user_id": user_id,
            "times": times.unwrap_or(1),
        })).await
    }
}
