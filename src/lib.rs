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
