//! # 消息分发器模块
//!
//! 负责区分 WebSocket 接收到的消息类型（API 响应 vs 事件），
//! 并将其路由到相应的处理器：
//! - 带有 `echo` 字段的消息 -> ApiClient（API 响应）
//! - 不带 `echo` 字段的消息 -> EventRouter（OneBot 11 事件）
//! - `heartbeat_` 前缀的 echo -> 忽略（心跳响应由心跳服务内部处理）

// 引入 serde_json::Value，用于解析和处理 JSON 消息
use serde_json::Value;

// 引入事件路由器，用于将事件消息分发到事件总线
use crate::event::EventRouter;

// 引入 API 客户端，用于处理 API 响应的请求-响应配对
use super::client::ApiClient;

/// 消息分发器 — 区分 API 响应和事件，分别路由到对应的处理器
///
/// `Dispatcher` 是 WebSocket 消息的第一道处理关卡：
/// 1. 解析 JSON 消息
/// 2. 检查是否包含 `echo` 字段
/// 3. 有 echo -> 转发给 ApiClient 进行请求-响应配对
/// 4. 无 echo -> 转发给 EventRouter 进行事件路由
///
/// # 设计思路
///
/// OneBot 11 协议中，API 响应和事件推送共享同一条 WebSocket 连接。
/// 两者的区别在于 API 响应必定包含 `echo` 字段（由请求方指定），
/// 而事件推送不包含 `echo` 字段。
pub struct Dispatcher {
    /// API 客户端引用（处理 API 响应的请求-响应配对）
    api_client: ApiClient,
    /// 事件路由器引用（处理 OneBot 11 事件的层级路由）
    event_router: EventRouter,
}

impl Dispatcher {
    /// 创建新的消息分发器实例
    ///
    /// # 参数
    ///
    /// - `api_client`: API 客户端，用于处理带 echo 的 API 响应
    /// - `event_router`: 事件路由器，用于处理不带 echo 的事件消息
    ///
    /// # 返回值
    ///
    /// 返回一个新的 `Dispatcher` 实例
    pub fn new(api_client: ApiClient, event_router: EventRouter) -> Self {
        Self {
            // 保存 API 客户端引用
            api_client,
            // 保存事件路由器引用
            event_router,
        }
    }

    /// 分发一条 WebSocket 文本消息
    ///
    /// 解析 JSON 并根据 `echo` 字段的存在与否，
    /// 将消息路由到 ApiClient 或 EventRouter。
    ///
    /// # 参数
    ///
    /// - `message`: 从 WebSocket 接收到的原始 JSON 字符串
    ///
    /// # 处理流程
    ///
    /// 1. 解析 JSON：失败则记录错误日志并返回
    /// 2. 检查 `echo` 字段：
    ///    - 存在且以 `heartbeat_` 开头：静默忽略（心跳响应）
    ///    - 存在且为其他值：转发给 `ApiClient::handle_response`
    ///    - 不存在：转发给 `EventRouter::route`
    pub fn dispatch(&self, message: &str) {
        // 步骤 1：将原始 JSON 字符串解析为 serde_json::Value
        let data: Value = match serde_json::from_str(message) {
            // 解析成功，获取 JSON 值
            Ok(v) => v,
            // 解析失败，记录错误日志并返回（不 panic）
            Err(e) => {
                tracing::error!("消息 JSON 解析失败: {}", e);
                return;
            }
        };

        // 步骤 2：检查是否包含 echo 字段（字符串类型）
        if let Some(echo) = data.get("echo").and_then(|v| v.as_str()) {
            // 有 echo 字段 -> 这是一条 API 响应

            // 检查是否是心跳响应（echo 以 "heartbeat_" 开头）
            if echo.starts_with("heartbeat_") {
                // 心跳响应由心跳服务内部处理，分发器忽略即可
                return;
            }

            // 非心跳响应 -> 转发给 ApiClient 进行请求-响应配对
            self.api_client.handle_response(echo, &data);
        } else if data.get("retcode").is_some() {
            // 无 echo 但含有 retcode 字段 -> 这是一条孤立的 API 响应
            // 检查是否是服务端错误响应（如认证失败 1403）
            let retcode = data.get("retcode").and_then(|v| v.as_i64()).unwrap_or(0);
            let message = data.get("message").and_then(|v| v.as_str()).unwrap_or("");
            if retcode != 0 {
                // 非零 retcode 表示服务端返回了错误（1403 = token 验证失败等）
                tracing::error!(
                    "收到服务端错误响应: retcode={}, message={}",
                    retcode,
                    message
                );
            } else {
                // retcode=0 的孤立响应（如无 echo 的心跳响应），安全忽略
                tracing::debug!("忽略无 echo 的 API 响应");
            }
        } else {
            // 无 echo 且无 retcode -> 这是一条 OneBot 11 事件推送
            // 转发给 EventRouter 进行层级事件路由和广播
            self.event_router.route(data);
        }
    }
}
