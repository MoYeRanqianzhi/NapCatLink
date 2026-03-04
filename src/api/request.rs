//! # 请求处理 API 模块
//!
//! 封装好友申请和群邀请等请求的处理 API，包括：
//! - 处理好友添加请求（set_friend_add_request）
//! - 处理群添加/邀请请求（set_group_add_request）

// 引入 serde_json 的 json! 宏和 Value 类型，用于构建 API 请求参数
use serde_json::{Value, json};

// 引入 API 客户端，用于发送 API 请求
use crate::api::client::ApiClient;

// 引入 SDK 的 Result 类型别名
use crate::error::Result;

/// 请求处理 API — 封装处理好友请求和群请求的操作
///
/// 通过持有 `ApiClient` 的克隆实例来发送请求。
/// `ApiClient` 内部使用 `Arc` 共享状态，Clone 成本极低。
#[derive(Clone)]
pub struct RequestApi {
    /// API 客户端实例（内部通过 Arc 共享状态，Clone 成本低）
    client: ApiClient,
}

impl RequestApi {
    /// 创建新的请求处理 API 实例
    ///
    /// # 参数
    ///
    /// - `client`: API 客户端实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的 `RequestApi` 实例
    pub fn new(client: ApiClient) -> Self {
        // 保存 API 客户端引用
        Self { client }
    }

    /// 处理好友添加请求
    ///
    /// 对应 OneBot action: `set_friend_add_request`
    ///
    /// # 参数
    ///
    /// - `flag`: 好友请求的 flag（从好友请求事件中获取）
    /// - `approve`: 是否同意（true = 同意，false = 拒绝）
    /// - `remark`: 好友备注（可选，仅在同意时有效）
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn handle_friend_request(
        &self,
        flag: &str,
        approve: bool,
        remark: Option<&str>,
    ) -> Result<Value> {
        // 构建基本请求参数：flag 标识和是否同意
        let mut params = json!({
            "flag": flag,
            "approve": approve,
        });
        // 如果指定了好友备注，添加到参数中
        if let Some(r) = remark {
            // 设置 remark 字段
            params["remark"] = json!(r);
        }
        // 调用 set_friend_add_request action 发送请求
        self.client.call("set_friend_add_request", params).await
    }

    /// 处理群添加/邀请请求
    ///
    /// 对应 OneBot action: `set_group_add_request`
    ///
    /// # 参数
    ///
    /// - `flag`: 群请求的 flag（从群请求事件中获取）
    /// - `sub_type`: 请求子类型（"add" = 加群请求，"invite" = 群邀请）
    /// - `approve`: 是否同意（true = 同意，false = 拒绝）
    /// - `reason`: 拒绝理由（可选，仅在拒绝时有效）
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn handle_group_request(
        &self,
        flag: &str,
        sub_type: &str,
        approve: bool,
        reason: Option<&str>,
    ) -> Result<Value> {
        // 构建基本请求参数：flag 标识、子类型和是否同意
        let mut params = json!({
            "flag": flag,
            "sub_type": sub_type,
            "approve": approve,
        });
        // 如果指定了拒绝理由，添加到参数中
        if let Some(r) = reason {
            // 设置 reason 字段
            params["reason"] = json!(r);
        }
        // 调用 set_group_add_request action 发送请求
        self.client.call("set_group_add_request", params).await
    }
}
