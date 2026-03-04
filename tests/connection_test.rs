//! # 连接层测试模块
//!
//! 测试连接层的各个组件：
//! - ConnectionState 状态枚举的 Display 实现
//! - ReconnectService 的指数退避延迟计算
//! - ReconnectService 的最大重连次数限制
//! - ReconnectService 的重置逻辑
//! - ReconnectService 禁用时的行为
//! - build_websocket_url 的 URL 构建逻辑

// 引入连接状态类型
use napcat_link::connection::state::ConnectionState;
// 引入重连服务
use napcat_link::connection::reconnect::ReconnectService;
// 引入 URL 构建函数
use napcat_link::connection::build_websocket_url;
// 引入配置类型
use napcat_link::config::{BackoffConfig, NapLinkConfigBuilder, ReconnectConfig};
// 引入 Duration 类型用于延迟断言
use std::time::Duration;

/// 测试 ConnectionState 的 Display 实现
///
/// 验证每个状态枚举变体的字符串表示是否正确。
#[test]
fn test_connection_state_display() {
    // Disconnected 应显示为 "disconnected"
    assert_eq!(ConnectionState::Disconnected.to_string(), "disconnected");
    // Connecting 应显示为 "connecting"
    assert_eq!(ConnectionState::Connecting.to_string(), "connecting");
    // Connected 应显示为 "connected"
    assert_eq!(ConnectionState::Connected.to_string(), "connected");
    // Reconnecting 应显示为 "reconnecting"
    assert_eq!(ConnectionState::Reconnecting.to_string(), "reconnecting");
}

/// 测试 ConnectionState 的相等性比较
///
/// 验证 PartialEq 和 Eq 的正确实现。
#[test]
fn test_connection_state_equality() {
    // 相同状态应该相等
    assert_eq!(ConnectionState::Connected, ConnectionState::Connected);
    // 不同状态不应该相等
    assert_ne!(ConnectionState::Connected, ConnectionState::Disconnected);
}

/// 测试 ConnectionState 的 Clone 和 Copy
///
/// 验证状态枚举可以被复制和克隆。
#[test]
fn test_connection_state_clone_copy() {
    // 测试 Copy 语义：赋值后原始变量仍然可用
    let state = ConnectionState::Connected;
    let state_copy = state;
    // 原始变量和拷贝应相等
    assert_eq!(state, state_copy);
    // Clone 后也应相等
    assert_eq!(state, state.clone());
}

/// 测试 ReconnectService 的指数退避延迟计算
///
/// 使用默认配置 (initial=5000ms, multiplier=2.0, max=60000ms)
/// 验证退避延迟按 5000, 10000, 20000, 40000, 60000 递增。
#[test]
fn test_reconnect_exponential_backoff() {
    // 创建自定义配置以方便测试
    let config = ReconnectConfig {
        // 启用重连
        enabled: true,
        // 最大 10 次，确保测试不会提前终止
        max_attempts: 10,
        backoff: BackoffConfig {
            // 初始延迟 1000ms
            initial_ms: 1000,
            // 最大延迟 16000ms
            max_ms: 16000,
            // 乘数 2.0（每次翻倍）
            multiplier: 2.0,
        },
    };

    // 创建重连服务
    let mut svc = ReconnectService::new(config);

    // 第 0 次: 1000 * 2^0 = 1000ms
    assert_eq!(svc.next_delay(), Some(Duration::from_millis(1000)));
    // 第 1 次: 1000 * 2^1 = 2000ms
    assert_eq!(svc.next_delay(), Some(Duration::from_millis(2000)));
    // 第 2 次: 1000 * 2^2 = 4000ms
    assert_eq!(svc.next_delay(), Some(Duration::from_millis(4000)));
    // 第 3 次: 1000 * 2^3 = 8000ms
    assert_eq!(svc.next_delay(), Some(Duration::from_millis(8000)));
    // 第 4 次: 1000 * 2^4 = 16000ms（达到上限）
    assert_eq!(svc.next_delay(), Some(Duration::from_millis(16000)));
    // 第 5 次: 1000 * 2^5 = 32000ms → 被限制为 16000ms
    assert_eq!(svc.next_delay(), Some(Duration::from_millis(16000)));
}

/// 测试 ReconnectService 达到最大重连次数后返回 None
///
/// 验证当尝试次数达到 max_attempts 后，next_delay 返回 None。
#[test]
fn test_reconnect_max_attempts() {
    // 创建最大重连 3 次的配置
    let config = ReconnectConfig {
        enabled: true,
        // 只允许 3 次重连
        max_attempts: 3,
        backoff: BackoffConfig {
            initial_ms: 1000,
            max_ms: 60000,
            multiplier: 2.0,
        },
    };

    // 创建重连服务
    let mut svc = ReconnectService::new(config);

    // 第 0 次: 有延迟
    assert!(svc.next_delay().is_some());
    // 第 1 次: 有延迟
    assert!(svc.next_delay().is_some());
    // 第 2 次: 有延迟（这是最后一次）
    assert!(svc.next_delay().is_some());
    // 第 3 次: 返回 None，已达到最大次数
    assert_eq!(svc.next_delay(), None);
    // 之后继续调用也返回 None
    assert_eq!(svc.next_delay(), None);

    // 验证当前尝试次数为 3
    assert_eq!(svc.current_attempt(), 3);
    // 验证没有剩余重连机会
    assert!(!svc.has_remaining_attempts());
}

/// 测试 ReconnectService 的 reset 方法
///
/// 验证 reset 后尝试次数归零，可以重新开始重连。
#[test]
fn test_reconnect_reset() {
    // 创建最大重连 2 次的配置
    let config = ReconnectConfig {
        enabled: true,
        max_attempts: 2,
        backoff: BackoffConfig {
            initial_ms: 1000,
            max_ms: 60000,
            multiplier: 2.0,
        },
    };

    // 创建重连服务
    let mut svc = ReconnectService::new(config);

    // 消耗所有重连机会
    assert!(svc.next_delay().is_some()); // 第 0 次
    assert!(svc.next_delay().is_some()); // 第 1 次
    assert_eq!(svc.next_delay(), None); // 已用完

    // 验证当前尝试次数为 2
    assert_eq!(svc.current_attempt(), 2);

    // 重置计数器
    svc.reset();

    // 验证尝试次数归零
    assert_eq!(svc.current_attempt(), 0);
    // 验证又有重连机会了
    assert!(svc.has_remaining_attempts());

    // 重置后可以再次获取延迟
    assert_eq!(svc.next_delay(), Some(Duration::from_millis(1000)));
}

/// 测试 ReconnectService 禁用时直接返回 None
///
/// 验证当 reconnect.enabled = false 时，next_delay 始终返回 None。
#[test]
fn test_reconnect_disabled() {
    // 创建禁用重连的配置
    let config = ReconnectConfig {
        // 禁用重连
        enabled: false,
        max_attempts: 5,
        backoff: BackoffConfig::default(),
    };

    // 创建重连服务
    let mut svc = ReconnectService::new(config);

    // 禁用时直接返回 None
    assert_eq!(svc.next_delay(), None);
    // 重复调用也是 None
    assert_eq!(svc.next_delay(), None);

    // 验证没有剩余重连机会（因为禁用了）
    assert!(!svc.has_remaining_attempts());
}

/// 测试 ReconnectService 的辅助方法
///
/// 验证 current_attempt、has_remaining_attempts、max_attempts 的正确性。
#[test]
fn test_reconnect_service_helpers() {
    // 创建最大重连 3 次的配置
    let config = ReconnectConfig {
        enabled: true,
        max_attempts: 3,
        backoff: BackoffConfig::default(),
    };

    // 创建重连服务
    let mut svc = ReconnectService::new(config);

    // 初始状态：0 次尝试，有剩余机会，最大 3 次
    assert_eq!(svc.current_attempt(), 0);
    assert!(svc.has_remaining_attempts());
    assert_eq!(svc.max_attempts(), 3);

    // 消耗一次
    svc.next_delay();
    assert_eq!(svc.current_attempt(), 1);
    assert!(svc.has_remaining_attempts());

    // 消耗第二次
    svc.next_delay();
    assert_eq!(svc.current_attempt(), 2);
    assert!(svc.has_remaining_attempts());

    // 消耗第三次（最后一次）
    svc.next_delay();
    assert_eq!(svc.current_attempt(), 3);
    assert!(!svc.has_remaining_attempts());
}

/// 测试 build_websocket_url — 无 Token 的情况
///
/// 验证没有 Token 时 URL 保持不变。
#[test]
fn test_build_websocket_url_no_token() {
    // 创建不带 Token 的配置
    let config = NapLinkConfigBuilder::new("ws://127.0.0.1:3001")
        .build()
        .unwrap();

    // 构建 URL
    let url = build_websocket_url(&config);

    // 无 Token 时 URL 应保持不变
    assert_eq!(url, "ws://127.0.0.1:3001");
}

/// 测试 build_websocket_url — 有 Token 的情况
///
/// 验证有 Token 时 URL 附加 access_token 参数。
#[test]
fn test_build_websocket_url_with_token() {
    // 创建带 Token 的配置
    let config = NapLinkConfigBuilder::new("ws://127.0.0.1:3001")
        .token("my-secret-token")
        .build()
        .unwrap();

    // 构建 URL
    let url = build_websocket_url(&config);

    // 应附加 access_token 查询参数
    assert_eq!(url, "ws://127.0.0.1:3001?access_token=my-secret-token");
}

/// 测试 build_websocket_url — URL 已有查询参数的情况
///
/// 验证当 URL 已包含 '?' 时，Token 使用 '&' 连接。
#[test]
fn test_build_websocket_url_with_existing_query() {
    // 创建 URL 中已有查询参数的配置
    let config = NapLinkConfigBuilder::new("ws://127.0.0.1:3001?foo=bar")
        .token("my-token")
        .build()
        .unwrap();

    // 构建 URL
    let url = build_websocket_url(&config);

    // Token 应使用 '&' 连接
    assert_eq!(url, "ws://127.0.0.1:3001?foo=bar&access_token=my-token");
}

/// 测试 build_websocket_url — Token 为空字符串的情况
///
/// 验证空 Token 时 URL 保持不变（与无 Token 行为一致）。
#[test]
fn test_build_websocket_url_empty_token() {
    // 创建带空 Token 的配置
    let config = NapLinkConfigBuilder::new("ws://127.0.0.1:3001")
        .token("")
        .build()
        .unwrap();

    // 构建 URL
    let url = build_websocket_url(&config);

    // 空 Token 时 URL 应保持不变
    assert_eq!(url, "ws://127.0.0.1:3001");
}
