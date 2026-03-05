//! # NapCatLink - 现代化的 NapCatQQ WebSocket Rust 客户端 SDK
//!
//! 本库提供与 NapCatQQ 框架的完整连接支持，基于 OneBot 11 协议，
//! 通过 WebSocket 实现双向通信。
//!
//! # 功能特性
//!
//! - 异步 WebSocket 连接管理（基于 Tokio）
//! - 自动重连与指数退避策略
//! - 心跳检测机制
//! - 完整的 OneBot 11 API 支持
//! - NapCat 扩展 API 支持
//! - 强类型事件系统
//! - 流式文件上传/下载
//!
//! # 快速开始
//! ```rust,no_run
//! use napcat_link::{NapLink, MessageSegment};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 创建客户端
//!     let client = NapLink::builder("ws://127.0.0.1:3001")
//!         .token("your_token")
//!         .build()?;
//!
//!     // 连接到服务器
//!     client.connect().await?;
//!
//!     // 发送消息
//!     client.api().message.send_group_message(
//!         123456789,
//!         vec![MessageSegment::text("Hello from NapCatLink!")],
//!     ).await?;
//!
//!     Ok(())
//! }
//! ```

// 配置模块：管理 WebSocket 连接参数、认证信息、重连策略等配置项
pub mod config;

// 错误模块：定义项目中所有自定义错误类型和错误处理逻辑
pub mod error;

// 类型模块：定义 OneBot 11 协议中的所有数据结构（消息、事件、API 请求/响应）
pub mod types;

// 连接模块：管理 WebSocket 连接的生命周期（建立、维护、重连、断开）
pub mod connection;

// 事件模块：事件总线和事件路由，负责接收和分发 OneBot 11 事件
pub mod event;

// API 模块：封装 OneBot 11 标准 API 和 NapCat 扩展 API 的调用接口
pub mod api;

// 客户端模块：提供面向用户的高层 API，整合连接、事件、API 调用等功能
pub mod client;

// 工具模块（内部使用）：日志初始化、通用辅助函数等内部工具集
mod util;

// ---- 核心类型 re-export ----
// 将最常用的类型直接导出到 crate 根，方便用户使用 `napcat_link::NapLink` 等简短路径

/// 主客户端 — 用户与 SDK 交互的主要入口
pub use client::NapLink;

/// 配置类型 — SDK 完整配置、构建器和日志级别枚举
pub use config::{NapLinkConfig, NapLinkConfigBuilder, LogLevel};

/// 错误类型 — SDK 统一错误枚举和 Result 类型别名
pub use error::{NapLinkError, Result};

/// 消息段 — OneBot 11 消息的基本构建单元
pub use types::message::MessageSegment;

/// 事件系统 — 事件总线、事件数据和事件订阅
pub use event::{EventBus, EventData, EventSubscription};

/// 连接状态 — WebSocket 连接状态枚举
pub use connection::ConnectionState;

/// API 聚合器 — 所有 API 模块的统一入口
pub use api::OneBotApi;

/// OneBot 事件枚举 — 所有 OneBot 11 事件的顶层枚举类型
pub use types::event::OneBotEvent;

/// 通知事件类型 — 群通知、好友通知等事件的具体结构体
pub use types::event::notice::{
    FriendAddNotice, FriendRecallNotice, GroupAdminNotice, GroupDecreaseNotice,
    GroupGrayTipNotice, GroupIncreaseNotice, GroupRecallNotice, GroupUploadNotice, PokeNotice,
};

/// 消息事件类型 — 私聊和群聊消息事件的具体结构体
pub use types::event::message::{GroupMessageEvent, PrivateMessageEvent};

/// 请求事件类型 — 好友请求和群请求事件的具体结构体
pub use types::event::request::{FriendRequestEvent, GroupRequestEvent};

/// 元事件类型 — 生命周期和心跳事件的具体结构体
pub use types::event::meta::{HeartbeatEvent, LifecycleEvent};
