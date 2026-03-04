//! # 事件路由模块
//!
//! 负责解析 OneBot 11 协议的原始 JSON 事件，并根据事件类型将其路由到
//! 相应的层级事件名称，通过事件总线进行广播分发。
//!
//! ## 事件层级结构
//!
//! OneBot 11 事件采用层级命名，例如：
//! - `message` -> `message.group` -> `message.group.normal`
//! - `notice` -> `notice.group_upload`
//! - `meta_event` -> `meta_event.lifecycle` -> `meta_event.lifecycle.connect`
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
/// 并通过事件总线进行广播。每条事件会根据其类型产生多个层级的事件发布。
///
/// # 路由规则
///
/// | post_type    | 层级1         | 层级2                        | 层级3                                  |
/// |-------------|---------------|------------------------------|----------------------------------------|
/// | message     | message       | message.{message_type}       | message.{message_type}.{sub_type}      |
/// | message_sent| message_sent  | message_sent.{message_type}  | message_sent.{message_type}.{sub_type} |
/// | notice      | notice        | notice.{notice_type}         | notice.{notice_type}.{sub_type}        |
/// | request     | request       | request.{request_type}       | request.{request_type}.{sub_type}      |
/// | meta_event  | meta_event    | meta_event.{meta_event_type} | meta_event.lifecycle.{sub_type}        |
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
            "message" => self.route_message(&data),
            // 自身发送的消息事件
            "message_sent" => self.route_message_sent(&data),
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
    /// 发布层级：`meta_event` -> `meta_event.{meta_event_type}`
    /// 对于 lifecycle 类型，额外发布：`meta_event.lifecycle.{sub_type}`
    ///
    /// # 参数
    /// - `data`：元事件的 JSON 数据
    fn route_meta_event(&self, data: &Value) {
        // 提取 meta_event_type 字段（如 "heartbeat"、"lifecycle"），缺失时使用空字符串
        let meta_type = data
            .get("meta_event_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // 发布第一层级事件: meta_event（所有元事件都会触发）
        self.bus.publish("meta_event", data.clone());

        // 如果 meta_event_type 不为空，发布第二层级事件
        if !meta_type.is_empty() {
            // 发布 meta_event.{meta_event_type}，如 "meta_event.heartbeat"
            self.bus
                .publish(format!("meta_event.{}", meta_type), data.clone());
        }

        // 对于 lifecycle 类型的元事件，额外发布包含 sub_type 的第三层级
        if meta_type == "lifecycle" {
            // 尝试提取 sub_type 字段（如 "connect"、"enable"）
            if let Some(sub_type) = data.get("sub_type").and_then(|v| v.as_str()) {
                // 发布 meta_event.lifecycle.{sub_type}，如 "meta_event.lifecycle.connect"
                self.bus.publish(
                    format!("meta_event.lifecycle.{}", sub_type),
                    data.clone(),
                );
            }
        }
    }

    /// 路由消息事件（message）
    ///
    /// 消息事件包括私聊消息和群聊消息。
    /// 发布层级：`message` -> `message.{message_type}` -> `message.{message_type}.{sub_type}`
    ///
    /// # 参数
    /// - `data`：消息事件的 JSON 数据
    fn route_message(&self, data: &Value) {
        // 提取 message_type 字段（如 "private"、"group"），缺失时使用空字符串
        let msg_type = data
            .get("message_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        // 提取 sub_type 字段（如 "normal"、"anonymous"、"notice"），缺失时使用空字符串
        let sub_type = data
            .get("sub_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // 发布第一层级事件: message（所有消息事件都会触发）
        self.bus.publish("message", data.clone());

        // 如果 message_type 不为空，发布更细粒度的事件
        if !msg_type.is_empty() {
            // 发布第二层级事件: message.{message_type}，如 "message.group"
            self.bus
                .publish(format!("message.{}", msg_type), data.clone());
            // 如果 sub_type 也不为空，发布第三层级事件
            if !sub_type.is_empty() {
                // 发布 message.{message_type}.{sub_type}，如 "message.group.normal"
                self.bus.publish(
                    format!("message.{}.{}", msg_type, sub_type),
                    data.clone(),
                );
            }
        }
    }

    /// 路由消息发送事件（message_sent）
    ///
    /// 当机器人自身发送消息时触发。层级结构与 message 相同。
    /// 发布层级：`message_sent` -> `message_sent.{message_type}` -> `message_sent.{message_type}.{sub_type}`
    ///
    /// # 参数
    /// - `data`：消息发送事件的 JSON 数据
    fn route_message_sent(&self, data: &Value) {
        // 提取 message_type 字段（如 "private"、"group"），缺失时使用空字符串
        let msg_type = data
            .get("message_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        // 提取 sub_type 字段，缺失时使用空字符串
        let sub_type = data
            .get("sub_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // 发布第一层级事件: message_sent
        self.bus.publish("message_sent", data.clone());

        // 如果 message_type 不为空，发布更细粒度的事件
        if !msg_type.is_empty() {
            // 发布第二层级事件: message_sent.{message_type}
            self.bus
                .publish(format!("message_sent.{}", msg_type), data.clone());
            // 如果 sub_type 也不为空，发布第三层级事件
            if !sub_type.is_empty() {
                // 发布 message_sent.{message_type}.{sub_type}
                self.bus.publish(
                    format!("message_sent.{}.{}", msg_type, sub_type),
                    data.clone(),
                );
            }
        }
    }

    /// 路由通知事件（notice）
    ///
    /// 通知事件包括群文件上传、成员增减、群禁言等。
    /// 发布层级：`notice` -> `notice.{notice_type}` -> `notice.{notice_type}.{sub_type}`
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

        // 发布第一层级事件: notice（所有通知事件都会触发）
        self.bus.publish("notice", data.clone());

        // 如果 notice_type 不为空，发布更细粒度的事件
        if !notice_type.is_empty() {
            // 发布第二层级事件: notice.{notice_type}，如 "notice.group_increase"
            self.bus
                .publish(format!("notice.{}", notice_type), data.clone());
            // 如果 sub_type 也不为空，发布第三层级事件
            if !sub_type.is_empty() {
                // 发布 notice.{notice_type}.{sub_type}，如 "notice.group_increase.approve"
                self.bus.publish(
                    format!("notice.{}.{}", notice_type, sub_type),
                    data.clone(),
                );
            }
        }
    }

    /// 路由请求事件（request）
    ///
    /// 请求事件包括加好友请求和加群请求。
    /// 发布层级：`request` -> `request.{request_type}` -> `request.{request_type}.{sub_type}`
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

        // 发布第一层级事件: request（所有请求事件都会触发）
        self.bus.publish("request", data.clone());

        // 如果 request_type 不为空，发布更细粒度的事件
        if !req_type.is_empty() {
            // 发布第二层级事件: request.{request_type}，如 "request.friend"
            self.bus
                .publish(format!("request.{}", req_type), data.clone());
            // 如果 sub_type 也不为空，发布第三层级事件
            if !sub_type.is_empty() {
                // 发布 request.{request_type}.{sub_type}，如 "request.group.invite"
                self.bus.publish(
                    format!("request.{}.{}", req_type, sub_type),
                    data.clone(),
                );
            }
        }
    }
}
