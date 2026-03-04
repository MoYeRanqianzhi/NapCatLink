//! # 消息 API 模块
//!
//! 封装 OneBot 11 消息相关的 API 调用，包括：
//! - 发送私聊消息（send_private_msg）
//! - 发送群聊消息（send_group_msg）
//! - 撤回消息（delete_msg）
//! - 获取消息详情（get_msg）
//! - 获取合并转发消息（get_forward_msg）
//! - 发送合并转发消息（send_group_forward_msg）
//! - 精华消息操作（set_essence_msg / delete_essence_msg / get_essence_msg_list）
//! - 消息已读标记（mark_msg_as_read / mark_group_msg_as_read 等）
//! - 群荣誉/系统消息查询
//! - 消息历史记录查询
//! - 表情回应（set_msg_emoji_like / fetch_emoji_like）
//! - 戳一戳（group_poke / friend_poke / send_poke）

// 引入 serde_json 的 json! 宏和 Value 类型，用于构建 API 请求参数
use serde_json::{Value, json};

// 引入 API 客户端，用于发送 API 请求
use super::client::ApiClient;

// 引入 SDK 的 Result 类型别名
use crate::error::Result;

// 引入消息段类型，用于消息发送方法的参数
use crate::types::message::MessageSegment;

/// 消息 API — 封装所有消息相关的 OneBot 11 API 调用
///
/// 通过持有 `ApiClient` 的克隆实例来发送请求。
/// `ApiClient` 是 `Clone` + `Send` + `Sync` 的，因此 `MessageApi` 也是。
pub struct MessageApi {
    /// API 客户端实例（内部通过 Arc 共享状态，Clone 成本低）
    client: ApiClient,
}

impl MessageApi {
    /// 创建新的消息 API 实例
    ///
    /// # 参数
    ///
    /// - `client`: API 客户端实例（会被 Clone 保存）
    ///
    /// # 返回值
    ///
    /// 返回一个新的 `MessageApi` 实例
    pub fn new(client: ApiClient) -> Self {
        // 保存 API 客户端引用
        Self { client }
    }

    // ========================================================================
    // 消息发送
    // ========================================================================

    /// 发送消息（通用）— 根据参数自动判断私聊或群聊
    ///
    /// 对应 OneBot action: `send_msg`
    ///
    /// # 参数
    ///
    /// - `params`: 完整的发送参数 JSON（包含 message_type、user_id/group_id、message 等）
    ///
    /// # 返回值
    ///
    /// 成功返回包含 `message_id` 的 JSON 数据
    pub async fn send_message(&self, params: Value) -> Result<Value> {
        // 调用 send_msg action，直接传递完整参数
        self.client.call("send_msg", params).await
    }

    /// 发送私聊消息
    ///
    /// 对应 OneBot action: `send_private_msg`
    ///
    /// # 参数
    ///
    /// - `user_id`: 目标用户的 QQ 号
    /// - `message`: 消息段数组（支持文本、图片、@、回复等组合）
    ///
    /// # 返回值
    ///
    /// 成功返回包含 `message_id` 的 JSON 数据
    pub async fn send_private_message(
        &self,
        user_id: i64,
        message: Vec<MessageSegment>,
    ) -> Result<Value> {
        // 调用 send_private_msg action，传入用户 ID 和消息内容
        self.client.call("send_private_msg", json!({
            "user_id": user_id,
            "message": message,
        })).await
    }

    /// 发送群聊消息
    ///
    /// 对应 OneBot action: `send_group_msg`
    ///
    /// # 参数
    ///
    /// - `group_id`: 目标群号
    /// - `message`: 消息段数组（支持文本、图片、@、回复等组合）
    ///
    /// # 返回值
    ///
    /// 成功返回包含 `message_id` 的 JSON 数据
    pub async fn send_group_message(
        &self,
        group_id: i64,
        message: Vec<MessageSegment>,
    ) -> Result<Value> {
        // 调用 send_group_msg action，传入群号和消息内容
        self.client.call("send_group_msg", json!({
            "group_id": group_id,
            "message": message,
        })).await
    }

    // ========================================================================
    // 消息操作
    // ========================================================================

    /// 撤回消息
    ///
    /// 对应 OneBot action: `delete_msg`
    ///
    /// # 参数
    ///
    /// - `message_id`: 要撤回的消息 ID
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn delete_message(&self, message_id: i64) -> Result<Value> {
        // 调用 delete_msg action，传入消息 ID
        self.client.call("delete_msg", json!({
            "message_id": message_id,
        })).await
    }

    /// 获取消息详情
    ///
    /// 对应 OneBot action: `get_msg`
    ///
    /// # 参数
    ///
    /// - `message_id`: 消息 ID
    ///
    /// # 返回值
    ///
    /// 成功返回消息详情 JSON（包含 sender、message、time 等字段）
    pub async fn get_message(&self, message_id: i64) -> Result<Value> {
        // 调用 get_msg action，传入消息 ID
        self.client.call("get_msg", json!({
            "message_id": message_id,
        })).await
    }

    /// 获取合并转发消息内容
    ///
    /// 对应 OneBot action: `get_forward_msg`
    ///
    /// # 参数
    ///
    /// - `id`: 合并转发消息的 ID
    ///
    /// # 返回值
    ///
    /// 成功返回转发消息内容 JSON（包含 messages 数组）
    pub async fn get_forward_message(&self, id: &str) -> Result<Value> {
        // 调用 get_forward_msg action，传入转发消息 ID
        self.client.call("get_forward_msg", json!({
            "id": id,
        })).await
    }

    /// 发送群合并转发消息
    ///
    /// 对应 OneBot action: `send_group_forward_msg`
    ///
    /// # 参数
    ///
    /// - `group_id`: 目标群号
    /// - `messages`: 合并转发节点消息段数组（每个元素为 Node 类型）
    ///
    /// # 返回值
    ///
    /// 成功返回包含 `message_id` 的 JSON 数据
    pub async fn send_group_forward_message(
        &self,
        group_id: i64,
        messages: Vec<MessageSegment>,
    ) -> Result<Value> {
        // 调用 send_group_forward_msg action，传入群号和转发节点
        self.client.call("send_group_forward_msg", json!({
            "group_id": group_id,
            "messages": messages,
        })).await
    }

    // ========================================================================
    // 精华消息
    // ========================================================================

    /// 设置精华消息
    ///
    /// 对应 OneBot action: `set_essence_msg`
    ///
    /// # 参数
    ///
    /// - `message_id`: 要设为精华的消息 ID
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn set_essence_message(&self, message_id: i64) -> Result<Value> {
        // 调用 set_essence_msg action，传入消息 ID
        self.client.call("set_essence_msg", json!({
            "message_id": message_id,
        })).await
    }

    /// 移除精华消息
    ///
    /// 对应 OneBot action: `delete_essence_msg`
    ///
    /// # 参数
    ///
    /// - `message_id`: 要移除精华的消息 ID
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn delete_essence_message(&self, message_id: i64) -> Result<Value> {
        // 调用 delete_essence_msg action，传入消息 ID
        self.client.call("delete_essence_msg", json!({
            "message_id": message_id,
        })).await
    }

    /// 获取群精华消息列表
    ///
    /// 对应 OneBot action: `get_essence_msg_list`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    ///
    /// # 返回值
    ///
    /// 成功返回精华消息列表 JSON 数组
    pub async fn get_essence_message_list(&self, group_id: i64) -> Result<Value> {
        // 调用 get_essence_msg_list action，传入群号
        self.client.call("get_essence_msg_list", json!({
            "group_id": group_id,
        })).await
    }

    // ========================================================================
    // 标记已读
    // ========================================================================

    /// 标记消息为已读
    ///
    /// 对应 OneBot action: `mark_msg_as_read`
    ///
    /// # 参数
    ///
    /// - `message_id`: 消息 ID
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn mark_message_as_read(&self, message_id: i64) -> Result<Value> {
        // 调用 mark_msg_as_read action，传入消息 ID
        self.client.call("mark_msg_as_read", json!({
            "message_id": message_id,
        })).await
    }

    /// 标记群消息为已读
    ///
    /// 对应 OneBot action: `mark_group_msg_as_read`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn mark_group_msg_as_read(&self, group_id: i64) -> Result<Value> {
        // 调用 mark_group_msg_as_read action，传入群号
        self.client.call("mark_group_msg_as_read", json!({
            "group_id": group_id,
        })).await
    }

    /// 标记私聊消息为已读
    ///
    /// 对应 OneBot action: `mark_private_msg_as_read`
    ///
    /// # 参数
    ///
    /// - `user_id`: 用户 QQ 号
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn mark_private_msg_as_read(&self, user_id: i64) -> Result<Value> {
        // 调用 mark_private_msg_as_read action，传入用户 ID
        self.client.call("mark_private_msg_as_read", json!({
            "user_id": user_id,
        })).await
    }

    /// 标记所有消息为已读
    ///
    /// 对应 OneBot action: `_mark_all_as_read`（NapCat 扩展 action，前缀下划线）
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn mark_all_as_read(&self) -> Result<Value> {
        // 调用 _mark_all_as_read action，无参数
        self.client.call("_mark_all_as_read", json!({})).await
    }

    // ========================================================================
    // 群信息查询
    // ========================================================================

    /// 获取群 @全体成员 剩余次数
    ///
    /// 对应 OneBot action: `get_group_at_all_remain`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    ///
    /// # 返回值
    ///
    /// 成功返回剩余次数信息 JSON
    pub async fn get_group_at_all_remain(&self, group_id: i64) -> Result<Value> {
        // 调用 get_group_at_all_remain action，传入群号
        self.client.call("get_group_at_all_remain", json!({
            "group_id": group_id,
        })).await
    }

    /// 获取群系统消息
    ///
    /// 对应 OneBot action: `get_group_system_msg`
    ///
    /// # 返回值
    ///
    /// 成功返回群系统消息列表 JSON（包含加群请求和被邀请信息）
    pub async fn get_group_system_msg(&self) -> Result<Value> {
        // 调用 get_group_system_msg action，无参数
        self.client.call("get_group_system_msg", json!({})).await
    }

    /// 获取群荣誉信息
    ///
    /// 对应 OneBot action: `get_group_honor_info`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    /// - `honor_type`: 荣誉类型（"talkative"=龙王、"performer"=群聊之火、
    ///   "legend"=群聊炽焰、"strong_newbie"=冒尖小春笋、"emotion"=快乐之源、"all"=全部）
    ///
    /// # 返回值
    ///
    /// 成功返回群荣誉信息 JSON
    pub async fn get_group_honor_info(
        &self,
        group_id: i64,
        honor_type: &str,
    ) -> Result<Value> {
        // 调用 get_group_honor_info action，传入群号和荣誉类型
        self.client.call("get_group_honor_info", json!({
            "group_id": group_id,
            "type": honor_type,
        })).await
    }

    /// 获取群消息历史记录
    ///
    /// 对应 OneBot action: `get_group_msg_history`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    /// - `message_seq`: 起始消息序号（可选，为 None 时从最新消息开始）
    /// - `count`: 获取条数（可选，默认由服务端决定）
    ///
    /// # 返回值
    ///
    /// 成功返回消息历史记录 JSON 数组
    pub async fn get_group_msg_history(
        &self,
        group_id: i64,
        message_seq: Option<i64>,
        count: Option<i32>,
    ) -> Result<Value> {
        // 构建请求参数 JSON 对象
        let mut params = json!({
            "group_id": group_id,
        });
        // 如果指定了起始消息序号，添加到参数中
        if let Some(seq) = message_seq {
            params["message_seq"] = json!(seq);
        }
        // 如果指定了获取条数，添加到参数中
        if let Some(c) = count {
            params["count"] = json!(c);
        }
        // 调用 get_group_msg_history action
        self.client.call("get_group_msg_history", params).await
    }

    /// 获取好友消息历史记录
    ///
    /// 对应 OneBot action: `get_friend_msg_history`
    ///
    /// # 参数
    ///
    /// - `user_id`: 好友 QQ 号
    /// - `message_seq`: 起始消息序号（可选，为 None 时从最新消息开始）
    /// - `count`: 获取条数（可选，默认由服务端决定）
    ///
    /// # 返回值
    ///
    /// 成功返回消息历史记录 JSON 数组
    pub async fn get_friend_msg_history(
        &self,
        user_id: i64,
        message_seq: Option<i64>,
        count: Option<i32>,
    ) -> Result<Value> {
        // 构建请求参数 JSON 对象
        let mut params = json!({
            "user_id": user_id,
        });
        // 如果指定了起始消息序号，添加到参数中
        if let Some(seq) = message_seq {
            params["message_seq"] = json!(seq);
        }
        // 如果指定了获取条数，添加到参数中
        if let Some(c) = count {
            params["count"] = json!(c);
        }
        // 调用 get_friend_msg_history action
        self.client.call("get_friend_msg_history", params).await
    }

    /// 获取最近联系人列表
    ///
    /// 对应 OneBot action: `get_recent_contact`
    ///
    /// # 参数
    ///
    /// - `count`: 获取条数（可选，默认由服务端决定）
    ///
    /// # 返回值
    ///
    /// 成功返回最近联系人列表 JSON 数组
    pub async fn get_recent_contact(&self, count: Option<i32>) -> Result<Value> {
        // 构建请求参数 JSON 对象
        let mut params = json!({});
        // 如果指定了获取条数，添加到参数中
        if let Some(c) = count {
            params["count"] = json!(c);
        }
        // 调用 get_recent_contact action
        self.client.call("get_recent_contact", params).await
    }

    // ========================================================================
    // 表情回应
    // ========================================================================

    /// 对消息设置表情回应（点赞）
    ///
    /// 对应 OneBot action: `set_msg_emoji_like`（NapCat 扩展）
    ///
    /// # 参数
    ///
    /// - `message_id`: 消息 ID
    /// - `emoji_id`: 表情 ID（例如 "128516"）
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn set_msg_emoji_like(
        &self,
        message_id: i64,
        emoji_id: &str,
    ) -> Result<Value> {
        // 调用 set_msg_emoji_like action，传入消息 ID 和表情 ID
        self.client.call("set_msg_emoji_like", json!({
            "message_id": message_id,
            "emoji_id": emoji_id,
        })).await
    }

    /// 获取消息的表情回应详情
    ///
    /// 对应 OneBot action: `fetch_emoji_like`（NapCat 扩展）
    ///
    /// # 参数
    ///
    /// - `message_id`: 消息 ID
    /// - `emoji_id`: 表情 ID
    /// - `emoji_type`: 表情类型
    ///
    /// # 返回值
    ///
    /// 成功返回表情回应详情 JSON
    pub async fn fetch_emoji_like(
        &self,
        message_id: i64,
        emoji_id: &str,
        emoji_type: &str,
    ) -> Result<Value> {
        // 调用 fetch_emoji_like action，传入消息 ID、表情 ID 和类型
        self.client.call("fetch_emoji_like", json!({
            "message_id": message_id,
            "emoji_id": emoji_id,
            "emoji_type": emoji_type,
        })).await
    }

    // ========================================================================
    // 戳一戳
    // ========================================================================

    /// 群内戳一戳
    ///
    /// 对应 OneBot action: `group_poke`（NapCat 扩展）
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    /// - `user_id`: 被戳的用户 QQ 号
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn group_poke(&self, group_id: i64, user_id: i64) -> Result<Value> {
        // 调用 group_poke action，传入群号和被戳用户 ID
        self.client.call("group_poke", json!({
            "group_id": group_id,
            "user_id": user_id,
        })).await
    }

    /// 好友戳一戳
    ///
    /// 对应 OneBot action: `friend_poke`（NapCat 扩展）
    ///
    /// # 参数
    ///
    /// - `user_id`: 被戳的好友 QQ 号
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn friend_poke(&self, user_id: i64) -> Result<Value> {
        // 调用 friend_poke action，传入被戳好友 ID
        self.client.call("friend_poke", json!({
            "user_id": user_id,
        })).await
    }

    /// 发送戳一戳（通用）
    ///
    /// 对应 OneBot action: `send_poke`（NapCat 扩展）
    ///
    /// 如果传入 `group_id` 则为群戳一戳，否则为好友戳一戳。
    ///
    /// # 参数
    ///
    /// - `user_id`: 被戳的用户 QQ 号
    /// - `group_id`: 群号（可选，传入则为群内戳一戳）
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn send_poke(
        &self,
        user_id: i64,
        group_id: Option<i64>,
    ) -> Result<Value> {
        // 构建请求参数 JSON 对象
        let mut params = json!({
            "user_id": user_id,
        });
        // 如果指定了群号，添加到参数中（表示群内戳一戳）
        if let Some(gid) = group_id {
            params["group_id"] = json!(gid);
        }
        // 调用 send_poke action
        self.client.call("send_poke", params).await
    }
}
