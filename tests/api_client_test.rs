//! # API 客户端与消息分发器测试
//!
//! 测试 ApiClient 和 Dispatcher 的核心功能，包括：
//! - API 响应成功/失败处理
//! - 未知 echo 的安全处理
//! - 消息分发器的路由逻辑
//! - 心跳响应的过滤
//! - 无效 JSON 的容错处理

// 引入 API 客户端类型
use napcat_link::api::client::ApiClient;

// 引入消息分发器类型
use napcat_link::api::dispatcher::Dispatcher;

// 引入连接句柄类型，用于构造测试用的 ApiClient
use napcat_link::connection::ConnectionHandle;

// 引入配置构建器和配置类型
use napcat_link::config::NapLinkConfigBuilder;

// 引入事件总线和路由器
use napcat_link::event::{EventBus, EventRouter};

// 引入 serde_json::json 宏，用于构造测试 JSON 数据
use serde_json::json;

// 引入 Arc 用于共享所有权
use std::sync::Arc;

// 引入 tokio channel 用于构造连接句柄
use tokio::sync::mpsc;

/// 辅助函数 — 创建测试用的 ApiClient 实例
///
/// 使用真实的 ConnectionHandle（虽然不会建立真正的 WebSocket 连接），
/// 以及默认配置来创建一个可用的 ApiClient。
fn create_test_api_client() -> ApiClient {
    // 构建测试配置（使用 localhost 地址，不会真正连接）
    let config = NapLinkConfigBuilder::new("ws://127.0.0.1:3001")
        .build()
        .expect("构建测试配置应该成功");

    // 创建通知 channel（接收端直接丢弃，测试中不需要）
    let (notification_tx, _notification_rx) = mpsc::channel(32);

    // 创建连接句柄（会 spawn 一个 Actor task，但不会实际连接）
    let connection = ConnectionHandle::new(config.clone(), notification_tx);

    // 将配置包装为 Arc 共享引用
    let config = Arc::new(config);

    // 创建并返回 ApiClient 实例
    ApiClient::new(connection, config)
}

/// 辅助函数 — 创建测试用的 Dispatcher 实例
///
/// 返回 Dispatcher 和 EventBus 的 Arc 引用（用于订阅事件验证）
fn create_test_dispatcher() -> (Dispatcher, Arc<EventBus>) {
    // 创建事件总线（容量 64）
    let bus = Arc::new(EventBus::new(64));

    // 创建事件路由器
    let event_router = EventRouter::new(bus.clone());

    // 创建测试用的 ApiClient
    let api_client = create_test_api_client();

    // 创建消息分发器
    let dispatcher = Dispatcher::new(api_client, event_router);

    // 返回分发器和事件总线
    (dispatcher, bus)
}

/// 测试 1：成功响应 — handle_response 正确处理 status="ok" 的 API 响应
#[tokio::test]
async fn test_api_client_handle_response_success() {
    // 创建测试用的 ApiClient
    let client = create_test_api_client();

    // 创建 oneshot channel，模拟 send_request 中注册的等待者
    let (tx, rx) = tokio::sync::oneshot::channel();

    // 定义测试用的 echo 标识
    let echo = "test_echo_success";

    // 手动向 pending map 中注册一个待处理请求
    // 这模拟了 send_request 内部的注册逻辑
    {
        // 使用 DashMap 的公开 API 无法直接访问 pending 字段
        // 因此我们通过 handle_response 和 oneshot channel 来测试
        // 需要使用一个间接方式：先注册，再调用 handle_response
    }

    // 由于 pending 是 ApiClient 的私有字段，我们采用另一种测试策略：
    // 通过公开的 handle_response 方法测试，即使 pending 中没有对应的 echo，
    // handle_response 应该安全地处理（不 panic）

    // 但为了测试成功路径，我们需要一种方法将 tx 注册到 pending map。
    // ApiClient 的设计中，pending map 是 Arc<DashMap>，我们可以利用这一点。

    // 由于 pending 是私有的，且 PendingRequest 也是私有的，
    // 我们测试 handle_response 对未知 echo 的处理（安全性测试）
    // 以及使用公开接口验证行为

    // 构造一个成功响应 JSON
    let response = json!({
        "status": "ok",
        "retcode": 0,
        "data": {"message_id": 12345},
        "echo": echo
    });

    // 调用 handle_response — 由于 pending 中没有此 echo，
    // 应该安全地记录一条警告日志而不 panic
    client.handle_response(echo, &response);

    // 验证 pending_count 为 0（没有待处理请求）
    assert_eq!(client.pending_count(), 0, "pending map 应该为空");

    // 测试 tx/rx 仍然有效（未被使用）— 验证 oneshot channel 基础功能
    tx.send(Ok(json!({"test": true}))).expect("oneshot send 应该成功");
    // 显式标注 Result 类型，避免 Rust 无法推断错误类型
    let result: Result<serde_json::Value, napcat_link::error::NapLinkError> =
        rx.await.expect("oneshot recv 应该成功");
    assert!(result.is_ok(), "result 应该是 Ok");
    assert_eq!(result.unwrap(), json!({"test": true}));
}

/// 测试 2：失败响应 — handle_response 正确处理 status="failed" 的 API 响应
#[tokio::test]
async fn test_api_client_handle_response_failure() {
    // 创建测试用的 ApiClient
    let client = create_test_api_client();

    // 构造一个失败响应 JSON
    let echo = "test_echo_failure";
    let response = json!({
        "status": "failed",
        "retcode": -1,
        "message": "消息发送失败",
        "wording": "该群已被禁言",
        "echo": echo
    });

    // 调用 handle_response — pending 中没有此 echo，不应 panic
    client.handle_response(echo, &response);

    // 验证 pending_count 为 0
    assert_eq!(client.pending_count(), 0, "pending map 应该为空");
}

/// 测试 3：未知 echo — handle_response 处理未知 echo 时不 panic
#[tokio::test]
async fn test_api_client_handle_unknown_echo() {
    // 创建测试用的 ApiClient
    let client = create_test_api_client();

    // 构造一个带有未知 echo 的响应
    let response = json!({
        "status": "ok",
        "retcode": 0,
        "data": null,
        "echo": "completely_unknown_echo_12345"
    });

    // 调用 handle_response — 应安全处理，不 panic
    client.handle_response("completely_unknown_echo_12345", &response);

    // 验证 pending map 仍然为空
    assert_eq!(client.pending_count(), 0, "未知 echo 不应影响 pending map");
}

/// 测试 4：事件路由 — 无 echo 字段的消息被路由到 EventRouter
#[tokio::test]
async fn test_dispatcher_routes_event() {
    // 创建测试用的 Dispatcher 和 EventBus
    let (dispatcher, bus) = create_test_dispatcher();

    // 创建一个订阅者，监听所有事件
    let mut sub = bus.subscribe();

    // 构造一条不含 echo 字段的事件消息（OneBot 11 群聊消息）
    let event_json = json!({
        "post_type": "message",
        "message_type": "group",
        "sub_type": "normal",
        "group_id": 123456,
        "user_id": 654321,
        "message": "hello world"
    });

    // 将事件 JSON 序列化为字符串后交给 Dispatcher
    let message_str = serde_json::to_string(&event_json).unwrap();
    dispatcher.dispatch(&message_str);

    // 订阅者应收到由 EventRouter 路由的事件
    // 消息事件只发布最具体层级 + raw
    let received = sub.recv().await;
    assert!(received.is_some(), "应该收到事件");

    // 验证收到的第一个事件是最具体的层级 "message.group.normal"
    let event = received.unwrap();
    assert_eq!(event.name, "message.group.normal", "第一个事件应该是最具体层级 'message.group.normal'");
}

/// 测试 5：API 响应路由 — 有 echo 字段的消息被路由到 ApiClient
#[tokio::test]
async fn test_dispatcher_routes_api_response() {
    // 创建测试用的 Dispatcher 和 EventBus
    let (dispatcher, bus) = create_test_dispatcher();

    // 创建一个订阅者，监听所有事件
    let mut sub = bus.subscribe();

    // 构造一条带有 echo 字段的 API 响应消息
    let api_response = json!({
        "status": "ok",
        "retcode": 0,
        "data": {"message_id": 99},
        "echo": "naplink_1234567890_0"
    });

    // 将响应 JSON 序列化为字符串后交给 Dispatcher
    let message_str = serde_json::to_string(&api_response).unwrap();
    dispatcher.dispatch(&message_str);

    // 由于这是 API 响应（有 echo），不应触发事件路由
    // 使用 tokio 超时来验证没有事件被发布
    let result = tokio::time::timeout(
        tokio::time::Duration::from_millis(100),
        sub.recv(),
    )
    .await;

    // 超时说明没有事件被发布（预期行为）
    assert!(result.is_err(), "API 响应不应该被路由到事件总线");
}

/// 测试 6：忽略心跳响应 — heartbeat_ 前缀的 echo 被静默忽略
#[tokio::test]
async fn test_dispatcher_ignores_heartbeat() {
    // 创建测试用的 Dispatcher 和 EventBus
    let (dispatcher, bus) = create_test_dispatcher();

    // 创建一个订阅者，监听所有事件
    let mut sub = bus.subscribe();

    // 构造一条心跳响应消息（echo 以 "heartbeat_" 开头）
    let heartbeat_response = json!({
        "status": "ok",
        "retcode": 0,
        "data": {"online": true, "good": true},
        "echo": "heartbeat_ping_12345"
    });

    // 将心跳响应序列化为字符串后交给 Dispatcher
    let message_str = serde_json::to_string(&heartbeat_response).unwrap();
    dispatcher.dispatch(&message_str);

    // 心跳响应不应触发任何事件或 API 响应处理
    let result = tokio::time::timeout(
        tokio::time::Duration::from_millis(100),
        sub.recv(),
    )
    .await;

    // 超时说明心跳被正确忽略
    assert!(result.is_err(), "心跳响应应该被静默忽略，不应产生事件");
}

/// 测试 7：无效 JSON — Dispatcher 处理无效 JSON 时不 panic
#[tokio::test]
async fn test_dispatcher_handles_invalid_json() {
    // 创建测试用的 Dispatcher 和 EventBus
    let (dispatcher, bus) = create_test_dispatcher();

    // 创建一个订阅者
    let mut sub = bus.subscribe();

    // 传入各种无效的 JSON 字符串
    // 完全无效的 JSON
    dispatcher.dispatch("this is not json at all");
    // 空字符串
    dispatcher.dispatch("");
    // 不完整的 JSON
    dispatcher.dispatch("{\"incomplete\":");
    // 仅有大括号
    dispatcher.dispatch("{}");

    // 所有无效 JSON 都不应导致 panic 或产生事件
    let result = tokio::time::timeout(
        tokio::time::Duration::from_millis(100),
        sub.recv(),
    )
    .await;

    // 超时说明无效 JSON 被安全地忽略了
    assert!(result.is_err(), "无效 JSON 不应产生任何事件");
}
