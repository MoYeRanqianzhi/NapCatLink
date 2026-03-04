//! # 类型模块
//!
//! 定义 OneBot 11 协议中使用的所有数据结构，分为以下子模块：
//! - `message`：消息段（MessageSegment）类型定义
//! - `event`：事件类型定义（消息事件、通知事件、请求事件、元事件）
//! - `api`：API 请求参数和响应结构体定义

// 消息类型子模块：定义消息段（text、image、at 等）的数据结构
pub mod message;

// 事件类型子模块：定义 OneBot 11 协议中的各类事件数据结构
pub mod event;

// API 类型子模块：定义 API 请求参数和响应体的数据结构
pub mod api;
