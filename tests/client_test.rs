//! # NapLink 客户端集成测试
//!
//! 测试 NapLink 客户端的创建、配置和初始状态。
//! 这些测试不需要实际的 WebSocket 服务器连接。

// 引入 NapLink 客户端类型
use napcat_link::NapLink;

// 引入连接状态枚举，用于验证初始状态
use napcat_link::ConnectionState;

// 引入错误类型，用于验证配置错误
use napcat_link::NapLinkError;

/// 测试：使用 builder 成功创建 NapLink 实例
///
/// 验证通过有效的 URL 和 Token 配置可以成功构建 NapLink 客户端。
/// 构建后客户端应处于未连接状态。
#[tokio::test]
async fn test_naplink_builder_creates_instance() {
    // 使用有效的 URL 和 Token 构建客户端
    let client = NapLink::builder("ws://127.0.0.1:3001")
        .token("test_token")
        .connection_timeout_ms(5000)
        .ping_interval_ms(10000)
        .reconnect_enabled(true)
        .reconnect_max_attempts(3)
        .api_timeout_ms(10000)
        .api_retries(1)
        .build();

    // 断言构建成功
    assert!(client.is_ok(), "builder 应该成功创建 NapLink 实例");

    // 获取客户端实例
    let client = client.unwrap();

    // 验证初始状态为断开连接
    assert_eq!(
        client.state(),
        ConnectionState::Disconnected,
        "新创建的客户端应处于 Disconnected 状态"
    );

    // 验证 is_connected 返回 false
    assert!(
        !client.is_connected(),
        "新创建的客户端不应处于已连接状态"
    );

    // 验证配置正确传递
    assert_eq!(
        client.config().connection.url,
        "ws://127.0.0.1:3001",
        "URL 应与配置一致"
    );
    assert_eq!(
        client.config().connection.token.as_deref(),
        Some("test_token"),
        "Token 应与配置一致"
    );
    assert_eq!(
        client.config().connection.timeout_ms,
        5000,
        "连接超时应与配置一致"
    );
    assert_eq!(
        client.config().api.timeout_ms,
        10000,
        "API 超时应与配置一致"
    );
    assert_eq!(
        client.config().api.retries,
        1,
        "API 重试次数应与配置一致"
    );
}

/// 测试：空 URL 构建失败
///
/// 验证当提供空 URL 时，builder 会返回 InvalidConfig 错误。
#[tokio::test]
async fn test_naplink_builder_invalid_url_fails() {
    // 使用空 URL 构建客户端
    let result = NapLink::builder("")
        .build();

    // 断言构建失败
    assert!(result.is_err(), "空 URL 应导致构建失败");

    // 验证错误类型为 InvalidConfig
    let err = result.unwrap_err();
    match &err {
        NapLinkError::InvalidConfig { field, .. } => {
            // 验证错误字段为 "url"
            assert_eq!(field, "url", "错误字段应为 url");
        }
        _ => panic!("期望 InvalidConfig 错误，但得到: {:?}", err),
    }
}

/// 测试：初始状态为 Disconnected
///
/// 验证新创建的客户端初始连接状态为 Disconnected。
#[tokio::test]
async fn test_naplink_initial_state_is_disconnected() {
    // 使用最简配置构建客户端
    let client = NapLink::builder("ws://127.0.0.1:3001")
        .build()
        .expect("有效 URL 的构建不应失败");

    // 验证初始状态
    assert_eq!(
        client.state(),
        ConnectionState::Disconnected,
        "初始状态应为 Disconnected"
    );

    // 验证 is_connected 返回 false
    assert!(
        !client.is_connected(),
        "初始状态下 is_connected 应返回 false"
    );

    // 验证可以正常获取 API 引用（不会 panic）
    let _api = client.api();

    // 验证可以创建事件订阅（不会 panic）
    let _sub = client.subscribe();
}
