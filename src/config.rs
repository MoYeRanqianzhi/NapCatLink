//! # 配置模块
//!
//! 管理 NapCatLink 客户端的所有配置项，包括：
//! - WebSocket 连接地址与端口
//! - 认证 Token
//! - 重连策略参数
//! - 心跳检测间隔
//! - 超时设置
//!
//! 提供 `NapLinkConfigBuilder` 构建器模式，支持链式调用构建配置。

// 引入 serde 的序列化/反序列化派生宏
use serde::{Deserialize, Serialize};

// 引入本库的错误类型
use crate::error::NapLinkError;

/// 日志级别枚举
///
/// 定义 SDK 内部日志的输出级别，从最详细到完全关闭。
/// 与 `tracing` 框架的日志级别对应。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    /// 调试级别：输出最详细的调试信息，通常仅在开发阶段使用
    Debug,
    /// 信息级别：输出一般性运行信息（默认级别）
    Info,
    /// 警告级别：输出潜在问题的警告信息
    Warn,
    /// 错误级别：仅输出错误信息
    Error,
    /// 关闭日志：不输出任何日志信息
    Off,
}

/// 心跳动作配置
///
/// 定义 WebSocket 心跳检测时发送的 OneBot API 请求。
/// 默认使用 `get_status` 作为心跳检测方法。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatAction {
    /// 心跳检测调用的 API 方法名（例如 "get_status"）
    pub action: String,
    /// 心跳检测调用的参数（JSON 值，通常为空对象 `{}`）
    pub params: serde_json::Value,
}

/// 为 HeartbeatAction 实现默认值
///
/// 默认心跳动作为调用 `get_status` API，不携带任何参数。
impl Default for HeartbeatAction {
    fn default() -> Self {
        Self {
            // 默认使用 get_status 作为心跳检测方法
            action: "get_status".to_string(),
            // 默认参数为空 JSON 对象
            params: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
}

/// WebSocket 连接配置
///
/// 包含建立和维护 WebSocket 连接所需的所有参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    /// WebSocket 服务器地址（例如 "ws://127.0.0.1:3001"）
    pub url: String,
    /// 认证 Token（可选），用于 WebSocket 连接的鉴权
    pub token: Option<String>,
    /// 连接超时时间（毫秒），默认 30000ms（30秒）
    pub timeout_ms: u64,
    /// 心跳 Ping 发送间隔（毫秒），默认 30000ms（30秒）
    pub ping_interval_ms: u64,
    /// 心跳检测时发送的 API 动作配置
    pub heartbeat_action: HeartbeatAction,
}

/// 指数退避策略配置
///
/// 控制重连间隔的递增行为，避免在服务端故障期间产生过多的重连请求。
/// 每次重连间隔 = min(initial_ms * multiplier^(attempt-1), max_ms)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackoffConfig {
    /// 初始退避时间（毫秒），即首次重连前等待的时间，默认 5000ms（5秒）
    pub initial_ms: u64,
    /// 最大退避时间（毫秒），退避时间的上限，默认 60000ms（60秒）
    pub max_ms: u64,
    /// 退避乘数因子，每次重连间隔在上一次基础上乘以此值，默认 2.0
    pub multiplier: f64,
}

/// 为 BackoffConfig 实现默认值
impl Default for BackoffConfig {
    fn default() -> Self {
        Self {
            // 初始退避时间 5 秒
            initial_ms: 5000,
            // 最大退避时间 60 秒
            max_ms: 60000,
            // 退避乘数 2.0（每次翻倍）
            multiplier: 2.0,
        }
    }
}

/// 重连策略配置
///
/// 控制连接断开后的自动重连行为，包括是否启用、最大重试次数和退避策略。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconnectConfig {
    /// 是否启用自动重连，默认 true
    pub enabled: bool,
    /// 最大重连尝试次数，默认 5 次；超过后停止重连并返回错误
    pub max_attempts: u32,
    /// 指数退避策略配置，控制重连间隔的递增行为
    pub backoff: BackoffConfig,
}

/// 为 ReconnectConfig 实现默认值
impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            // 默认启用自动重连
            enabled: true,
            // 默认最大重连 5 次
            max_attempts: 5,
            // 使用默认的退避策略
            backoff: BackoffConfig::default(),
        }
    }
}

/// 日志配置
///
/// 控制 SDK 内部日志的输出行为。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// 日志输出级别，默认 `Info`
    pub level: LogLevel,
}

/// 为 LoggingConfig 实现默认值
impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            // 默认日志级别为 Info
            level: LogLevel::Info,
        }
    }
}

/// API 调用配置
///
/// 控制 OneBot API 调用的超时和重试行为。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// API 调用超时时间（毫秒），默认 15000ms（15秒）
    pub timeout_ms: u64,
    /// API 调用失败时的重试次数，默认 2 次
    pub retries: u32,
}

/// 为 ApiConfig 实现默认值
impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            // 默认 API 超时 15 秒
            timeout_ms: 15000,
            // 默认重试 2 次
            retries: 2,
        }
    }
}

/// NapCatLink 主配置结构体
///
/// 聚合所有子配置模块，作为 SDK 的完整配置。
/// 推荐通过 `NapLinkConfigBuilder` 构建。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NapLinkConfig {
    /// WebSocket 连接配置
    pub connection: ConnectionConfig,
    /// 重连策略配置
    pub reconnect: ReconnectConfig,
    /// 日志配置
    pub logging: LoggingConfig,
    /// API 调用配置
    pub api: ApiConfig,
}

/// NapCatLink 配置构建器
///
/// 使用 Builder 模式逐步构建 `NapLinkConfig`，支持链式调用。
/// 仅 `url` 为必填字段，其余字段均有合理的默认值。
///
/// # 用法示例
///
/// ```rust
/// use napcat_link::config::NapLinkConfigBuilder;
///
/// let config = NapLinkConfigBuilder::new("ws://127.0.0.1:3001")
///     .token("my-secret-token")
///     .timeout_ms(10000)
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct NapLinkConfigBuilder {
    /// WebSocket 服务器地址（必填）
    url: String,
    /// 认证 Token（可选）
    token: Option<String>,
    /// 连接超时时间（毫秒）
    timeout_ms: u64,
    /// 心跳 Ping 发送间隔（毫秒）
    ping_interval_ms: u64,
    /// 心跳检测动作配置
    heartbeat_action: HeartbeatAction,
    /// 是否启用自动重连
    reconnect_enabled: bool,
    /// 最大重连尝试次数
    max_reconnect_attempts: u32,
    /// 指数退避策略配置
    backoff: BackoffConfig,
    /// 日志级别
    log_level: LogLevel,
    /// API 调用超时时间（毫秒）
    api_timeout_ms: u64,
    /// API 调用重试次数
    api_retries: u32,
}

impl NapLinkConfigBuilder {
    /// 创建新的配置构建器
    ///
    /// # 参数
    ///
    /// - `url`: WebSocket 服务器地址（例如 "ws://127.0.0.1:3001"）
    ///
    /// # 返回值
    ///
    /// 返回一个带有默认值的配置构建器实例。
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            // 设置 WebSocket 服务器地址
            url: url.into(),
            // Token 默认为 None（不使用认证）
            token: None,
            // 连接超时默认 30 秒
            timeout_ms: 30000,
            // 心跳间隔默认 30 秒
            ping_interval_ms: 30000,
            // 使用默认的心跳动作（get_status）
            heartbeat_action: HeartbeatAction::default(),
            // 默认启用自动重连
            reconnect_enabled: true,
            // 默认最大重连 5 次
            max_reconnect_attempts: 5,
            // 使用默认的退避策略
            backoff: BackoffConfig::default(),
            // 默认日志级别为 Info
            log_level: LogLevel::Info,
            // API 超时默认 15 秒
            api_timeout_ms: 15000,
            // API 重试默认 2 次
            api_retries: 2,
        }
    }

    /// 设置认证 Token
    ///
    /// # 参数
    ///
    /// - `token`: 用于 WebSocket 连接鉴权的 Token 字符串
    pub fn token(mut self, token: impl Into<String>) -> Self {
        // 将 Token 包装为 Some 并存储
        self.token = Some(token.into());
        // 返回自身以支持链式调用
        self
    }

    /// 设置连接超时时间
    ///
    /// # 参数
    ///
    /// - `ms`: 超时时间（毫秒）
    pub fn timeout_ms(mut self, ms: u64) -> Self {
        // 更新连接超时时间
        self.timeout_ms = ms;
        // 返回自身以支持链式调用
        self
    }

    /// 设置心跳 Ping 发送间隔
    ///
    /// # 参数
    ///
    /// - `ms`: 心跳间隔（毫秒）
    pub fn ping_interval_ms(mut self, ms: u64) -> Self {
        // 更新心跳间隔
        self.ping_interval_ms = ms;
        // 返回自身以支持链式调用
        self
    }

    /// 设置心跳检测动作
    ///
    /// # 参数
    ///
    /// - `action`: 心跳检测的 API 动作配置
    pub fn heartbeat_action(mut self, action: HeartbeatAction) -> Self {
        // 更新心跳动作配置
        self.heartbeat_action = action;
        // 返回自身以支持链式调用
        self
    }

    /// 设置是否启用自动重连
    ///
    /// # 参数
    ///
    /// - `enabled`: true 启用，false 禁用
    pub fn reconnect_enabled(mut self, enabled: bool) -> Self {
        // 更新重连启用状态
        self.reconnect_enabled = enabled;
        // 返回自身以支持链式调用
        self
    }

    /// 设置最大重连尝试次数
    ///
    /// # 参数
    ///
    /// - `attempts`: 最大重连次数
    pub fn max_reconnect_attempts(mut self, attempts: u32) -> Self {
        // 更新最大重连次数
        self.max_reconnect_attempts = attempts;
        // 返回自身以支持链式调用
        self
    }

    /// 设置指数退避策略
    ///
    /// # 参数
    ///
    /// - `backoff`: 退避策略配置
    pub fn backoff(mut self, backoff: BackoffConfig) -> Self {
        // 更新退避策略
        self.backoff = backoff;
        // 返回自身以支持链式调用
        self
    }

    /// 设置日志级别
    ///
    /// # 参数
    ///
    /// - `level`: 日志输出级别
    pub fn log_level(mut self, level: LogLevel) -> Self {
        // 更新日志级别
        self.log_level = level;
        // 返回自身以支持链式调用
        self
    }

    /// 设置 API 调用超时时间
    ///
    /// # 参数
    ///
    /// - `ms`: API 超时时间（毫秒）
    pub fn api_timeout_ms(mut self, ms: u64) -> Self {
        // 更新 API 超时时间
        self.api_timeout_ms = ms;
        // 返回自身以支持链式调用
        self
    }

    /// 设置 API 调用重试次数
    ///
    /// # 参数
    ///
    /// - `retries`: 重试次数
    pub fn api_retries(mut self, retries: u32) -> Self {
        // 更新 API 重试次数
        self.api_retries = retries;
        // 返回自身以支持链式调用
        self
    }

    /// 构建最终的配置对象
    ///
    /// 验证所有必填字段并组装 `NapLinkConfig`。
    ///
    /// # 返回值
    ///
    /// - `Ok(NapLinkConfig)`: 配置验证通过，返回完整配置
    /// - `Err(NapLinkError::InvalidConfig)`: 配置验证失败（例如 URL 为空）
    ///
    /// # 错误
    ///
    /// - 当 `url` 为空字符串时，返回 `InvalidConfig` 错误
    pub fn build(self) -> crate::error::Result<NapLinkConfig> {
        // 验证 URL 不能为空
        if self.url.trim().is_empty() {
            return Err(NapLinkError::InvalidConfig {
                field: "url".to_string(),
                reason: "WebSocket URL 不能为空".to_string(),
            });
        }

        // 组装并返回完整配置
        Ok(NapLinkConfig {
            // 连接配置：URL、Token、超时、心跳
            connection: ConnectionConfig {
                url: self.url,
                token: self.token,
                timeout_ms: self.timeout_ms,
                ping_interval_ms: self.ping_interval_ms,
                heartbeat_action: self.heartbeat_action,
            },
            // 重连配置：启用状态、最大次数、退避策略
            reconnect: ReconnectConfig {
                enabled: self.reconnect_enabled,
                max_attempts: self.max_reconnect_attempts,
                backoff: self.backoff,
            },
            // 日志配置
            logging: LoggingConfig {
                level: self.log_level,
            },
            // API 配置：超时和重试
            api: ApiConfig {
                timeout_ms: self.api_timeout_ms,
                retries: self.api_retries,
            },
        })
    }
}
