//! # 连接状态模块
//!
//! 定义 WebSocket 连接的状态机，包括：
//! - 断开状态（Disconnected）
//! - 连接中状态（Connecting）
//! - 已连接状态（Connected）
//! - 重连中状态（Reconnecting）
//! - 状态转换逻辑和状态变更通知
