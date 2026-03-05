//! # 事件路由模块
//!
//! 负责解析 OneBot 11 协议的原始 JSON 事件，并根据事件类型将其路由到
//! 最具体的层级事件名称，通过事件总线进行广播分发。
//!
//! ## 事件层级结构
//!
//! OneBot 11 事件采用层级命名，路由器只发布最具体的层级：
//! - 有 message_type + sub_type: 发布 `message.{message_type}.{sub_type}`
//! - 仅有 message_type: 发布 `message.{message_type}`
//! - 两者都没有: 发布 `message`
//!
//! 订阅者通过 `recv_filter` 的前缀匹配机制接收所需层级的事件：
//! - `recv_filter("message")` 匹配所有消息事件
//! - `recv_filter("message.private")` 匹配所有私聊消息
//! - `recv_exact("message.private.friend")` 精确匹配好友私聊
//!
//! ## 特殊事件
//!
//! - `raw`：所有原始事件都会以此名称额外发布一次
//! - `unknown`：无法识别的 `post_type` 会以此名称发布

// 引入 serde_json::Value，用于处理 OneBot 11 协议的 JSON 事件数据
use serde_json::Value;

// 引入事件总线，用于发布路由后的事件
use super::bus::EventBus;

/// 事件路由器 — 解析 OneBot 11 事件 JSON 并分发到层级事件名
///
/// 路由器负责将原始的 OneBot 11 JSON 事件解析为结构化的层级事件名称，
/// 并通过事件总线进行广播。**每条事件只发布一个最具体的层级事件名**，
/// 避免订阅者通过 `recv_filter` 前缀匹配时重复接收同一事件。
///
/// # 路由规则
///
/// 路由器总是选择最具体的层级名称来发布事件：
///
/// | post_type    | 最具体层级名称                                 |
/// |-------------|-----------------------------------------------|
/// | message     | message.{message_type}.{sub_type}             |
/// | message_sent| message_sent.{message_type}.{sub_type}        |
/// | notice      | notice.{notice_type}.{sub_type}               |
/// | request     | request.{request_type}.{sub_type}             |
/// | meta_event  | meta_event.lifecycle.{sub_type} 或 meta_event.{meta_event_type} |
///
/// 如果某个字段缺失，则使用更粗粒度的层级名称。
pub struct EventRouter {
    /// 事件总线的共享引用，用于发布路由后的事件
    bus: std::sync::Arc<EventBus>,
}

impl EventRouter {
    /// 创建新的事件路由器实例
    ///
    /// # 参数
    /// - `bus`：事件总线的 `Arc` 共享引用，路由器将通过此总线发布解析后的事件
    ///
    /// # 返回值
    /// 返回一个新的 `EventRouter` 实例
    pub fn new(bus: std::sync::Arc<EventBus>) -> Self {
        // 将事件总线引用保存到路由器中
        Self { bus }
    }

    /// 路由一条原始 JSON 消息到相应的层级事件
    ///
    /// 解析 JSON 中的 `post_type` 字段，根据其值将事件分发到对应的路由处理方法。
    /// 无论什么类型的事件，都会额外发布一个 `"raw"` 事件。
    ///
    /// # 参数
    /// - `data`：原始的 OneBot 11 事件 JSON 数据
    pub fn route(&self, data: Value) {
        // 从 JSON 中提取 post_type 字段，确定事件的顶层类型
        let post_type = match data.get("post_type").and_then(|v| v.as_str()) {
            // 成功提取到 post_type，转换为 String 以便后续使用
            Some(pt) => pt.to_string(),
            // 缺少 post_type 字段，记录警告并中止路由
            None => {
                tracing::warn!("收到无效消息: 缺少 post_type");
                return;
            }
        };

        // 根据 post_type 的值分发到不同的路由处理方法
        match post_type.as_str() {
            // 元事件（心跳、生命周期等）
            "meta_event" => self.route_meta_event(&data),
            // 接收到的消息事件
            "message" => self.route_message("message", &data),
            // 自身发送的消息事件（与 message 使用相同逻辑，仅前缀不同）
            "message_sent" => self.route_message("message_sent", &data),
            // 通知事件（群文件上传、成员变动等）
            "notice" => self.route_notice(&data),
            // 请求事件（加好友、加群等）
            "request" => self.route_request(&data),
            // 未知的 post_type，发布为 unknown 事件
            _ => {
                tracing::warn!("未知的 post_type: {}", post_type);
                // 将未知类型的事件发布到 "unknown" 频道
                self.bus.publish("unknown", data.clone());
            }
        }

        // 无论事件类型如何，始终发布一个 "raw" 事件，方便全局监听
        self.bus.publish("raw", data);
    }

    /// 路由元事件（meta_event）
    ///
    /// 元事件包括心跳（heartbeat）和生命周期（lifecycle）事件。
    /// 只发布最具体的层级：
    /// - lifecycle 有 sub_type: `meta_event.lifecycle.{sub_type}`
    /// - 其他有 meta_event_type: `meta_event.{meta_event_type}`
    /// - 均无: `meta_event`
    ///
    /// # 参数
    /// - `data`：元事件的 JSON 数据
    fn route_meta_event(&self, data: &Value) {
        // 提取 meta_event_type 字段（如 "heartbeat"、"lifecycle"），缺失时使用空字符串
        let meta_type = data
            .get("meta_event_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // 对于 lifecycle 类型，尝试提取 sub_type 以发布最具体的层级
        if meta_type == "lifecycle" {
            if let Some(sub_type) = data.get("sub_type").and_then(|v| v.as_str()) {
                // 发布最具体层级: meta_event.lifecycle.{sub_type}
                self.bus.publish(
                    format!("meta_event.lifecycle.{}", sub_type),
                    data.clone(),
                );
                return;
            }
        }

        // 如果 meta_event_type 不为空，发布第二层级
        if !meta_type.is_empty() {
            // 发布 meta_event.{meta_event_type}，如 "meta_event.heartbeat"
            self.bus
                .publish(format!("meta_event.{}", meta_type), data.clone());
            return;
        }

        // 均无有效字段，发布顶层 meta_event
        self.bus.publish("meta_event", data.clone());
    }

    /// 路由消息事件（message / message_sent）
    ///
    /// 消息事件包括私聊消息和群聊消息。
    /// 只发布最具体的层级：
    /// - 有 message_type + sub_type: `{prefix}.{message_type}.{sub_type}`
    /// - 仅有 message_type: `{prefix}.{message_type}`
    /// - 均无: `{prefix}`
    ///
    /// # 参数
    /// - `prefix`：事件前缀（"message" 或 "message_sent"）
    /// - `data`：消息事件的 JSON 数据
    fn route_message(&self, prefix: &str, data: &Value) {
        // 提取 message_type 字段（如 "private"、"group"），缺失时使用空字符串
        let msg_type = data
            .get("message_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        // 提取 sub_type 字段（如 "normal"、"anonymous"、"friend"），缺失时使用空字符串
        let sub_type = data
            .get("sub_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // 根据字段存在情况选择最具体的层级发布
        if !msg_type.is_empty() && !sub_type.is_empty() {
            // 发布最具体层级: {prefix}.{message_type}.{sub_type}
            self.bus.publish(
                format!("{}.{}.{}", prefix, msg_type, sub_type),
                data.clone(),
            );
        } else if !msg_type.is_empty() {
            // 发布第二层级: {prefix}.{message_type}
            self.bus
                .publish(format!("{}.{}", prefix, msg_type), data.clone());
        } else {
            // 均无有效字段，发布顶层
            self.bus.publish(prefix, data.clone());
        }
    }

    /// 路由通知事件（notice）
    ///
    /// 通知事件包括群文件上传、成员增减、群禁言等。
    /// 只发布最具体的层级：
    /// - 有 notice_type + sub_type: `notice.{notice_type}.{sub_type}`
    /// - 仅有 notice_type: `notice.{notice_type}`
    /// - 均无: `notice`
    ///
    /// # 参数
    /// - `data`：通知事件的 JSON 数据
    fn route_notice(&self, data: &Value) {
        // 提取 notice_type 字段（如 "group_upload"、"group_increase"），缺失时使用空字符串
        let notice_type = data
            .get("notice_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        // 提取 sub_type 字段（如 "approve"、"invite"），缺失时使用空字符串
        let sub_type = data
            .get("sub_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // 根据字段存在情况选择最具体的层级发布
        if !notice_type.is_empty() && !sub_type.is_empty() {
            // 发布最具体层级: notice.{notice_type}.{sub_type}
            self.bus.publish(
                format!("notice.{}.{}", notice_type, sub_type),
                data.clone(),
            );
        } else if !notice_type.is_empty() {
            // 发布第二层级: notice.{notice_type}
            self.bus
                .publish(format!("notice.{}", notice_type), data.clone());
        } else {
            // 均无有效字段，发布顶层 notice
            self.bus.publish("notice", data.clone());
        }
    }

    /// 路由请求事件（request）
    ///
    /// 请求事件包括加好友请求和加群请求。
    /// 只发布最具体的层级：
    /// - 有 request_type + sub_type: `request.{request_type}.{sub_type}`
    /// - 仅有 request_type: `request.{request_type}`
    /// - 均无: `request`
    ///
    /// # 参数
    /// - `data`：请求事件的 JSON 数据
    fn route_request(&self, data: &Value) {
        // 提取 request_type 字段（如 "friend"、"group"），缺失时使用空字符串
        let req_type = data
            .get("request_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        // 提取 sub_type 字段（如 "add"、"invite"），缺失时使用空字符串
        let sub_type = data
            .get("sub_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // 根据字段存在情况选择最具体的层级发布
        if !req_type.is_empty() && !sub_type.is_empty() {
            // 发布最具体层级: request.{request_type}.{sub_type}
            self.bus.publish(
                format!("request.{}.{}", req_type, sub_type),
                data.clone(),
            );
        } else if !req_type.is_empty() {
            // 发布第二层级: request.{request_type}
            self.bus
                .publish(format!("request.{}", req_type), data.clone());
        } else {
            // 均无有效字段，发布顶层 request
            self.bus.publish("request", data.clone());
        }
    }
}
