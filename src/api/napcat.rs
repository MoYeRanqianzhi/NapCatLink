//! # NapCat 扩展 API 模块
//!
//! 封装 NapCat 框架特有的扩展 API 调用，这些 API 不属于 OneBot 11 标准，
//! 而是 NapCat 为增强功能而提供的额外接口，包括：
//! - Rkey 获取（get_rkey / get_rkey_server / nc_get_rkey）
//! - 好友扩展操作（set_friend_remark / delete_friend 等）
//! - 群扩展信息（set_group_remark / get_group_info_ex 等）
//! - 合并转发扩展（send_private_forward_msg / forward_friend_single_msg 等）
//! - 群公告管理（_send_group_notice / _get_group_notice / _del_group_notice）
//! - 在线状态（set_online_status / set_diy_online_status）
//! - Ark 分享（send_ark_share / send_group_ark_share / get_mini_app_ark）
//! - AI 语音（get_ai_characters / get_ai_record / send_group_ai_record）
//! - 其他杂项功能（群签到、自定义表情、内联键盘等）

// 引入 serde_json 的 json! 宏和 Value 类型，用于构建 API 请求参数
use serde_json::{Value, json};

// 引入 API 客户端，用于发送 API 请求
use crate::api::client::ApiClient;

// 引入 SDK 的 Result 类型别名
use crate::error::Result;

/// NapCat 扩展 API — 封装 NapCat 框架特有的扩展功能
///
/// 这些 API 不属于 OneBot 11 标准协议，而是 NapCat 框架为增强功能
/// 而额外提供的接口。使用这些 API 时需要确保服务端为 NapCat。
///
/// 通过持有 `ApiClient` 的克隆实例来发送请求。
/// `ApiClient` 内部使用 `Arc` 共享状态，Clone 成本极低。
#[derive(Clone)]
pub struct NapCatApi {
    /// API 客户端实例（内部通过 Arc 共享状态，Clone 成本低）
    client: ApiClient,
}

impl NapCatApi {
    /// 创建新的 NapCat 扩展 API 实例
    ///
    /// # 参数
    ///
    /// - `client`: API 客户端实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的 `NapCatApi` 实例
    pub fn new(client: ApiClient) -> Self {
        // 保存 API 客户端引用
        Self { client }
    }

    // ========================================================================
    // Rkey — 用于媒体资源 URL 签名
    // ========================================================================

    /// 获取 Rkey（用于媒体资源 URL 签名）
    ///
    /// 对应 OneBot action: `get_rkey`
    ///
    /// # 返回值
    ///
    /// 成功返回 Rkey 信息 JSON
    pub async fn get_rkey(&self) -> Result<Value> {
        // 调用 get_rkey action，无参数
        self.client.call("get_rkey", json!({})).await
    }

    /// 获取 Rkey（别名方法）
    ///
    /// 对应 OneBot action: `get_rkey`（与 `get_rkey` 相同的 action）
    ///
    /// 此方法为 TS 版 `getRkeyEx()` 的对应实现，
    /// 在 TS 版中 `getRkeyEx` 同样映射到 `get_rkey` action。
    ///
    /// # 返回值
    ///
    /// 成功返回 Rkey 信息 JSON
    pub async fn get_rkey_ex(&self) -> Result<Value> {
        // 调用 get_rkey action，无参数（与 get_rkey 方法使用相同的 action）
        self.client.call("get_rkey", json!({})).await
    }

    /// 从服务器获取 Rkey
    ///
    /// 对应 OneBot action: `get_rkey_server`
    ///
    /// # 返回值
    ///
    /// 成功返回服务端 Rkey 信息 JSON
    pub async fn get_rkey_server(&self) -> Result<Value> {
        // 调用 get_rkey_server action，无参数
        self.client.call("get_rkey_server", json!({})).await
    }

    /// 获取 NapCat Rkey（NapCat 特有的 Rkey 获取方式）
    ///
    /// 对应 OneBot action: `nc_get_rkey`
    ///
    /// # 返回值
    ///
    /// 成功返回 NapCat Rkey 信息 JSON
    pub async fn nc_get_rkey(&self) -> Result<Value> {
        // 调用 nc_get_rkey action，无参数
        self.client.call("nc_get_rkey", json!({})).await
    }

    // ========================================================================
    // 好友扩展
    // ========================================================================

    /// 设置好友备注
    ///
    /// 对应 OneBot action: `set_friend_remark`
    ///
    /// # 参数
    ///
    /// - `user_id`: 好友 QQ 号
    /// - `remark`: 备注名称
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn set_friend_remark(&self, user_id: i64, remark: &str) -> Result<Value> {
        // 调用 set_friend_remark action，传入用户 ID 和备注
        self.client.call("set_friend_remark", json!({"user_id": user_id, "remark": remark})).await
    }

    /// 删除好友
    ///
    /// 对应 OneBot action: `delete_friend`
    ///
    /// # 参数
    ///
    /// - `user_id`: 好友 QQ 号
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn delete_friend(&self, user_id: i64) -> Result<Value> {
        // 调用 delete_friend action，传入用户 ID
        self.client.call("delete_friend", json!({"user_id": user_id})).await
    }

    /// 获取单向好友列表
    ///
    /// 对应 OneBot action: `get_unidirectional_friend_list`
    ///
    /// 单向好友是指对方加了你但你没有加对方的好友关系。
    ///
    /// # 返回值
    ///
    /// 成功返回单向好友列表 JSON 数组
    pub async fn get_unidirectional_friend_list(&self) -> Result<Value> {
        // 调用 get_unidirectional_friend_list action，无参数
        self.client.call("get_unidirectional_friend_list", json!({})).await
    }

    // ========================================================================
    // 群扩展
    // ========================================================================

    /// 设置群备注
    ///
    /// 对应 OneBot action: `set_group_remark`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    /// - `remark`: 群备注
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn set_group_remark(&self, group_id: i64, remark: &str) -> Result<Value> {
        // TS 版使用 String(groupId) 将群号转为字符串发送
        // 调用 set_group_remark action，传入字符串类型的群号和备注
        self.client.call("set_group_remark", json!({"group_id": group_id.to_string(), "remark": remark})).await
    }

    /// 获取群扩展信息
    ///
    /// 对应 OneBot action: `get_group_info_ex`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    ///
    /// # 返回值
    ///
    /// 成功返回群扩展信息 JSON
    pub async fn get_group_info_ex(&self, group_id: i64) -> Result<Value> {
        // 调用 get_group_info_ex action，传入群号
        self.client.call("get_group_info_ex", json!({"group_id": group_id})).await
    }

    /// 获取群详细信息
    ///
    /// 对应 OneBot action: `get_group_detail_info`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    ///
    /// # 返回值
    ///
    /// 成功返回群详细信息 JSON
    pub async fn get_group_detail_info(&self, group_id: i64) -> Result<Value> {
        // 调用 get_group_detail_info action，传入群号
        self.client.call("get_group_detail_info", json!({"group_id": group_id})).await
    }

    /// 获取群被忽略的通知列表
    ///
    /// 对应 OneBot action: `get_group_ignored_notifies`
    ///
    /// 此 API 无需参数，服务端将返回所有被忽略的群通知。
    ///
    /// # 返回值
    ///
    /// 成功返回被忽略通知列表 JSON
    pub async fn get_group_ignored_notifies(&self) -> Result<Value> {
        // 调用 get_group_ignored_notifies action，无参数
        self.client.call("get_group_ignored_notifies", json!({})).await
    }

    /// 获取群禁言列表
    ///
    /// 对应 OneBot action: `get_group_shut_list`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    ///
    /// # 返回值
    ///
    /// 成功返回当前被禁言成员列表 JSON
    pub async fn get_group_shut_list(&self, group_id: i64) -> Result<Value> {
        // 调用 get_group_shut_list action，传入群号
        self.client.call("get_group_shut_list", json!({"group_id": group_id})).await
    }

    // ========================================================================
    // 合并转发扩展
    // ========================================================================

    /// 发送私聊合并转发消息
    ///
    /// 对应 OneBot action: `send_private_forward_msg`
    ///
    /// # 参数
    ///
    /// - `user_id`: 目标用户 QQ 号
    /// - `messages`: 合并转发节点消息 JSON 数组
    /// - `news`: 转发消息摘要新闻条目（可选，自定义转发卡片预览）
    /// - `prompt`: 转发消息提示文本（可选，如 "[聊天记录]"）
    /// - `summary`: 转发消息摘要文本（可选，如 "查看 3 条转发消息"）
    /// - `source`: 转发消息来源标签（可选，如 "聊天记录"）
    ///
    /// # 返回值
    ///
    /// 成功返回包含 `message_id` 的 JSON 数据
    pub async fn send_private_forward_msg(
        &self,
        user_id: i64,
        messages: Value,
        news: Option<Value>,
        prompt: Option<&str>,
        summary: Option<&str>,
        source: Option<&str>,
    ) -> Result<Value> {
        // 构建基本请求参数：用户 ID 和转发节点
        let mut params = json!({"user_id": user_id, "messages": messages});
        // 如果指定了新闻摘要条目，添加到参数中
        if let Some(n) = news {
            params["news"] = n;
        }
        // 如果指定了提示文本，添加到参数中
        if let Some(p) = prompt {
            params["prompt"] = json!(p);
        }
        // 如果指定了摘要文本，添加到参数中
        if let Some(s) = summary {
            params["summary"] = json!(s);
        }
        // 如果指定了来源标签，添加到参数中
        if let Some(src) = source {
            params["source"] = json!(src);
        }
        // 调用 send_private_forward_msg action 发送请求
        self.client.call("send_private_forward_msg", params).await
    }

    /// 转发单条好友消息
    ///
    /// 对应 OneBot action: `forward_friend_single_msg`
    ///
    /// 将指定消息转发给好友。
    ///
    /// # 参数
    ///
    /// - `user_id`: 目标好友 QQ 号
    /// - `message_id`: 要转发的消息 ID
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn forward_friend_single_msg(&self, user_id: i64, message_id: i64) -> Result<Value> {
        // 调用 forward_friend_single_msg action，传入用户 ID 和消息 ID
        self.client.call("forward_friend_single_msg", json!({"user_id": user_id, "message_id": message_id})).await
    }

    /// 转发单条群消息
    ///
    /// 对应 OneBot action: `forward_group_single_msg`
    ///
    /// 将指定消息转发到群。
    ///
    /// # 参数
    ///
    /// - `group_id`: 目标群号
    /// - `message_id`: 要转发的消息 ID
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn forward_group_single_msg(&self, group_id: i64, message_id: i64) -> Result<Value> {
        // 调用 forward_group_single_msg action，传入群号和消息 ID
        self.client.call("forward_group_single_msg", json!({"group_id": group_id, "message_id": message_id})).await
    }

    /// 发送合并转发消息（通用）
    ///
    /// 对应 OneBot action: `send_forward_msg`
    ///
    /// # 参数
    ///
    /// - `params`: 完整的转发参数 JSON（可包含 user_id 或 group_id、messages 等）
    ///
    /// # 返回值
    ///
    /// 成功返回包含 `message_id` 的 JSON 数据
    pub async fn send_forward_msg(&self, params: Value) -> Result<Value> {
        // 调用 send_forward_msg action，直接传递完整参数
        self.client.call("send_forward_msg", params).await
    }

    // ========================================================================
    // 群公告
    // ========================================================================

    /// 发送群公告
    ///
    /// 对应 OneBot action: `_send_group_notice`（注意前缀下划线）
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    /// - `content`: 公告内容
    /// - `image`: 公告图片 URL（可选）
    /// - `pinned`: 是否置顶（可选，0 = 不置顶，1 = 置顶）
    /// - `notice_type`: 公告类型（可选，对应 JSON 字段 "type"）
    /// - `confirm_required`: 是否需要确认（可选，0 = 不需要，1 = 需要）
    /// - `is_show_edit_card`: 是否显示编辑名片提示（可选，0 = 不显示，1 = 显示）
    /// - `tip_window_type`: 提示窗口类型（可选）
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn send_group_notice(
        &self,
        group_id: i64,
        content: &str,
        image: Option<&str>,
        pinned: Option<i32>,
        notice_type: Option<i32>,
        confirm_required: Option<i32>,
        is_show_edit_card: Option<i32>,
        tip_window_type: Option<i32>,
    ) -> Result<Value> {
        // 构建基本请求参数：群号和公告内容
        let mut params = json!({"group_id": group_id, "content": content});
        // 如果指定了图片 URL，添加到参数中
        if let Some(img) = image {
            // 设置 image 字段为公告图片 URL
            params["image"] = json!(img);
        }
        // 如果指定了置顶选项，添加到参数中
        if let Some(p) = pinned {
            params["pinned"] = json!(p);
        }
        // 如果指定了公告类型，添加到参数中（JSON 字段名为 "type"）
        if let Some(t) = notice_type {
            params["type"] = json!(t);
        }
        // 如果指定了是否需要确认，添加到参数中
        if let Some(c) = confirm_required {
            params["confirm_required"] = json!(c);
        }
        // 如果指定了是否显示编辑名片提示，添加到参数中
        if let Some(s) = is_show_edit_card {
            params["is_show_edit_card"] = json!(s);
        }
        // 如果指定了提示窗口类型，添加到参数中
        if let Some(tw) = tip_window_type {
            params["tip_window_type"] = json!(tw);
        }
        // 调用 _send_group_notice action 发送请求
        self.client.call("_send_group_notice", params).await
    }

    /// 获取群公告列表
    ///
    /// 对应 OneBot action: `_get_group_notice`（注意前缀下划线）
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    ///
    /// # 返回值
    ///
    /// 成功返回群公告列表 JSON 数组
    pub async fn get_group_notice(&self, group_id: i64) -> Result<Value> {
        // 调用 _get_group_notice action，传入群号
        self.client.call("_get_group_notice", json!({"group_id": group_id})).await
    }

    /// 删除群公告
    ///
    /// 对应 OneBot action: `_del_group_notice`（注意前缀下划线）
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    /// - `notice_id`: 公告 ID
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn del_group_notice(&self, group_id: i64, notice_id: &str) -> Result<Value> {
        // TS 版使用 +noticeId 将字符串转为数字，这里也做同样处理
        // 如果 notice_id 无法解析为数字，则原样发送字符串
        let notice_id_value = notice_id.parse::<i64>()
            .map(|n| json!(n))
            .unwrap_or_else(|_| json!(notice_id));
        // 调用 _del_group_notice action，传入群号和公告 ID（数字类型）
        self.client.call("_del_group_notice", json!({"group_id": group_id, "notice_id": notice_id_value})).await
    }

    // ========================================================================
    // 在线状态
    // ========================================================================

    /// 设置在线状态
    ///
    /// 对应 OneBot action: `set_online_status`
    ///
    /// # 参数
    ///
    /// - `status`: 在线状态码
    /// - `ext_status`: 扩展状态码
    /// - `battery_status`: 电池状态码
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn set_online_status(
        &self,
        status: i32,
        ext_status: i32,
        battery_status: i32,
    ) -> Result<Value> {
        // 调用 set_online_status action，传入各状态码
        self.client.call("set_online_status", json!({
            "status": status, "ext_status": ext_status, "battery_status": battery_status
        })).await
    }

    /// 设置自定义在线状态
    ///
    /// 对应 OneBot action: `set_diy_online_status`
    ///
    /// # 参数
    ///
    /// - `face_id`: 表情 ID
    /// - `wording`: 状态文字描述（TS 版默认为空格 ' '）
    /// - `face_type`: 表情类型（可选，默认为 1，与 TS 版 faceType = 1 一致）
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn set_diy_online_status(
        &self,
        face_id: i32,
        wording: &str,
        face_type: Option<i32>,
    ) -> Result<Value> {
        // 构建请求参数：表情 ID、文字描述和表情类型（默认值 1）
        let params = json!({
            "face_id": face_id,
            "wording": wording,
            "face_type": face_type.unwrap_or(1)
        });
        // 调用 set_diy_online_status action 发送请求
        self.client.call("set_diy_online_status", params).await
    }

    // ========================================================================
    // Ark 分享
    // ========================================================================

    /// 发送 Ark 分享消息
    ///
    /// 对应 OneBot action: `send_ark_share`
    ///
    /// # 参数
    ///
    /// - `params`: Ark 分享参数 JSON（包含目标和 Ark 消息内容）
    ///
    /// # 返回值
    ///
    /// 成功返回分享结果 JSON
    pub async fn send_ark_share(&self, params: Value) -> Result<Value> {
        // 调用 send_ark_share action，直接传递完整参数
        self.client.call("send_ark_share", params).await
    }

    /// 发送群 Ark 分享消息
    ///
    /// 对应 OneBot action: `send_group_ark_share`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    ///
    /// # 返回值
    ///
    /// 成功返回分享结果 JSON
    pub async fn send_group_ark_share(&self, group_id: i64) -> Result<Value> {
        // 调用 send_group_ark_share action，传入群号
        self.client.call("send_group_ark_share", json!({"group_id": group_id})).await
    }

    /// 获取小程序 Ark 消息
    ///
    /// 对应 OneBot action: `get_mini_app_ark`
    ///
    /// # 参数
    ///
    /// - `params`: 小程序参数 JSON
    ///
    /// # 返回值
    ///
    /// 成功返回小程序 Ark 消息 JSON
    pub async fn get_mini_app_ark(&self, params: Value) -> Result<Value> {
        // 调用 get_mini_app_ark action，直接传递完整参数
        self.client.call("get_mini_app_ark", params).await
    }

    // ========================================================================
    // AI 语音
    // ========================================================================

    /// 获取 AI 语音角色列表
    ///
    /// 对应 OneBot action: `get_ai_characters`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    /// - `chat_type`: 聊天类型标识（可选，默认为 1，与 TS 版 chatType = 1 一致）
    ///
    /// # 返回值
    ///
    /// 成功返回 AI 语音角色列表 JSON
    pub async fn get_ai_characters(&self, group_id: i64, chat_type: Option<i64>) -> Result<Value> {
        // 调用 get_ai_characters action，传入群号和聊天类型（默认值 1）
        self.client.call("get_ai_characters", json!({
            "group_id": group_id,
            "chat_type": chat_type.unwrap_or(1)
        })).await
    }

    /// 获取 AI 语音
    ///
    /// 对应 OneBot action: `get_ai_record`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    /// - `character`: AI 角色标识
    /// - `text`: 要转换为语音的文本
    ///
    /// # 返回值
    ///
    /// 成功返回 AI 语音数据 JSON
    pub async fn get_ai_record(
        &self,
        group_id: i64,
        character: &str,
        text: &str,
    ) -> Result<Value> {
        // 调用 get_ai_record action，传入群号、角色标识和文本内容
        self.client.call("get_ai_record", json!({
            "group_id": group_id,
            "character": character,
            "text": text
        })).await
    }

    /// 发送群 AI 语音
    ///
    /// 对应 OneBot action: `send_group_ai_record`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    /// - `character`: AI 角色标识
    /// - `text`: 要转换为语音并发送的文本
    ///
    /// # 返回值
    ///
    /// 成功返回包含 `message_id` 的 JSON 数据
    pub async fn send_group_ai_record(
        &self,
        group_id: i64,
        character: &str,
        text: &str,
    ) -> Result<Value> {
        // 调用 send_group_ai_record action，传入群号、角色标识和文本
        self.client.call("send_group_ai_record", json!({
            "group_id": group_id, "character": character, "text": text
        })).await
    }

    // ========================================================================
    // 其他杂项功能
    // ========================================================================

    /// 群签到/群打卡（set_group_sign）
    ///
    /// 对应 OneBot action: `set_group_sign`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn set_group_sign(&self, group_id: i64) -> Result<Value> {
        // 调用 set_group_sign action，传入群号
        self.client.call("set_group_sign", json!({"group_id": group_id})).await
    }

    /// 群签到/群打卡（send_group_sign，别名）
    ///
    /// 对应 OneBot action: `send_group_sign`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn send_group_sign(&self, group_id: i64) -> Result<Value> {
        // 调用 send_group_sign action，传入群号
        self.client.call("send_group_sign", json!({"group_id": group_id})).await
    }

    /// 获取自定义表情列表
    ///
    /// 对应 OneBot action: `fetch_custom_face`
    ///
    /// # 返回值
    ///
    /// 成功返回自定义表情列表 JSON 数组
    pub async fn fetch_custom_face(&self) -> Result<Value> {
        // 调用 fetch_custom_face action，无参数
        self.client.call("fetch_custom_face", json!({})).await
    }

    /// 获取消息的表情回应列表
    ///
    /// 对应 OneBot action: `get_emoji_likes`
    ///
    /// # 参数
    ///
    /// - `params`: 查询参数 JSON（包含 message_id 等字段）
    ///
    /// # 返回值
    ///
    /// 成功返回表情回应列表 JSON
    pub async fn get_emoji_likes(&self, params: Value) -> Result<Value> {
        // 调用 get_emoji_likes action，直接传递完整参数
        self.client.call("get_emoji_likes", params).await
    }

    /// 获取 ClientKey
    ///
    /// 对应 OneBot action: `get_clientkey`
    ///
    /// # 返回值
    ///
    /// 成功返回 ClientKey JSON
    pub async fn get_clientkey(&self) -> Result<Value> {
        // 调用 get_clientkey action，无参数
        self.client.call("get_clientkey", json!({})).await
    }

    /// 点击内联键盘按钮
    ///
    /// 对应 OneBot action: `click_inline_keyboard_button`
    ///
    /// TS 版会为缺失的字段设置默认值：
    /// - `button_id`: 默认 `""`
    /// - `callback_data`: 默认 `""`
    /// - `msg_seq`: 默认 `"10086"`
    ///
    /// # 参数
    ///
    /// - `params`: 按钮参数 JSON（包含 bot_appid、button_id 等字段）
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn click_inline_keyboard_button(&self, params: Value) -> Result<Value> {
        // 克隆参数以便修改，为缺失的字段填充默认值（与 TS 版一致）
        let mut p = params;
        // button_id 默认为空字符串
        if p.get("button_id").is_none() {
            p["button_id"] = json!("");
        }
        // callback_data 默认为空字符串
        if p.get("callback_data").is_none() {
            p["callback_data"] = json!("");
        }
        // msg_seq 默认为 "10086"
        if p.get("msg_seq").is_none() {
            p["msg_seq"] = json!("10086");
        }
        // 调用 click_inline_keyboard_button action
        self.client.call("click_inline_keyboard_button", p).await
    }
}
