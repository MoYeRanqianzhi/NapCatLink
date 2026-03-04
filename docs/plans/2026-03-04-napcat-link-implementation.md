# NapCatLink-rs Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a complete Rust library that replicates NapLink's full functionality for connecting to NapCatQQ via OneBot 11 WebSocket protocol.

**Architecture:** Actor model using Tokio — ConnectionActor manages WebSocket lifecycle in an independent task, ApiClient pairs requests/responses via oneshot channels, EventBus distributes events via broadcast channels. All components communicate through message passing, ensuring zero deadlocks and natural concurrency safety.

**Tech Stack:** Rust 2024 Edition, Tokio, tokio-tungstenite, serde/serde_json, tracing, thiserror, dashmap, futures-util, url

---

## Phase 1: Project Scaffolding & Core Types

### Task 1: Initialize Cargo project and directory structure

**Files:**
- Create: `NapCatLink/Cargo.toml`
- Create: `NapCatLink/src/lib.rs`

**Step 1: Create Cargo.toml**

```toml
[package]
name = "napcat-link"
version = "0.1.0-alpha.1"
edition = "2024"
description = "A modern Rust SDK for connecting to NapCatQQ via OneBot 11 WebSocket protocol"
license = "MIT"
repository = "https://github.com/example/napcat-link-rs"
keywords = ["qq", "bot", "onebot", "napcat", "websocket"]
categories = ["api-bindings", "network-programming"]

[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = { version = "0.26", features = ["native-tls"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
thiserror = "2"
url = "2"
futures-util = "0.3"
dashmap = "6"
uuid = { version = "1", features = ["v4"] }
sha2 = "0.10"
base64 = "0.22"
tokio-stream = "0.1"

[dev-dependencies]
tokio-test = "0.4"
```

**Step 2: Create stub lib.rs**

```rust
//! NapCatLink - 现代化的 NapCatQQ WebSocket Rust 客户端 SDK
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

pub mod config;
pub mod error;
pub mod types;
pub mod connection;
pub mod event;
pub mod api;
pub mod client;
mod util;
```

**Step 3: Create module stubs**

Create empty `mod.rs` files for all submodules:
- `NapCatLink/src/config.rs`
- `NapCatLink/src/error.rs`
- `NapCatLink/src/types/mod.rs`
- `NapCatLink/src/connection/mod.rs`
- `NapCatLink/src/event/mod.rs`
- `NapCatLink/src/api/mod.rs`
- `NapCatLink/src/client.rs`
- `NapCatLink/src/util/mod.rs`

**Step 4: Verify project compiles**

Run: `cd NapCatLink && cargo check`
Expected: Compilation succeeds (possibly with unused warnings)

**Step 5: Initialize git and commit**

```bash
cd NapCatLink
git init
git add -A
git commit -m "feat: initialize NapCatLink Rust project scaffolding"
git tag v0.1.0-alpha.1
```

---

### Task 2: Implement error types

**Files:**
- Create: `NapCatLink/src/error.rs`
- Create: `NapCatLink/tests/error_test.rs`

**Step 1: Write failing test**

```rust
// tests/error_test.rs
use napcat_link::error::*;

#[test]
fn test_connection_error_display() {
    let err = NapLinkError::Connection("WebSocket creation failed".into());
    assert!(err.to_string().contains("WebSocket creation failed"));
}

#[test]
fn test_api_timeout_error() {
    let err = NapLinkError::ApiTimeout {
        method: "send_msg".into(),
        timeout_ms: 30000,
    };
    assert!(err.to_string().contains("send_msg"));
    assert!(err.to_string().contains("30000"));
}

#[test]
fn test_api_error() {
    let err = NapLinkError::Api {
        method: "send_msg".into(),
        retcode: -1,
        message: "failed".into(),
        wording: Some("发送失败".into()),
    };
    assert!(err.to_string().contains("发送失败"));
}

#[test]
fn test_error_code() {
    let err = NapLinkError::Connection("test".into());
    assert_eq!(err.code(), "E_CONNECTION");

    let err = NapLinkError::MaxReconnectAttempts { attempts: 10 };
    assert_eq!(err.code(), "E_MAX_RECONNECT");
}
```

**Step 2: Run test to verify it fails**

Run: `cd NapCatLink && cargo test --test error_test`
Expected: FAIL — module/types not defined

**Step 3: Implement error types**

```rust
// src/error.rs
//! NapCatLink 错误类型层次
//!
//! 所有 SDK 错误统一使用 `NapLinkError` 枚举，
//! 通过 `thiserror` 宏自动生成 Display 和 Error 实现。

use thiserror::Error;

/// NapCatLink SDK 统一错误类型
///
/// 覆盖连接错误、API 错误、配置错误等所有可能的失败场景。
#[derive(Debug, Error)]
pub enum NapLinkError {
    /// WebSocket 连接失败
    #[error("连接错误: {0}")]
    Connection(String),

    /// API 调用超时
    #[error("API 调用 {method} 超时 ({timeout_ms}ms)")]
    ApiTimeout {
        /// API 方法名
        method: String,
        /// 超时时间（毫秒）
        timeout_ms: u64,
    },

    /// API 调用返回错误
    #[error("{}", wording.as_deref().unwrap_or(message.as_str()))]
    Api {
        /// API 方法名
        method: String,
        /// 错误码
        retcode: i64,
        /// 错误消息
        message: String,
        /// 用户友好的错误描述
        wording: Option<String>,
    },

    /// 达到最大重连次数
    #[error("达到最大重连次数 ({attempts})")]
    MaxReconnectAttempts {
        /// 已尝试的重连次数
        attempts: u32,
    },

    /// 连接已关闭
    #[error("连接已关闭: {reason} (code: {code})")]
    ConnectionClosed {
        /// WebSocket 关闭码
        code: u16,
        /// 关闭原因
        reason: String,
    },

    /// 无效配置
    #[error("无效的配置: {field} - {reason}")]
    InvalidConfig {
        /// 配置字段名
        field: String,
        /// 错误原因
        reason: String,
    },

    /// WebSocket 协议错误
    #[error("WebSocket 错误: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    /// JSON 序列化/反序列化错误
    #[error("JSON 错误: {0}")]
    Json(#[from] serde_json::Error),

    /// URL 解析错误
    #[error("URL 解析错误: {0}")]
    UrlParse(#[from] url::ParseError),
}

impl NapLinkError {
    /// 获取错误码字符串
    ///
    /// 返回与 TypeScript 版本对应的错误码，便于跨语言错误识别。
    pub fn code(&self) -> &'static str {
        match self {
            Self::Connection(_) => "E_CONNECTION",
            Self::ApiTimeout { .. } => "E_API_TIMEOUT",
            Self::Api { .. } => "E_API_FAILED",
            Self::MaxReconnectAttempts { .. } => "E_MAX_RECONNECT",
            Self::ConnectionClosed { .. } => "E_CONNECTION_CLOSED",
            Self::InvalidConfig { .. } => "E_INVALID_CONFIG",
            Self::WebSocket(_) => "E_WEBSOCKET",
            Self::Json(_) => "E_JSON",
            Self::UrlParse(_) => "E_URL_PARSE",
        }
    }
}

/// SDK Result 类型别名
pub type Result<T> = std::result::Result<T, NapLinkError>;
```

**Step 4: Run test to verify it passes**

Run: `cd NapCatLink && cargo test --test error_test`
Expected: PASS

**Step 5: Commit**

```bash
git add -A && git commit -m "feat: implement error type hierarchy with thiserror"
```

---

### Task 3: Implement configuration types

**Files:**
- Create: `NapCatLink/src/config.rs`
- Create: `NapCatLink/tests/config_test.rs`

**Step 1: Write failing test**

```rust
// tests/config_test.rs
use napcat_link::config::*;

#[test]
fn test_builder_minimal() {
    let config = NapLinkConfig::builder("ws://127.0.0.1:3001")
        .build()
        .unwrap();
    assert_eq!(config.connection.url, "ws://127.0.0.1:3001");
    assert!(config.reconnect.enabled);
    assert_eq!(config.reconnect.max_attempts, 5);
    assert_eq!(config.api.timeout_ms, 15000);
    assert_eq!(config.api.retries, 2);
}

#[test]
fn test_builder_full() {
    let config = NapLinkConfig::builder("ws://127.0.0.1:3001")
        .token("my_token")
        .connection_timeout_ms(5000)
        .ping_interval_ms(10000)
        .reconnect_enabled(false)
        .reconnect_max_attempts(3)
        .backoff_initial_ms(2000)
        .backoff_max_ms(30000)
        .backoff_multiplier(1.5)
        .log_level(LogLevel::Debug)
        .api_timeout_ms(20000)
        .api_retries(5)
        .build()
        .unwrap();

    assert_eq!(config.connection.token.as_deref(), Some("my_token"));
    assert_eq!(config.connection.timeout_ms, 5000);
    assert!(!config.reconnect.enabled);
    assert_eq!(config.reconnect.max_attempts, 3);
}

#[test]
fn test_builder_empty_url_fails() {
    let result = NapLinkConfig::builder("").build();
    assert!(result.is_err());
}
```

**Step 2: Run test to verify it fails**

Run: `cd NapCatLink && cargo test --test config_test`
Expected: FAIL

**Step 3: Implement config**

```rust
// src/config.rs
//! NapCatLink 配置类型
//!
//! 提供 Builder 模式构建配置，所有可选参数都有合理的默认值。
//! 用户只需提供 WebSocket URL 即可快速启动。

use crate::error::NapLinkError;

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogLevel {
    /// 调试级别 — 输出所有日志
    Debug,
    /// 信息级别 — 默认级别
    #[default]
    Info,
    /// 警告级别
    Warn,
    /// 错误级别
    Error,
    /// 关闭日志
    Off,
}

/// 自定义心跳动作配置
#[derive(Debug, Clone)]
pub struct HeartbeatAction {
    /// API action 名称（默认 "get_status"）
    pub action: String,
    /// 附加参数
    pub params: serde_json::Value,
}

impl Default for HeartbeatAction {
    fn default() -> Self {
        Self {
            action: "get_status".into(),
            params: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
}

/// 连接配置
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    /// WebSocket 服务器 URL
    pub url: String,
    /// 访问令牌（可选）
    pub token: Option<String>,
    /// 连接超时时间（毫秒，默认 30000）
    pub timeout_ms: u64,
    /// 心跳间隔（毫秒，默认 30000，0 表示禁用）
    pub ping_interval_ms: u64,
    /// 自定义心跳动作
    pub heartbeat_action: HeartbeatAction,
}

/// 指数退避配置
#[derive(Debug, Clone)]
pub struct BackoffConfig {
    /// 初始延迟（毫秒，默认 5000）
    pub initial_ms: u64,
    /// 最大延迟（毫秒，默认 60000）
    pub max_ms: u64,
    /// 退避倍数（默认 2.0）
    pub multiplier: f64,
}

/// 重连配置
#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    /// 是否启用自动重连（默认 true）
    pub enabled: bool,
    /// 最大重连次数（默认 5）
    pub max_attempts: u32,
    /// 指数退避配置
    pub backoff: BackoffConfig,
}

/// 日志配置
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// 日志级别
    pub level: LogLevel,
}

/// API 配置
#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// API 调用超时时间（毫秒，默认 15000）
    pub timeout_ms: u64,
    /// 失败重试次数（默认 2）
    pub retries: u32,
}

/// NapCatLink 完整配置
///
/// 使用 `NapLinkConfig::builder(url)` 创建，所有可选参数都有合理默认值。
#[derive(Debug, Clone)]
pub struct NapLinkConfig {
    /// 连接配置
    pub connection: ConnectionConfig,
    /// 重连配置
    pub reconnect: ReconnectConfig,
    /// 日志配置
    pub logging: LoggingConfig,
    /// API 配置
    pub api: ApiConfig,
}

impl NapLinkConfig {
    /// 创建配置构建器
    ///
    /// # 参数
    /// - `url`: WebSocket 服务器 URL（必填）
    pub fn builder(url: &str) -> NapLinkConfigBuilder {
        NapLinkConfigBuilder::new(url)
    }
}

/// 配置构建器
///
/// 使用 Builder 模式设置各项配置参数。
pub struct NapLinkConfigBuilder {
    url: String,
    token: Option<String>,
    connection_timeout_ms: u64,
    ping_interval_ms: u64,
    heartbeat_action: HeartbeatAction,
    reconnect_enabled: bool,
    reconnect_max_attempts: u32,
    backoff_initial_ms: u64,
    backoff_max_ms: u64,
    backoff_multiplier: f64,
    log_level: LogLevel,
    api_timeout_ms: u64,
    api_retries: u32,
}

impl NapLinkConfigBuilder {
    /// 创建新的构建器实例
    fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            token: None,
            connection_timeout_ms: 30000,
            ping_interval_ms: 30000,
            heartbeat_action: HeartbeatAction::default(),
            reconnect_enabled: true,
            reconnect_max_attempts: 5,
            backoff_initial_ms: 5000,
            backoff_max_ms: 60000,
            backoff_multiplier: 2.0,
            log_level: LogLevel::Info,
            api_timeout_ms: 15000,
            api_retries: 2,
        }
    }

    /// 设置访问令牌
    pub fn token(mut self, token: &str) -> Self {
        self.token = Some(token.to_string());
        self
    }

    /// 设置连接超时时间（毫秒）
    pub fn connection_timeout_ms(mut self, ms: u64) -> Self {
        self.connection_timeout_ms = ms;
        self
    }

    /// 设置心跳间隔（毫秒，0 表示禁用）
    pub fn ping_interval_ms(mut self, ms: u64) -> Self {
        self.ping_interval_ms = ms;
        self
    }

    /// 设置自定义心跳动作
    pub fn heartbeat_action(mut self, action: HeartbeatAction) -> Self {
        self.heartbeat_action = action;
        self
    }

    /// 设置是否启用自动重连
    pub fn reconnect_enabled(mut self, enabled: bool) -> Self {
        self.reconnect_enabled = enabled;
        self
    }

    /// 设置最大重连次数
    pub fn reconnect_max_attempts(mut self, attempts: u32) -> Self {
        self.reconnect_max_attempts = attempts;
        self
    }

    /// 设置退避初始延迟（毫秒）
    pub fn backoff_initial_ms(mut self, ms: u64) -> Self {
        self.backoff_initial_ms = ms;
        self
    }

    /// 设置退避最大延迟（毫秒）
    pub fn backoff_max_ms(mut self, ms: u64) -> Self {
        self.backoff_max_ms = ms;
        self
    }

    /// 设置退避倍数
    pub fn backoff_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    /// 设置日志级别
    pub fn log_level(mut self, level: LogLevel) -> Self {
        self.log_level = level;
        self
    }

    /// 设置 API 调用超时时间（毫秒）
    pub fn api_timeout_ms(mut self, ms: u64) -> Self {
        self.api_timeout_ms = ms;
        self
    }

    /// 设置 API 失败重试次数
    pub fn api_retries(mut self, retries: u32) -> Self {
        self.api_retries = retries;
        self
    }

    /// 构建配置
    ///
    /// # 错误
    /// 当 URL 为空时返回 `InvalidConfig` 错误。
    pub fn build(self) -> Result<NapLinkConfig, NapLinkError> {
        // 验证 URL 非空
        if self.url.trim().is_empty() {
            return Err(NapLinkError::InvalidConfig {
                field: "connection.url".into(),
                reason: "URL 不能为空".into(),
            });
        }

        Ok(NapLinkConfig {
            connection: ConnectionConfig {
                url: self.url,
                token: self.token,
                timeout_ms: self.connection_timeout_ms,
                ping_interval_ms: self.ping_interval_ms,
                heartbeat_action: self.heartbeat_action,
            },
            reconnect: ReconnectConfig {
                enabled: self.reconnect_enabled,
                max_attempts: self.reconnect_max_attempts,
                backoff: BackoffConfig {
                    initial_ms: self.backoff_initial_ms,
                    max_ms: self.backoff_max_ms,
                    multiplier: self.backoff_multiplier,
                },
            },
            logging: LoggingConfig {
                level: self.log_level,
            },
            api: ApiConfig {
                timeout_ms: self.api_timeout_ms,
                retries: self.api_retries,
            },
        })
    }
}
```

**Step 4: Run test**

Run: `cd NapCatLink && cargo test --test config_test`
Expected: PASS

**Step 5: Commit**

```bash
git add -A && git commit -m "feat: implement configuration types with builder pattern"
```

---

### Task 4: Implement OneBot message segment types

**Files:**
- Create: `NapCatLink/src/types/mod.rs`
- Create: `NapCatLink/src/types/message.rs`
- Create: `NapCatLink/tests/message_segment_test.rs`

**Step 1: Write failing test**

```rust
// tests/message_segment_test.rs
use napcat_link::types::message::*;

#[test]
fn test_text_segment_serialize() {
    let seg = MessageSegment::Text { text: "hello".into() };
    let json = serde_json::to_value(&seg).unwrap();
    assert_eq!(json["type"], "text");
    assert_eq!(json["data"]["text"], "hello");
}

#[test]
fn test_at_segment_serialize() {
    let seg = MessageSegment::At { qq: "12345".into(), name: None };
    let json = serde_json::to_value(&seg).unwrap();
    assert_eq!(json["type"], "at");
    assert_eq!(json["data"]["qq"], "12345");
}

#[test]
fn test_at_all_serialize() {
    let seg = MessageSegment::At { qq: "all".into(), name: None };
    let json = serde_json::to_value(&seg).unwrap();
    assert_eq!(json["data"]["qq"], "all");
}

#[test]
fn test_image_segment_serialize() {
    let seg = MessageSegment::Image {
        file: "http://example.com/img.png".into(),
        summary: Some("图片".into()),
        sub_type: None,
        url: None,
    };
    let json = serde_json::to_value(&seg).unwrap();
    assert_eq!(json["type"], "image");
    assert_eq!(json["data"]["file"], "http://example.com/img.png");
    assert_eq!(json["data"]["summary"], "图片");
}

#[test]
fn test_deserialize_text_segment() {
    let json = r#"{"type":"text","data":{"text":"hello"}}"#;
    let seg: MessageSegment = serde_json::from_str(json).unwrap();
    match seg {
        MessageSegment::Text { text } => assert_eq!(text, "hello"),
        _ => panic!("expected Text segment"),
    }
}

#[test]
fn test_deserialize_unknown_segment() {
    let json = r#"{"type":"unknown_type","data":{"foo":"bar"}}"#;
    let seg: MessageSegment = serde_json::from_str(json).unwrap();
    match seg {
        MessageSegment::Unknown { r#type, data } => {
            assert_eq!(r#type, "unknown_type");
        }
        _ => panic!("expected Unknown segment"),
    }
}

#[test]
fn test_message_segment_helpers() {
    let seg = MessageSegment::text("hello world");
    match seg {
        MessageSegment::Text { text } => assert_eq!(text, "hello world"),
        _ => panic!("expected Text"),
    }

    let seg = MessageSegment::at("12345");
    match seg {
        MessageSegment::At { qq, .. } => assert_eq!(qq, "12345"),
        _ => panic!("expected At"),
    }

    let seg = MessageSegment::at_all();
    match seg {
        MessageSegment::At { qq, .. } => assert_eq!(qq, "all"),
        _ => panic!("expected At all"),
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd NapCatLink && cargo test --test message_segment_test`
Expected: FAIL

**Step 3: Implement message segment types**

```rust
// src/types/message.rs
//! OneBot 11 消息段类型
//!
//! 消息段（MessageSegment）是 OneBot 协议中消息的基本组成单元。
//! 每条消息由一个或多个消息段组成，每个消息段包含 type 和 data 两个字段。
//!
//! 使用 Rust 枚举 + serde 的 tagged enum 实现类型安全的序列化/反序列化。

use serde::{Deserialize, Serialize};

/// OneBot 11 消息段
///
/// 表示消息中的一个组成部分，如文本、图片、@某人等。
/// 支持所有 OneBot 11 标准消息段类型和 NapCat 扩展类型。
///
/// # 序列化格式
/// ```json
/// {"type": "text", "data": {"text": "hello"}}
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum MessageSegment {
    /// 纯文本消息
    #[serde(rename = "text")]
    Text {
        /// 文本内容
        text: String,
    },

    /// @某人 / @全体成员
    #[serde(rename = "at")]
    At {
        /// QQ 号或 "all" 表示全体成员
        qq: String,
        /// 被@者的名称（可选，用于显示）
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },

    /// QQ 表情
    #[serde(rename = "face")]
    Face {
        /// 表情 ID
        id: String,
    },

    /// 回复消息（引用）
    #[serde(rename = "reply")]
    Reply {
        /// 被回复的消息 ID
        id: String,
    },

    /// 图片消息
    #[serde(rename = "image")]
    Image {
        /// 图片文件（路径、URL 或 Base64）
        file: String,
        /// 图片摘要描述
        #[serde(skip_serializing_if = "Option::is_none")]
        summary: Option<String>,
        /// 子类型
        #[serde(skip_serializing_if = "Option::is_none")]
        sub_type: Option<String>,
        /// 图片 URL（服务端返回）
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
    },

    /// 语音消息
    #[serde(rename = "record")]
    Record {
        /// 语音文件
        file: String,
        /// 语音 URL（服务端返回）
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
    },

    /// 视频消息
    #[serde(rename = "video")]
    Video {
        /// 视频文件
        file: String,
        /// 视频 URL（服务端返回）
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
    },

    /// 文件消息
    #[serde(rename = "file")]
    File {
        /// 文件标识
        file: String,
        /// 文件名
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        /// 文件 URL（服务端返回）
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
    },

    /// JSON 卡片消息
    #[serde(rename = "json")]
    Json {
        /// JSON 字符串内容
        data: String,
    },

    /// XML 消息
    #[serde(rename = "xml")]
    Xml {
        /// XML 字符串内容
        data: String,
    },

    /// Markdown 消息
    #[serde(rename = "markdown")]
    Markdown {
        /// Markdown 文本内容
        content: String,
    },

    /// 戳一戳
    #[serde(rename = "poke")]
    Poke {
        /// 类型
        #[serde(rename = "type")]
        poke_type: String,
        /// ID
        id: String,
    },

    /// 骰子
    #[serde(rename = "dice")]
    Dice {
        /// 结果值（可选）
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<String>,
    },

    /// 石头剪刀布
    #[serde(rename = "rps")]
    Rps {
        /// 结果值（可选）
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<String>,
    },

    /// 商城表情
    #[serde(rename = "mface")]
    MFace {
        /// 表情 ID
        #[serde(skip_serializing_if = "Option::is_none")]
        emoji_id: Option<String>,
        /// 表情包 ID
        #[serde(skip_serializing_if = "Option::is_none")]
        emoji_package_id: Option<String>,
        /// 显示名称
        #[serde(skip_serializing_if = "Option::is_none")]
        key: Option<String>,
        /// 摘要
        #[serde(skip_serializing_if = "Option::is_none")]
        summary: Option<String>,
    },

    /// 音乐分享
    #[serde(rename = "music")]
    Music {
        /// 音乐类型（qq/163/custom）
        #[serde(rename = "type")]
        music_type: String,
        /// 音乐 ID（非 custom 类型时使用）
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        /// 自定义音乐 URL
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        /// 音频 URL
        #[serde(skip_serializing_if = "Option::is_none")]
        audio: Option<String>,
        /// 标题
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        /// 内容描述
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
        /// 图片 URL
        #[serde(skip_serializing_if = "Option::is_none")]
        image: Option<String>,
    },

    /// 合并转发节点
    #[serde(rename = "node")]
    Node {
        /// 转发消息 ID（使用已有消息）
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        /// 自定义发送者 ID
        #[serde(skip_serializing_if = "Option::is_none")]
        user_id: Option<String>,
        /// 自定义发送者昵称
        #[serde(skip_serializing_if = "Option::is_none")]
        nickname: Option<String>,
        /// 自定义消息内容
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<Vec<MessageSegment>>,
    },

    /// 合并转发（上报用）
    #[serde(rename = "forward")]
    Forward {
        /// 转发消息 ID
        id: String,
    },

    /// 联系人分享
    #[serde(rename = "contact")]
    Contact {
        /// 类型（qq/group）
        #[serde(rename = "type")]
        contact_type: String,
        /// QQ 号或群号
        id: String,
    },

    /// 位置消息
    #[serde(rename = "location")]
    Location {
        /// 纬度
        lat: String,
        /// 经度
        lon: String,
        /// 标题
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        /// 内容描述
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
    },

    /// 小程序
    #[serde(rename = "miniapp")]
    MiniApp {
        /// 小程序数据
        #[serde(flatten)]
        data: serde_json::Value,
    },

    /// 未知消息段类型（兜底）
    ///
    /// 当收到未识别的消息段类型时，保留原始数据而不丢失。
    #[serde(untagged)]
    Unknown {
        /// 消息段类型名称
        r#type: String,
        /// 原始数据
        data: serde_json::Value,
    },
}

/// 消息段快捷构造方法
impl MessageSegment {
    /// 创建文本消息段
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }

    /// 创建 @某人 消息段
    pub fn at(qq: impl Into<String>) -> Self {
        Self::At { qq: qq.into(), name: None }
    }

    /// 创建 @全体成员 消息段
    pub fn at_all() -> Self {
        Self::At { qq: "all".into(), name: None }
    }

    /// 创建图片消息段
    pub fn image(file: impl Into<String>) -> Self {
        Self::Image { file: file.into(), summary: None, sub_type: None, url: None }
    }

    /// 创建回复消息段
    pub fn reply(id: impl Into<String>) -> Self {
        Self::Reply { id: id.into() }
    }

    /// 创建表情消息段
    pub fn face(id: impl Into<String>) -> Self {
        Self::Face { id: id.into() }
    }

    /// 创建语音消息段
    pub fn record(file: impl Into<String>) -> Self {
        Self::Record { file: file.into(), url: None }
    }

    /// 创建视频消息段
    pub fn video(file: impl Into<String>) -> Self {
        Self::Video { file: file.into(), url: None }
    }

    /// 创建 JSON 卡片消息段
    pub fn json(data: impl Into<String>) -> Self {
        Self::Json { data: data.into() }
    }

    /// 创建 Markdown 消息段
    pub fn markdown(content: impl Into<String>) -> Self {
        Self::Markdown { content: content.into() }
    }
}
```

**Step 4: Run test**

Run: `cd NapCatLink && cargo test --test message_segment_test`
Expected: PASS

**Step 5: Commit**

```bash
git add -A && git commit -m "feat: implement OneBot 11 message segment types with serde"
```

---

### Task 5: Implement OneBot event types

**Files:**
- Create: `NapCatLink/src/types/event/mod.rs`
- Create: `NapCatLink/src/types/event/shared.rs`
- Create: `NapCatLink/src/types/event/message.rs`
- Create: `NapCatLink/src/types/event/notice.rs`
- Create: `NapCatLink/src/types/event/request.rs`
- Create: `NapCatLink/src/types/event/meta.rs`
- Create: `NapCatLink/tests/event_types_test.rs`

**Step 1: Write failing test**

Test deserialization of all major event types from JSON (as server would send).

**Step 2: Implement all event types**

This task implements the complete type hierarchy:
- `OneBotEvent` — top-level enum dispatched by `post_type`
- `MessageEvent` (private/group), `NoticeEvent` (12+ subtypes), `RequestEvent`, `MetaEvent`
- `Sender`, `Anonymous`, `FileInfo`, `GroupHonorInfo` shared types

All types derive `Debug, Clone, Serialize, Deserialize`.

**Step 3: Run test, commit**

```bash
git add -A && git commit -m "feat: implement complete OneBot 11 event type hierarchy"
```

---

## Phase 2: Connection Layer

### Task 6: Implement connection state and URL builder

**Files:**
- Create: `NapCatLink/src/connection/state.rs`
- Create: `NapCatLink/src/connection/url.rs`
- Create: `NapCatLink/tests/connection_url_test.rs`

**Step 1-3:** Implement `ConnectionState` enum (Disconnected, Connecting, Connected, Reconnecting) and `build_websocket_url(config) -> String` function.

**Step 4: Commit**

```bash
git add -A && git commit -m "feat: implement connection state enum and URL builder"
```

---

### Task 7: Implement heartbeat service

**Files:**
- Create: `NapCatLink/src/connection/heartbeat.rs`
- Create: `NapCatLink/tests/heartbeat_test.rs`

Implement `HeartbeatService` as an async task:
- Sends ping at configurable interval via channel
- Tracks consecutive missed pongs (max 3)
- Notifies timeout via callback channel
- `record_pong()` resets miss counter
- `stop()` cancels the task

**Commit:**
```bash
git add -A && git commit -m "feat: implement heartbeat service with timeout detection"
```

---

### Task 8: Implement reconnect service

**Files:**
- Create: `NapCatLink/src/connection/reconnect.rs`
- Create: `NapCatLink/tests/reconnect_test.rs`

Implement `ReconnectService`:
- Exponential backoff: `min(initial * multiplier^attempt, max)`
- Tracks attempt count
- `schedule()` returns next delay
- `reset()` clears attempt count
- `has_remaining_attempts()` check

**Commit:**
```bash
git add -A && git commit -m "feat: implement reconnect service with exponential backoff"
```

---

### Task 9: Implement ConnectionActor

**Files:**
- Create: `NapCatLink/src/connection/actor.rs`
- Modify: `NapCatLink/src/connection/mod.rs`

The core of the library. ConnectionActor runs as an independent Tokio task:

**Commands (sent via mpsc channel):**
```rust
enum ConnectionCommand {
    Connect { reply: oneshot::Sender<Result<()>> },
    Disconnect,
    Send { payload: String },
}
```

**Notifications (sent via mpsc channel to owner):**
```rust
enum ConnectionNotification {
    StateChanged(ConnectionState),
    Message(String),
    ConnectionLost { attempts: u32 },
    ConnectionRestored { timestamp: u64 },
}
```

**Internal flow:**
1. Receive `Connect` command → build URL → connect via `tokio_tungstenite::connect_async`
2. Split WebSocket into read/write halves
3. Spawn read loop: forward messages as `ConnectionNotification::Message`
4. Handle heartbeat via HeartbeatService
5. On close: trigger reconnect via ReconnectService
6. `Send` command writes to WebSocket write half

**Commit:**
```bash
git add -A && git commit -m "feat: implement ConnectionActor with WebSocket lifecycle management"
```

---

## Phase 3: Event System

### Task 10: Implement EventBus

**Files:**
- Create: `NapCatLink/src/event/bus.rs`
- Create: `NapCatLink/tests/event_bus_test.rs`

`EventBus` wraps `tokio::sync::broadcast`:
- `publish(event_name: &str, data: serde_json::Value)` — broadcast to all subscribers
- `subscribe() -> EventSubscription` — returns a receiver that gets all events
- `EventSubscription` has `recv()` and filtering methods

**Commit:**
```bash
git add -A && git commit -m "feat: implement EventBus with broadcast channel"
```

---

### Task 11: Implement EventRouter

**Files:**
- Create: `NapCatLink/src/event/router.rs`
- Create: `NapCatLink/tests/event_router_test.rs`

Routes raw JSON messages to hierarchical event names:
- `message` → `message`, `message.{type}`, `message.{type}.{sub_type}`
- `message_sent` → `message_sent`, `message_sent.{type}`
- `notice` → `notice`, `notice.{type}`, `notice.{type}.{sub_type}`
- `request` → `request`, `request.{type}`, `request.{type}.{sub_type}`
- `meta_event` → `meta_event`, `meta_event.{type}`
- Always emits `raw` for every event

**Commit:**
```bash
git add -A && git commit -m "feat: implement EventRouter with hierarchical event routing"
```

---

## Phase 4: API Layer

### Task 12: Implement ApiClient

**Files:**
- Create: `NapCatLink/src/api/client.rs`
- Create: `NapCatLink/tests/api_client_test.rs`

Core API request/response pairing:
- `call<T>(method, params) -> Result<T>` — send request, await response via oneshot
- Request format: `{ action, params, echo }`
- Response matching: `DashMap<String, oneshot::Sender<Result<Value>>>`
- Timeout handling: `tokio::time::timeout`
- Retry logic: only retry `ApiTimeout` and `ApiError`
- Stream support: `call_stream()` returns `AsyncStream` of packets
- Periodic cleanup of stale requests

**Commit:**
```bash
git add -A && git commit -m "feat: implement ApiClient with request-response pairing and retry"
```

---

### Task 13: Implement dispatcher (message routing)

**Files:**
- Create: `NapCatLink/src/api/dispatcher.rs`

Dispatcher receives raw WebSocket messages and routes them:
- Has `echo` field → ApiClient response (skip heartbeat_ prefixed)
- No `echo` field → EventRouter

**Commit:**
```bash
git add -A && git commit -m "feat: implement message dispatcher"
```

---

### Task 14: Implement MessageApi

**Files:**
- Create: `NapCatLink/src/api/message.rs`
- Create: `NapCatLink/tests/api_message_test.rs`

All message-related API methods:
- `send_message()`, `send_private_message()`, `send_group_message()`
- `delete_message()`, `get_message()`, `get_forward_message()`
- `send_group_forward_message()`
- `set_essence_message()`, `delete_essence_message()`, `get_essence_message_list()`
- `mark_message_as_read()`, `mark_group_msg_as_read()`, `mark_private_msg_as_read()`, `mark_all_as_read()`
- `get_group_at_all_remain()`, `get_group_system_msg()`, `get_group_honor_info()`
- `get_group_msg_history()`, `get_friend_msg_history()`, `get_recent_contact()`
- `set_msg_emoji_like()`, `fetch_emoji_like()`
- `group_poke()`, `friend_poke()`, `send_poke()`

Each method is an async fn that calls `ApiClient::call()` with the correct action name and params.

**Commit:**
```bash
git add -A && git commit -m "feat: implement complete MessageApi (20+ methods)"
```

---

### Task 15: Implement GroupApi

**Files:**
- Create: `NapCatLink/src/api/group.rs`

Methods:
- `set_group_ban()`, `unset_group_ban()`, `set_group_whole_ban()`
- `set_group_kick()`, `set_group_leave()`, `set_group_card()`
- `set_group_name()`, `set_group_admin()`, `set_group_anonymous_ban()`
- `set_group_special_title()`, `send_like()`

**Commit:**
```bash
git add -A && git commit -m "feat: implement GroupApi (11 methods)"
```

---

### Task 16: Implement AccountApi

**Files:**
- Create: `NapCatLink/src/api/account.rs`

Methods:
- `get_login_info()`, `get_status()`, `get_friend_list()`, `get_group_list()`
- `get_group_info()`, `get_group_member_list()`, `get_group_member_info()`
- `get_stranger_info()`, `get_version_info()`

**Commit:**
```bash
git add -A && git commit -m "feat: implement AccountApi (9 methods)"
```

---

### Task 17: Implement MediaApi

**Files:**
- Create: `NapCatLink/src/api/media.rs`

Methods:
- `get_image()`, `get_record()`, `get_file()`
- `hydrate_media(message: &mut [MessageSegment])` — fill in download URLs

**Commit:**
```bash
git add -A && git commit -m "feat: implement MediaApi with hydrate_media support"
```

---

### Task 18: Implement FileApi

**Files:**
- Create: `NapCatLink/src/api/file.rs`

Methods:
- `upload_group_file()`, `upload_private_file()`
- `get_group_file_system_info()`, `get_group_root_files()`
- `get_group_files_by_folder()`, `get_group_file_url()`
- `delete_group_file()`, `create_group_file_folder()`, `delete_group_folder()`
- `download_file()`

**Commit:**
```bash
git add -A && git commit -m "feat: implement FileApi (10 methods)"
```

---

### Task 19: Implement StreamApi

**Files:**
- Create: `NapCatLink/src/api/stream.rs`

Methods:
- `upload_file_stream()` — chunked upload via stream-action
- `get_upload_stream_status()`
- `download_file_stream()` — returns `Pin<Box<dyn Stream<Item = StreamPacket>>>`
- `download_file_stream_to_file()` — download to disk
- `download_file_image_stream()`, `download_file_record_stream()`

Uses `ApiClient::call_stream()` for streaming responses.

**Commit:**
```bash
git add -A && git commit -m "feat: implement StreamApi with async stream upload/download"
```

---

### Task 20: Implement RequestApi, SystemApi, NapCatApi

**Files:**
- Create: `NapCatLink/src/api/request.rs`
- Create: `NapCatLink/src/api/system.rs`
- Create: `NapCatLink/src/api/napcat.rs`

**RequestApi (2 methods):**
- `handle_friend_request()`, `handle_group_request()`

**SystemApi (12+ methods):**
- `get_online_clients()`, `can_send_image()`, `can_send_record()`
- `get_cookies()`, `get_csrf_token()`, `get_credentials()`
- `set_input_status()`, `ocr_image()`, `translate_en2zh()`
- `check_url_safely()`, `handle_quick_operation()`

**NapCatApi (25+ methods):**
- Rkey: `get_rkey()`, `nc_get_rkey()`
- Friends: `set_friend_remark()`, `delete_friend()`, `get_unidirectional_friend_list()`
- Groups: `set_group_remark()`, `get_group_info_ex()`, `get_group_ignored_notifies()`
- Forwarding: `send_private_forward_msg()`, `forward_friend_single_msg()`, `forward_group_single_msg()`, `send_forward_msg()`
- Notices: `send_group_notice()`, `get_group_notice()`, `del_group_notice()`
- Status: `set_online_status()`, `set_diy_online_status()`
- AI: `get_ai_characters()`, `get_ai_record()`, `send_group_ai_record()`
- Others: `get_clientkey()`, `fetch_custom_face()`, `get_emoji_likes()`

**Commit:**
```bash
git add -A && git commit -m "feat: implement RequestApi, SystemApi, and NapCatApi"
```

---

### Task 21: Implement RawActionApi

**Files:**
- Create: `NapCatLink/src/api/raw.rs`

Provides access to all 165+ NapCat actions via:
```rust
/// 调用任意原始 action
pub async fn call(&self, action: &str, params: Value) -> Result<Value>
```

Also provides a `NAPCAT_ACTIONS` constant array listing all known action names.

**Commit:**
```bash
git add -A && git commit -m "feat: implement RawActionApi with 165+ action support"
```

---

## Phase 5: NapLink Client (Facade)

### Task 22: Implement NapLink client

**Files:**
- Create: `NapCatLink/src/client.rs`
- Create: `NapCatLink/tests/client_test.rs`

The main entry point. NapLink orchestrates all components:

```rust
pub struct NapLink {
    config: NapLinkConfig,
    // Connection actor handle
    conn_cmd_tx: mpsc::Sender<ConnectionCommand>,
    // Event bus for subscriptions
    event_bus: Arc<EventBus>,
    // API client
    api_client: Arc<ApiClient>,
    // All API modules
    api: OneBotApi,
    // Connection state
    state: Arc<AtomicU8>,
    // Background task handles
    _tasks: Vec<JoinHandle<()>>,
}
```

**Public API:**
- `NapLink::builder(url) -> NapLinkBuilder` (wraps config builder, adds connect logic)
- `connect() -> Result<()>` — connect to server
- `disconnect()` — graceful shutdown
- `state() -> ConnectionState`
- `is_connected() -> bool`
- `subscribe(filter) -> EventSubscription` — subscribe to events
- `on(event_name, callback)` — convenience event handler
- `api() -> &OneBotApi` — access all API methods
- `call_api(method, params) -> Result<Value>` — raw API call

**Internal wiring:**
1. Spawn ConnectionActor task
2. Spawn notification handler task (processes ConnectionNotification)
3. Dispatcher routes messages to ApiClient or EventRouter→EventBus

**Commit:**
```bash
git add -A && git commit -m "feat: implement NapLink client facade with full API access"
```

---

### Task 23: Implement OneBotApi aggregator

**Files:**
- Modify: `NapCatLink/src/api/mod.rs`

Aggregates all API modules into a single struct:

```rust
pub struct OneBotApi {
    pub message: MessageApi,
    pub group: GroupApi,
    pub account: AccountApi,
    pub media: MediaApi,
    pub file: FileApi,
    pub stream: StreamApi,
    pub request: RequestApi,
    pub system: SystemApi,
    pub napcat: NapCatApi,
    pub raw: RawActionApi,
}
```

**Commit:**
```bash
git add -A && git commit -m "feat: implement OneBotApi aggregator"
```

---

## Phase 6: Public Re-exports & lib.rs

### Task 24: Wire up lib.rs public API

**Files:**
- Modify: `NapCatLink/src/lib.rs`

Re-export all public types for ergonomic usage:
```rust
pub use client::NapLink;
pub use config::{NapLinkConfig, LogLevel};
pub use error::{NapLinkError, Result};
pub use types::message::MessageSegment;
pub use event::EventSubscription;
pub use connection::state::ConnectionState;
pub use api::OneBotApi;
```

**Commit:**
```bash
git add -A && git commit -m "feat: wire up public API re-exports in lib.rs"
```

---

## Phase 7: Examples & Documentation

### Task 25: Create examples

**Files:**
- Create: `NapCatLink/examples/basic.rs` — minimal connect + echo bot
- Create: `NapCatLink/examples/group_admin.rs` — group management
- Create: `NapCatLink/examples/echo_bot.rs` — echo all messages

**Commit:**
```bash
git add -A && git commit -m "feat: add usage examples (basic, group_admin, echo_bot)"
```

---

### Task 26: Write development documentation

**Files:**
- Create: `NapCatLink/docs/development/architecture.md`
- Create: `NapCatLink/docs/development/api-reference.md`
- Create: `NapCatLink/docs/development/getting-started.md`
- Modify: `NapCatLink/docs/plans/` (update plan status)

**Commit:**
```bash
git add -A && git commit -m "docs: add development documentation"
```

---

## Phase 8: Final Verification

### Task 27: Full compilation check and cargo clippy

Run:
```bash
cd NapCatLink && cargo clippy -- -D warnings
cd NapCatLink && cargo test
cd NapCatLink && cargo doc --no-deps
```

Fix any warnings, test failures, or doc issues.

**Commit:**
```bash
git add -A && git commit -m "chore: fix clippy warnings and verify full test suite"
git tag v0.1.0-alpha.2
```

---

## Summary

| Phase | Tasks | Description |
|-------|-------|-------------|
| 1 | 1-5 | Project scaffolding, error/config/message/event types |
| 2 | 6-9 | Connection layer (state, heartbeat, reconnect, actor) |
| 3 | 10-11 | Event system (bus, router) |
| 4 | 12-21 | API layer (client, dispatcher, all 10 API modules) |
| 5 | 22-23 | NapLink client facade + API aggregator |
| 6 | 24 | Public re-exports |
| 7 | 25-26 | Examples and documentation |
| 8 | 27 | Final verification |

Total: 27 tasks, covering ~45 source files
