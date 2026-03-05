//! # 配置类型测试模块
//!
//! 测试 `NapLinkConfigBuilder` 的构建行为和默认值，
//! 以及配置验证逻辑。

// 引入配置相关类型
use napcat_link::config::{BackoffConfig, LogLevel, NapLinkConfigBuilder};

/// 测试使用最少参数（仅 URL）构建配置
///
/// 验证 Builder 在仅提供 URL 时，能使用合理的默认值构建完整配置。
#[test]
fn test_builder_minimal() {
    // 仅提供 WebSocket 地址，其余使用默认值
    let config = NapLinkConfigBuilder::new("ws://127.0.0.1:3001")
        .build()
        .expect("最少参数构建应该成功");

    // 验证连接配置的默认值
    assert_eq!(config.connection.url, "ws://127.0.0.1:3001");
    // Token 默认为 None
    assert!(config.connection.token.is_none());
    // 连接超时默认 30000ms
    assert_eq!(config.connection.timeout_ms, 30000);
    // 心跳间隔默认 30000ms
    assert_eq!(config.connection.ping_interval_ms, 30000);
    // 心跳动作默认为 get_status
    assert_eq!(config.connection.heartbeat_action.action, "get_status");

    // 验证重连配置的默认值
    // 默认启用自动重连
    assert!(config.reconnect.enabled);
    // 默认最大重连 10 次（与 TS 版一致）
    assert_eq!(config.reconnect.max_attempts, 10);
    // 默认初始退避 1000ms（与 TS 版一致）
    assert_eq!(config.reconnect.backoff.initial_ms, 1000);
    // 默认最大退避 60000ms
    assert_eq!(config.reconnect.backoff.max_ms, 60000);
    // 默认退避乘数 2.0
    assert!((config.reconnect.backoff.multiplier - 2.0).abs() < f64::EPSILON);

    // 验证日志配置的默认值
    // 默认日志级别为 Info
    assert_eq!(config.logging.level, LogLevel::Info);

    // 验证 API 配置的默认值
    // API 超时默认 30000ms（与 TS 版一致）
    assert_eq!(config.api.timeout_ms, 30000);
    // API 重试默认 3 次（与 TS 版一致）
    assert_eq!(config.api.retries, 3);
}

/// 测试使用全部参数构建配置
///
/// 验证 Builder 在所有字段都被显式设置时，能正确保存每个配置值。
#[test]
fn test_builder_full() {
    // 使用所有可选参数构建配置
    let config = NapLinkConfigBuilder::new("ws://10.0.0.1:6700")
        // 设置认证 Token
        .token("my-secret-token")
        // 设置连接超时为 10 秒
        .timeout_ms(10000)
        // 设置心跳间隔为 20 秒
        .ping_interval_ms(20000)
        // 禁用自动重连
        .reconnect_enabled(false)
        // 设置最大重连 10 次
        .max_reconnect_attempts(10)
        // 设置自定义退避策略
        .backoff(BackoffConfig {
            initial_ms: 1000,
            max_ms: 30000,
            multiplier: 1.5,
        })
        // 设置日志级别为 Debug
        .log_level(LogLevel::Debug)
        // 设置 API 超时为 5 秒
        .api_timeout_ms(5000)
        // 设置 API 重试 3 次
        .api_retries(3)
        .build()
        .expect("全参数构建应该成功");

    // 验证连接配置
    assert_eq!(config.connection.url, "ws://10.0.0.1:6700");
    assert_eq!(config.connection.token.as_deref(), Some("my-secret-token"));
    assert_eq!(config.connection.timeout_ms, 10000);
    assert_eq!(config.connection.ping_interval_ms, 20000);

    // 验证重连配置
    assert!(!config.reconnect.enabled);
    assert_eq!(config.reconnect.max_attempts, 10);
    assert_eq!(config.reconnect.backoff.initial_ms, 1000);
    assert_eq!(config.reconnect.backoff.max_ms, 30000);
    assert!((config.reconnect.backoff.multiplier - 1.5).abs() < f64::EPSILON);

    // 验证日志级别
    assert_eq!(config.logging.level, LogLevel::Debug);

    // 验证 API 配置
    assert_eq!(config.api.timeout_ms, 5000);
    assert_eq!(config.api.retries, 3);
}

/// 测试空 URL 构建失败
///
/// 验证当 URL 为空字符串时，Builder 的 `build()` 方法返回 `InvalidConfig` 错误。
#[test]
fn test_builder_empty_url_fails() {
    // 使用空字符串作为 URL
    let result = NapLinkConfigBuilder::new("").build();
    // 验证构建失败
    assert!(result.is_err(), "空 URL 应该导致构建失败");

    // 获取错误并验证类型
    let err = result.unwrap_err();
    // 验证错误码为无效配置
    assert_eq!(err.code(), "E_INVALID_CONFIG");
    // 验证错误消息包含字段名
    assert!(err.to_string().contains("url"), "错误消息应包含字段名 'url'");

    // 测试仅包含空白字符的 URL 也应该失败
    let result_whitespace = NapLinkConfigBuilder::new("   ").build();
    assert!(result_whitespace.is_err(), "仅空白字符的 URL 应该导致构建失败");
}
