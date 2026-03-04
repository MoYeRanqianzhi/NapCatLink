//! # 连接状态模块
//!
//! 定义 WebSocket 连接的状态机，包括：
//! - 断开状态（Disconnected）
//! - 连接中状态（Connecting）
//! - 已连接状态（Connected）
//! - 重连中状态（Reconnecting）

// 引入 serde 序列化/反序列化派生宏
use serde::{Deserialize, Serialize};

/// WebSocket 连接状态枚举
///
/// 描述 WebSocket 连接在整个生命周期中可能处于的四种状态。
/// 状态转换路径：
/// - Disconnected -> Connecting -> Connected
/// - Connected -> Reconnecting -> Connected（重连成功）
/// - Connected -> Reconnecting -> Disconnected（重连失败）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    /// 已断开连接 — 初始状态或重连失败后的最终状态
    Disconnected,
    /// 正在连接中 — 正在与 WebSocket 服务器建立连接
    Connecting,
    /// 已连接 — WebSocket 连接正常工作中
    Connected,
    /// 正在尝试重新连接 — 连接丢失后正在自动重连
    Reconnecting,
}

/// 为 ConnectionState 实现 Display trait
///
/// 将状态枚举转换为小写英文字符串，用于日志输出和状态展示。
impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 根据当前状态匹配对应的字符串表示
        match self {
            // 已断开连接
            Self::Disconnected => write!(f, "disconnected"),
            // 正在连接中
            Self::Connecting => write!(f, "connecting"),
            // 已连接
            Self::Connected => write!(f, "connected"),
            // 正在重连中
            Self::Reconnecting => write!(f, "reconnecting"),
        }
    }
}
