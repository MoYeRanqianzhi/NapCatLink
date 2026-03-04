//! # 连接管理模块
//!
//! 管理 WebSocket 连接的完整生命周期，包括以下子模块：
//! - `state`：连接状态机（断开、连接中、已连接、重连中）
//! - `heartbeat`：心跳检测机制，定时发送心跳并检测连接存活
//! - `reconnect`：自动重连策略（指数退避、最大重试次数等）
//! - `actor`：连接 Actor，管理 WebSocket 读写任务和消息分发

// 连接状态子模块：定义连接状态枚举和状态转换逻辑
pub mod state;

// 心跳检测子模块：实现心跳发送、超时检测、连接存活判断
pub mod heartbeat;

// 重连策略子模块：实现指数退避重连算法和重连参数配置
pub mod reconnect;

// 连接 Actor 子模块：核心连接管理器，协调 WebSocket 读写和消息路由
pub mod actor;

// 重新导出常用类型，方便外部直接使用
// ConnectionState — 连接状态枚举
pub use state::ConnectionState;
// ConnectionHandle — 连接控制句柄
pub use actor::ConnectionHandle;
// ConnectionNotification — Actor 发出的通知
pub use actor::ConnectionNotification;
// build_websocket_url — URL 构建辅助函数
pub use actor::build_websocket_url;
