//! # 事件系统模块
//!
//! 实现事件的接收、分发和处理机制，包括以下子模块：
//! - `bus`：事件总线，基于 broadcast channel 实现发布-订阅模式的事件分发
//! - `router`：事件路由器，解析 OneBot 11 事件 JSON 并路由到层级事件名称

// 事件总线子模块：基于 tokio broadcast channel 实现的发布-订阅事件分发中心
pub mod bus;

// 事件路由子模块：解析 OneBot 11 事件类型并将其分发到对应的层级事件名
pub mod router;

// 重导出核心类型，方便外部直接通过 `napcat_link::event::EventBus` 等路径访问
pub use bus::{EventBus, EventData, EventSubscription};
pub use router::EventRouter;
