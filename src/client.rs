//! # 客户端模块
//!
//! 提供面向用户的高层 NapCatLink 客户端 API，整合以下功能：
//! - WebSocket 连接管理（自动重连、心跳）
//! - 事件监听和处理
//! - OneBot 11 标准 API 调用
//! - NapCat 扩展 API 调用
//!
//! 这是用户与 NapCatLink SDK 交互的主要入口点。
