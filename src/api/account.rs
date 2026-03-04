//! # 账号 API 模块
//!
//! 封装 OneBot 11 账号相关的 API 调用，包括：
//! - 获取登录号信息（get_login_info）
//! - 获取运行状态（get_status）
//! - 获取好友列表（get_friend_list）
//! - 获取群列表（get_group_list）
//! - 获取群信息（get_group_info）
//! - 获取群成员列表（get_group_member_list）
//! - 获取群成员信息（get_group_member_info）
//! - 获取陌生人信息（get_stranger_info）
//! - 获取版本信息（get_version_info）

// 引入 serde_json 的 json! 宏和 Value 类型，用于构建 API 请求参数
use serde_json::{Value, json};

// 引入 API 客户端，用于发送 API 请求
use super::client::ApiClient;

// 引入 SDK 的 Result 类型别名
use crate::error::Result;

/// 账号 API — 封装所有账号信息查询相关的 OneBot 11 API 调用
///
/// 通过持有 `ApiClient` 的克隆实例来发送请求。
/// `ApiClient` 内部使用 `Arc` 共享状态，Clone 成本极低。
#[derive(Clone)]
pub struct AccountApi {
    /// API 客户端实例（内部通过 Arc 共享状态，Clone 成本低）
    client: ApiClient,
}

impl AccountApi {
    /// 创建新的账号 API 实例
    ///
    /// # 参数
    ///
    /// - `client`: API 客户端实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的 `AccountApi` 实例
    pub fn new(client: ApiClient) -> Self {
        // 保存 API 客户端引用
        Self { client }
    }

    /// 获取登录号信息
    ///
    /// 对应 OneBot action: `get_login_info`
    ///
    /// # 返回值
    ///
    /// 成功返回包含 `user_id` 和 `nickname` 的 JSON 数据
    pub async fn get_login_info(&self) -> Result<Value> {
        // 调用 get_login_info action，无参数
        self.client.call("get_login_info", json!({})).await
    }

    /// 获取运行状态
    ///
    /// 对应 OneBot action: `get_status`
    ///
    /// # 返回值
    ///
    /// 成功返回运行状态 JSON（包含 online、good 等字段）
    pub async fn get_status(&self) -> Result<Value> {
        // 调用 get_status action，无参数
        self.client.call("get_status", json!({})).await
    }

    /// 获取好友列表
    ///
    /// 对应 OneBot action: `get_friend_list`
    ///
    /// # 返回值
    ///
    /// 成功返回好友列表 JSON 数组（每个元素包含 user_id、nickname、remark 等）
    pub async fn get_friend_list(&self) -> Result<Value> {
        // 调用 get_friend_list action，无参数
        self.client.call("get_friend_list", json!({})).await
    }

    /// 获取群列表
    ///
    /// 对应 OneBot action: `get_group_list`
    ///
    /// # 返回值
    ///
    /// 成功返回群列表 JSON 数组（每个元素包含 group_id、group_name、member_count 等）
    pub async fn get_group_list(&self) -> Result<Value> {
        // 调用 get_group_list action，无参数
        self.client.call("get_group_list", json!({})).await
    }

    /// 获取群信息
    ///
    /// 对应 OneBot action: `get_group_info`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    /// - `no_cache`: 是否不使用缓存（默认 false，使用缓存获取更快）
    ///
    /// # 返回值
    ///
    /// 成功返回群信息 JSON（包含 group_id、group_name、member_count 等）
    pub async fn get_group_info(
        &self,
        group_id: i64,
        no_cache: Option<bool>,
    ) -> Result<Value> {
        // 调用 get_group_info action，no_cache 默认 false
        self.client.call("get_group_info", json!({
            "group_id": group_id,
            "no_cache": no_cache.unwrap_or(false),
        })).await
    }

    /// 获取群成员列表
    ///
    /// 对应 OneBot action: `get_group_member_list`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    ///
    /// # 返回值
    ///
    /// 成功返回群成员列表 JSON 数组（每个元素包含 user_id、nickname、card、role 等）
    pub async fn get_group_member_list(&self, group_id: i64) -> Result<Value> {
        // 调用 get_group_member_list action，传入群号
        self.client.call("get_group_member_list", json!({
            "group_id": group_id,
        })).await
    }

    /// 获取群成员信息
    ///
    /// 对应 OneBot action: `get_group_member_info`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    /// - `user_id`: 成员 QQ 号
    /// - `no_cache`: 是否不使用缓存（默认 false）
    ///
    /// # 返回值
    ///
    /// 成功返回群成员信息 JSON（包含 user_id、nickname、card、role、join_time 等）
    pub async fn get_group_member_info(
        &self,
        group_id: i64,
        user_id: i64,
        no_cache: Option<bool>,
    ) -> Result<Value> {
        // 调用 get_group_member_info action，no_cache 默认 false
        self.client.call("get_group_member_info", json!({
            "group_id": group_id,
            "user_id": user_id,
            "no_cache": no_cache.unwrap_or(false),
        })).await
    }

    /// 获取陌生人信息
    ///
    /// 对应 OneBot action: `get_stranger_info`
    ///
    /// # 参数
    ///
    /// - `user_id`: 用户 QQ 号
    /// - `no_cache`: 是否不使用缓存（默认 false）
    ///
    /// # 返回值
    ///
    /// 成功返回陌生人信息 JSON（包含 user_id、nickname、sex、age 等）
    pub async fn get_stranger_info(
        &self,
        user_id: i64,
        no_cache: Option<bool>,
    ) -> Result<Value> {
        // 调用 get_stranger_info action，no_cache 默认 false
        self.client.call("get_stranger_info", json!({
            "user_id": user_id,
            "no_cache": no_cache.unwrap_or(false),
        })).await
    }

    /// 获取版本信息
    ///
    /// 对应 OneBot action: `get_version_info`
    ///
    /// # 返回值
    ///
    /// 成功返回版本信息 JSON（包含 app_name、app_version、protocol_version 等）
    pub async fn get_version_info(&self) -> Result<Value> {
        // 调用 get_version_info action，无参数
        self.client.call("get_version_info", json!({})).await
    }
}
