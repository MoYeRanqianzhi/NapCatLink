//! # 事件类型模块
//!
//! 定义 OneBot 11 协议中所有事件类型的数据结构，分为以下子模块：
//! - `shared`：事件公共字段和枚举定义（Sender、Anonymous、FileInfo、BotStatus）
//! - `message`：消息事件（私聊消息、群聊消息）
//! - `notice`：通知事件（群成员变动、撤回、戳一戳等）
//! - `request`：请求事件（好友申请、群邀请等）
//! - `meta`：元事件（生命周期、心跳等）
//!
//! ## 顶层事件枚举
//!
//! `OneBotEvent` 枚举使用自定义反序列化实现，通过检查 JSON 中的
//! `post_type`、`message_type`、`notice_type`、`request_type`、`meta_event_type`
//! 等判别字段来精确路由到正确的变体，避免 `#[serde(untagged)]` 因字段子集匹配
//! 导致的错误分派问题。

// 引入 serde 的序列化/反序列化 trait 和派生宏
use serde::{Deserialize, Deserializer, Serialize, Serializer};
// 引入 serde_json::Value 用于中间 JSON 处理
use serde_json::Value;

// 公共定义子模块：事件基础字段、公共枚举类型
pub mod shared;

// 消息事件子模块：私聊消息事件、群聊消息事件的数据结构
pub mod message;

// 通知事件子模块：群成员增减、消息撤回、戳一戳等通知事件
pub mod notice;

// 请求事件子模块：好友添加请求、群邀请请求等事件
pub mod request;

// 元事件子模块：生命周期事件、心跳事件等元信息事件
pub mod meta;

// 从消息事件子模块导入消息事件类型
use message::{GroupMessageEvent, PrivateMessageEvent};
// 从通知事件子模块导入所有通知事件类型
use notice::{
    FriendAddNotice, FriendRecallNotice, GroupAdminNotice, GroupDecreaseNotice,
    GroupGrayTipNotice, GroupIncreaseNotice, GroupRecallNotice, GroupUploadNotice, PokeNotice,
};
// 从请求事件子模块导入请求事件类型
use request::{FriendRequestEvent, GroupRequestEvent};
// 从元事件子模块导入元事件类型
use meta::{HeartbeatEvent, LifecycleEvent};

/// OneBot 11 事件总枚举
///
/// 表示 OneBot 11 协议中所有可能的事件类型。
/// 使用自定义反序列化实现，通过检查 JSON 中的判别字段精确路由：
///
/// 1. `post_type = "message"` → 检查 `message_type` 区分私聊/群聊
/// 2. `post_type = "notice"` → 检查 `notice_type` 区分各类通知
/// 3. `post_type = "request"` → 检查 `request_type` 区分好友/群请求
/// 4. `post_type = "meta_event"` → 检查 `meta_event_type` 区分生命周期/心跳
/// 5. 其他 → `Unknown` 兜底
///
/// ## 使用示例
///
/// ```rust,no_run
/// use napcat_link::types::event::OneBotEvent;
///
/// let json_str = r#"{"time":1700000000,"self_id":123,"post_type":"meta_event","meta_event_type":"heartbeat","status":{"online":true,"good":true},"interval":5000}"#;
/// let event: OneBotEvent = serde_json::from_str(json_str).unwrap();
/// ```
#[derive(Debug, Clone)]
pub enum OneBotEvent {
    /// 私聊消息事件（post_type = "message", message_type = "private"）
    PrivateMessage(PrivateMessageEvent),

    /// 群聊消息事件（post_type = "message", message_type = "group"）
    GroupMessage(GroupMessageEvent),

    /// 群消息撤回通知（post_type = "notice", notice_type = "group_recall"）
    GroupRecall(GroupRecallNotice),

    /// 好友消息撤回通知（post_type = "notice", notice_type = "friend_recall"）
    FriendRecall(FriendRecallNotice),

    /// 群文件上传通知（post_type = "notice", notice_type = "group_upload"）
    GroupUpload(GroupUploadNotice),

    /// 群管理员变动通知（post_type = "notice", notice_type = "group_admin"）
    GroupAdmin(GroupAdminNotice),

    /// 群成员减少通知（post_type = "notice", notice_type = "group_decrease"）
    GroupDecrease(GroupDecreaseNotice),

    /// 群成员增加通知（post_type = "notice", notice_type = "group_increase"）
    GroupIncrease(GroupIncreaseNotice),

    /// 好友添加通知（post_type = "notice", notice_type = "friend_add"）
    FriendAdd(FriendAddNotice),

    /// 戳一戳通知（post_type = "notice", notice_type = "notify", sub_type = "poke"）
    Poke(PokeNotice),

    /// 群灰色提示条通知（post_type = "notice", notice_type = "notify", sub_type = "gray_tip"）
    GroupGrayTip(GroupGrayTipNotice),

    /// 好友添加请求（post_type = "request", request_type = "friend"）
    FriendRequest(FriendRequestEvent),

    /// 群添加/邀请请求（post_type = "request", request_type = "group"）
    GroupRequest(GroupRequestEvent),

    /// 生命周期元事件（post_type = "meta_event", meta_event_type = "lifecycle"）
    Lifecycle(LifecycleEvent),

    /// 心跳元事件（post_type = "meta_event", meta_event_type = "heartbeat"）
    Heartbeat(HeartbeatEvent),

    /// 私聊消息发送事件（post_type = "message_sent", message_type = "private"）
    ///
    /// 当 bot 自身发送私聊消息时触发，数据结构与 PrivateMessageEvent 完全相同，
    /// 仅 post_type 不同（"message_sent" 而非 "message"）。
    PrivateMessageSent(PrivateMessageEvent),

    /// 群聊消息发送事件（post_type = "message_sent", message_type = "group"）
    ///
    /// 当 bot 自身发送群聊消息时触发，数据结构与 GroupMessageEvent 完全相同，
    /// 仅 post_type 不同（"message_sent" 而非 "message"）。
    GroupMessageSent(GroupMessageEvent),

    /// 未知事件（兜底变体，捕获所有无法匹配已知类型的事件 JSON）
    Unknown(Value),
}

/// 为 OneBotEvent 实现自定义序列化
///
/// 将每个变体序列化为其内部结构体对应的 JSON 格式。
/// Unknown 变体直接输出原始 JSON 值。
impl Serialize for OneBotEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 根据变体类型，委托给内部结构体的序列化实现
        match self {
            // 消息事件：委托给对应消息事件结构体序列化
            OneBotEvent::PrivateMessage(e) => e.serialize(serializer),
            OneBotEvent::GroupMessage(e) => e.serialize(serializer),
            // 通知事件：委托给对应通知事件结构体序列化
            OneBotEvent::GroupRecall(e) => e.serialize(serializer),
            OneBotEvent::FriendRecall(e) => e.serialize(serializer),
            OneBotEvent::GroupUpload(e) => e.serialize(serializer),
            OneBotEvent::GroupAdmin(e) => e.serialize(serializer),
            OneBotEvent::GroupDecrease(e) => e.serialize(serializer),
            OneBotEvent::GroupIncrease(e) => e.serialize(serializer),
            OneBotEvent::FriendAdd(e) => e.serialize(serializer),
            OneBotEvent::Poke(e) => e.serialize(serializer),
            OneBotEvent::GroupGrayTip(e) => e.serialize(serializer),
            // 请求事件：委托给对应请求事件结构体序列化
            OneBotEvent::FriendRequest(e) => e.serialize(serializer),
            OneBotEvent::GroupRequest(e) => e.serialize(serializer),
            // 元事件：委托给对应元事件结构体序列化
            OneBotEvent::Lifecycle(e) => e.serialize(serializer),
            OneBotEvent::Heartbeat(e) => e.serialize(serializer),
            // 消息发送事件（bot 自身发送）：委托给对应消息事件结构体序列化
            OneBotEvent::PrivateMessageSent(e) => e.serialize(serializer),
            OneBotEvent::GroupMessageSent(e) => e.serialize(serializer),
            // 未知事件：直接序列化原始 JSON 值
            OneBotEvent::Unknown(v) => v.serialize(serializer),
        }
    }
}

/// 为 OneBotEvent 实现自定义反序列化
///
/// 通过检查 JSON 中的判别字段（post_type、message_type、notice_type 等）
/// 精确路由到正确的变体，解决 serde untagged 因结构相似导致的错误匹配问题。
///
/// ## 路由逻辑
///
/// 1. 先将 JSON 解析为 serde_json::Value
/// 2. 读取 post_type 字段确定事件大类
/// 3. 根据事件大类读取对应的子类型字段进一步区分
/// 4. 使用 serde_json::from_value 反序列化为具体结构体
/// 5. 无法识别的事件保存为 Unknown
impl<'de> Deserialize<'de> for OneBotEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // 第一步：将输入反序列化为通用 JSON Value（不丢失任何数据）
        let value = Value::deserialize(deserializer)?;

        // 第二步：提取 post_type 字段的值（字符串），用于确定事件大类
        let post_type = value
            .get("post_type")                  // 尝试获取 post_type 字段
            .and_then(|v| v.as_str())          // 转换为字符串切片
            .unwrap_or("");                    // 如果不存在或非字符串，默认为空串

        // 第三步：根据 post_type 路由到对应的子类型反序列化逻辑
        match post_type {
            // ========== 消息事件 ==========
            "message" => {
                // 读取 message_type 字段区分私聊和群聊
                let message_type = value
                    .get("message_type")       // 获取 message_type 字段
                    .and_then(|v| v.as_str())  // 转换为字符串
                    .unwrap_or("");            // 默认为空串

                match message_type {
                    // 私聊消息：反序列化为 PrivateMessageEvent
                    "private" => {
                        let event = serde_json::from_value(value.clone())
                            .map_err(serde::de::Error::custom)?;
                        Ok(OneBotEvent::PrivateMessage(event))
                    }
                    // 群聊消息：反序列化为 GroupMessageEvent
                    "group" => {
                        let event = serde_json::from_value(value.clone())
                            .map_err(serde::de::Error::custom)?;
                        Ok(OneBotEvent::GroupMessage(event))
                    }
                    // 未知的 message_type：作为 Unknown 保留
                    _ => Ok(OneBotEvent::Unknown(value)),
                }
            }

            // ========== 消息发送事件（bot 自身发送的消息回调） ==========
            // NapCatQQ 在 bot 自身发送消息时会推送 post_type = "message_sent" 的事件，
            // 其数据结构与普通 message 事件完全相同，仅 post_type 字段不同。
            "message_sent" => {
                // 读取 message_type 字段区分私聊和群聊（与 message 事件逻辑一致）
                let message_type = value
                    .get("message_type")       // 获取 message_type 字段
                    .and_then(|v| v.as_str())  // 转换为字符串
                    .unwrap_or("");            // 默认为空串

                match message_type {
                    // bot 发送的私聊消息：反序列化为 PrivateMessageEvent，包装为 PrivateMessageSent
                    "private" => {
                        let event = serde_json::from_value(value.clone())
                            .map_err(serde::de::Error::custom)?;
                        Ok(OneBotEvent::PrivateMessageSent(event))
                    }
                    // bot 发送的群聊消息：反序列化为 GroupMessageEvent，包装为 GroupMessageSent
                    "group" => {
                        let event = serde_json::from_value(value.clone())
                            .map_err(serde::de::Error::custom)?;
                        Ok(OneBotEvent::GroupMessageSent(event))
                    }
                    // 未知的 message_type：作为 Unknown 保留
                    _ => Ok(OneBotEvent::Unknown(value)),
                }
            }

            // ========== 通知事件 ==========
            "notice" => {
                // 读取 notice_type 字段区分不同的通知子类型
                let notice_type = value
                    .get("notice_type")        // 获取 notice_type 字段
                    .and_then(|v| v.as_str())  // 转换为字符串
                    .unwrap_or("");            // 默认为空串

                match notice_type {
                    // 群消息撤回通知
                    "group_recall" => {
                        let event = serde_json::from_value(value.clone())
                            .map_err(serde::de::Error::custom)?;
                        Ok(OneBotEvent::GroupRecall(event))
                    }
                    // 好友消息撤回通知
                    "friend_recall" => {
                        let event = serde_json::from_value(value.clone())
                            .map_err(serde::de::Error::custom)?;
                        Ok(OneBotEvent::FriendRecall(event))
                    }
                    // 群文件上传通知
                    "group_upload" => {
                        let event = serde_json::from_value(value.clone())
                            .map_err(serde::de::Error::custom)?;
                        Ok(OneBotEvent::GroupUpload(event))
                    }
                    // 群管理员变动通知
                    "group_admin" => {
                        let event = serde_json::from_value(value.clone())
                            .map_err(serde::de::Error::custom)?;
                        Ok(OneBotEvent::GroupAdmin(event))
                    }
                    // 群成员减少通知
                    "group_decrease" => {
                        let event = serde_json::from_value(value.clone())
                            .map_err(serde::de::Error::custom)?;
                        Ok(OneBotEvent::GroupDecrease(event))
                    }
                    // 群成员增加通知
                    "group_increase" => {
                        let event = serde_json::from_value(value.clone())
                            .map_err(serde::de::Error::custom)?;
                        Ok(OneBotEvent::GroupIncrease(event))
                    }
                    // 好友添加通知
                    "friend_add" => {
                        let event = serde_json::from_value(value.clone())
                            .map_err(serde::de::Error::custom)?;
                        Ok(OneBotEvent::FriendAdd(event))
                    }
                    // 通用通知（戳一戳等，通过 sub_type 进一步区分）
                    "notify" => {
                        // 读取 sub_type 字段进一步区分通知子类型
                        let sub_type = value
                            .get("sub_type")       // 获取 sub_type 字段
                            .and_then(|v| v.as_str()) // 转换为字符串
                            .unwrap_or("");        // 默认为空串

                        match sub_type {
                            // 戳一戳通知
                            "poke" => {
                                let event = serde_json::from_value(value.clone())
                                    .map_err(serde::de::Error::custom)?;
                                Ok(OneBotEvent::Poke(event))
                            }
                            // 群灰色提示条通知（NapCat 扩展）
                            "gray_tip" => {
                                let event = serde_json::from_value(value.clone())
                                    .map_err(serde::de::Error::custom)?;
                                Ok(OneBotEvent::GroupGrayTip(event))
                            }
                            // 其他未知的 notify 子类型：作为 Unknown 保留
                            _ => Ok(OneBotEvent::Unknown(value)),
                        }
                    }
                    // 未知的 notice_type：作为 Unknown 保留
                    _ => Ok(OneBotEvent::Unknown(value)),
                }
            }

            // ========== 请求事件 ==========
            "request" => {
                // 读取 request_type 字段区分好友请求和群请求
                let request_type = value
                    .get("request_type")       // 获取 request_type 字段
                    .and_then(|v| v.as_str())  // 转换为字符串
                    .unwrap_or("");            // 默认为空串

                match request_type {
                    // 好友添加请求
                    "friend" => {
                        let event = serde_json::from_value(value.clone())
                            .map_err(serde::de::Error::custom)?;
                        Ok(OneBotEvent::FriendRequest(event))
                    }
                    // 群添加/邀请请求
                    "group" => {
                        let event = serde_json::from_value(value.clone())
                            .map_err(serde::de::Error::custom)?;
                        Ok(OneBotEvent::GroupRequest(event))
                    }
                    // 未知的 request_type：作为 Unknown 保留
                    _ => Ok(OneBotEvent::Unknown(value)),
                }
            }

            // ========== 元事件 ==========
            "meta_event" => {
                // 读取 meta_event_type 字段区分生命周期和心跳
                let meta_event_type = value
                    .get("meta_event_type")    // 获取 meta_event_type 字段
                    .and_then(|v| v.as_str())  // 转换为字符串
                    .unwrap_or("");            // 默认为空串

                match meta_event_type {
                    // 生命周期事件
                    "lifecycle" => {
                        let event = serde_json::from_value(value.clone())
                            .map_err(serde::de::Error::custom)?;
                        Ok(OneBotEvent::Lifecycle(event))
                    }
                    // 心跳事件
                    "heartbeat" => {
                        let event = serde_json::from_value(value.clone())
                            .map_err(serde::de::Error::custom)?;
                        Ok(OneBotEvent::Heartbeat(event))
                    }
                    // 未知的 meta_event_type：作为 Unknown 保留
                    _ => Ok(OneBotEvent::Unknown(value)),
                }
            }

            // ========== 未知事件 ==========
            // post_type 不匹配任何已知类型，作为 Unknown 兜底保留原始 JSON
            _ => Ok(OneBotEvent::Unknown(value)),
        }
    }
}
