//! # 错误模块
//!
//! 定义 NapCatLink 中所有自定义错误类型，包括：
//! - WebSocket 连接错误
//! - 协议解析错误
//! - API 调用错误
//! - 序列化/反序列化错误
//! - 超时错误
//!
//! 所有 SDK 错误统一使用 `NapLinkError` 枚举，
//! 通过 `thiserror` 宏自动生成 Display 和 Error 实现。

// 引入 thiserror 的 Error 派生宏，用于自动生成 std::error::Error trait 实现
use thiserror::Error;

/// NapCatLink SDK 统一错误类型
///
/// 覆盖连接错误、API 错误、配置错误等所有可能的失败场景。
/// 每个变体对应一种特定的错误情况，携带相关的上下文信息。
#[derive(Debug, Error)]
pub enum NapLinkError {
    /// WebSocket 连接失败
    ///
    /// 当无法建立 WebSocket 连接时产生，携带错误描述字符串。
    /// 例如：连接被拒绝、DNS 解析失败、TLS 握手失败等。
    #[error("连接错误: {0}")]
    Connection(String),

    /// API 调用超时
    ///
    /// 当 API 请求在指定时间内未收到响应时产生。
    /// 携带 API 方法名和超时时间，便于排查慢请求。
    #[error("API 调用 {method} 超时 ({timeout_ms}ms)")]
    ApiTimeout {
        /// API 方法名（例如 "send_msg"、"get_group_list" 等）
        method: String,
        /// 超时时间（毫秒），即等待响应的最大时长
        timeout_ms: u64,
    },

    /// API 调用返回错误
    ///
    /// 当 NapCatQQ 服务端返回非成功状态码时产生。
    /// 优先显示 `wording`（用户友好描述），若无则退回 `message`。
    #[error("{}", wording.as_deref().unwrap_or(message.as_str()))]
    Api {
        /// API 方法名（例如 "send_msg"、"set_group_ban" 等）
        method: String,
        /// 服务端返回的错误码（retcode 字段）
        retcode: i64,
        /// 服务端返回的原始错误消息
        message: String,
        /// 用户友好的错误描述（可选，由服务端提供）
        wording: Option<String>,
    },

    /// 达到最大重连次数
    ///
    /// 当自动重连机制耗尽所有重试机会后产生。
    /// 表示连接已不可恢复，需要用户介入。
    #[error("达到最大重连次数 ({attempts})")]
    MaxReconnectAttempts {
        /// 已尝试的重连次数
        attempts: u32,
    },

    /// 连接已关闭
    ///
    /// 当 WebSocket 连接被远端关闭时产生。
    /// 携带 WebSocket 关闭码和关闭原因字符串。
    #[error("连接已关闭: {reason} (code: {code})")]
    ConnectionClosed {
        /// WebSocket 关闭码（例如 1000 表示正常关闭，1006 表示异常关闭）
        code: u16,
        /// 关闭原因描述
        reason: String,
    },

    /// 无效配置
    ///
    /// 当用户提供的配置参数不合法时产生。
    /// 携带具体的字段名和错误原因，便于定位配置问题。
    #[error("无效的配置: {field} - {reason}")]
    InvalidConfig {
        /// 配置字段名（例如 "url"、"token"、"timeout_ms" 等）
        field: String,
        /// 错误原因描述（例如 "不能为空"、"必须为正整数" 等）
        reason: String,
    },

    /// WebSocket 协议错误
    ///
    /// 对底层 `tokio_tungstenite` WebSocket 错误的透明包装。
    /// 使用 `Box` 包装以减小枚举整体大小（tungstenite::Error 体积较大）。
    /// 使用 `#[from]` 实现从 tungstenite Error 到 NapLinkError 的自动转换。
    #[error("WebSocket 错误: {0}")]
    WebSocket(#[from] Box<tokio_tungstenite::tungstenite::Error>),

    /// JSON 序列化/反序列化错误
    ///
    /// 对 `serde_json::Error` 的透明包装。
    /// 当消息 JSON 解析失败或序列化失败时产生。
    #[error("JSON 错误: {0}")]
    Json(#[from] serde_json::Error),

    /// URL 解析错误
    ///
    /// 对 `url::ParseError` 的透明包装。
    /// 当 WebSocket 连接地址格式不正确时产生。
    #[error("URL 解析错误: {0}")]
    UrlParse(#[from] url::ParseError),
}

impl NapLinkError {
    /// 获取错误码字符串
    ///
    /// 每种错误类型对应一个唯一的错误码前缀，格式为 `E_XXX`。
    /// 可用于日志分析、监控告警和错误分类统计。
    ///
    /// # 返回值
    ///
    /// 返回静态字符串引用，表示该错误类型的分类码。
    pub fn code(&self) -> &'static str {
        // 根据错误变体匹配对应的错误码
        match self {
            // 连接相关错误
            Self::Connection(_) => "E_CONNECTION",
            // API 超时错误
            Self::ApiTimeout { .. } => "E_API_TIMEOUT",
            // API 调用失败
            Self::Api { .. } => "E_API_FAILED",
            // 达到最大重连次数
            Self::MaxReconnectAttempts { .. } => "E_MAX_RECONNECT",
            // 连接关闭错误
            Self::ConnectionClosed { .. } => "E_CONNECTION_CLOSED",
            // 配置无效错误
            Self::InvalidConfig { .. } => "E_INVALID_CONFIG",
            // 底层 WebSocket 协议错误
            Self::WebSocket(_) => "E_WEBSOCKET",
            // JSON 处理错误
            Self::Json(_) => "E_JSON",
            // URL 解析错误
            Self::UrlParse(_) => "E_URL_PARSE",
        }
    }
}

/// SDK Result 类型别名
///
/// 将标准 `Result` 的错误类型固定为 `NapLinkError`，
/// 简化库内部和用户代码中的类型签名。
///
/// # 用法
///
/// ```rust
/// use napcat_link::error::Result;
///
/// fn do_something() -> Result<String> {
///     Ok("success".to_string())
/// }
/// ```
pub type Result<T> = std::result::Result<T, NapLinkError>;
