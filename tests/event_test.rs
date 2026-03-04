//! # 事件系统测试
//!
//! 测试 EventBus 和 EventRouter 的核心功能，包括：
//! - 基本发布订阅
//! - 多订阅者广播
//! - 前缀过滤和精确匹配
//! - 各类 OneBot 11 事件的路由

// 引入事件总线和路由器的核心类型
use napcat_link::event::{EventBus, EventRouter};

// 引入 serde_json::json 宏，用于构造测试用的 JSON 数据
use serde_json::json;

// 引入 Arc 用于在多个地方共享事件总线的所有权
use std::sync::Arc;

/// 测试 1：基本发布订阅 — 发布一个事件后订阅者能够接收到
#[tokio::test]
async fn test_event_bus_publish_subscribe() {
    // 创建缓冲区容量为 16 的事件总线
    let bus = EventBus::new(16);
    // 创建一个订阅者
    let mut sub = bus.subscribe();

    // 构造测试用的 JSON 数据
    let test_data = json!({"key": "value"});
    // 发布名为 "test.event" 的事件
    bus.publish("test.event", test_data.clone());

    // 接收事件并验证
    let received = sub.recv().await;
    // 断言接收到的事件不为 None
    assert!(received.is_some(), "应该接收到事件");
    // 解包事件数据
    let event = received.unwrap();
    // 验证事件名称正确
    assert_eq!(event.name, "test.event");
    // 验证事件数据正确
    assert_eq!(event.data, test_data);
}

/// 测试 2：多个订阅者 — 发布一个事件后所有订阅者都能接收到
#[tokio::test]
async fn test_event_bus_multiple_subscribers() {
    // 创建事件总线
    let bus = EventBus::new(16);
    // 创建两个独立的订阅者
    let mut sub1 = bus.subscribe();
    let mut sub2 = bus.subscribe();

    // 构造测试数据并发布
    let test_data = json!({"msg": "broadcast"});
    bus.publish("broadcast.test", test_data.clone());

    // 两个订阅者都应该接收到相同的事件
    let received1 = sub1.recv().await;
    let received2 = sub2.recv().await;

    // 断言两个订阅者都接收到了事件
    assert!(received1.is_some(), "订阅者 1 应该接收到事件");
    assert!(received2.is_some(), "订阅者 2 应该接收到事件");

    // 验证两个订阅者接收到的事件名称和数据都一致
    let event1 = received1.unwrap();
    let event2 = received2.unwrap();
    assert_eq!(event1.name, "broadcast.test");
    assert_eq!(event2.name, "broadcast.test");
    assert_eq!(event1.data, test_data);
    assert_eq!(event2.data, test_data);
}

/// 测试 3：按前缀过滤接收 — recv_filter 只接收名称前缀匹配的事件
#[tokio::test]
async fn test_event_subscription_recv_filter() {
    // 创建事件总线
    let bus = EventBus::new(16);
    // 创建订阅者
    let mut sub = bus.subscribe();

    // 发布多个不同前缀的事件
    bus.publish("notice.group_increase", json!({"type": "notice"}));
    bus.publish("message.group.normal", json!({"type": "message"}));
    bus.publish("message.private", json!({"type": "message_private"}));

    // 使用前缀 "message" 过滤，应该跳过 notice 事件，接收到第一个 message 事件
    let received = sub.recv_filter("message").await;
    assert!(received.is_some(), "应该接收到 message 前缀的事件");
    let event = received.unwrap();
    // 第一个匹配的是 "message.group.normal"
    assert_eq!(event.name, "message.group.normal");
    assert_eq!(event.data, json!({"type": "message"}));
}

/// 测试 4：精确匹配过滤 — recv_exact 只接收名称完全匹配的事件
#[tokio::test]
async fn test_event_subscription_recv_exact() {
    // 创建事件总线
    let bus = EventBus::new(16);
    // 创建订阅者
    let mut sub = bus.subscribe();

    // 发布多个事件，包括部分名称相似的事件
    bus.publish("message", json!({"level": 1}));
    bus.publish("message.group", json!({"level": 2}));
    bus.publish("message.group.normal", json!({"level": 3}));

    // 使用精确匹配 "message.group.normal"，应该跳过前两个事件
    let received = sub.recv_exact("message.group.normal").await;
    assert!(received.is_some(), "应该接收到精确匹配的事件");
    let event = received.unwrap();
    // 验证接收到的是第三个事件
    assert_eq!(event.name, "message.group.normal");
    assert_eq!(event.data, json!({"level": 3}));
}

/// 测试 5：消息事件路由 — 验证 message 类型事件生成 3 级层级事件名
#[tokio::test]
async fn test_event_router_message_routing() {
    // 创建共享的事件总线
    let bus = Arc::new(EventBus::new(64));
    // 创建事件路由器
    let router = EventRouter::new(bus.clone());
    // 创建订阅者用于接收路由后的事件
    let mut sub = bus.subscribe();

    // 构造一条群聊消息事件 JSON
    let msg_event = json!({
        "post_type": "message",
        "message_type": "group",
        "sub_type": "normal",
        "group_id": 123456,
        "user_id": 654321,
        "message": "hello"
    });

    // 通过路由器分发事件
    router.route(msg_event.clone());

    // 收集所有路由产生的事件名称
    let mut event_names = Vec::new();
    // 消息事件应产生 4 个事件: message, message.group, message.group.normal, raw
    for _ in 0..4 {
        if let Some(event) = sub.recv().await {
            event_names.push(event.name);
        }
    }

    // 验证所有层级的事件名都已生成
    assert!(
        event_names.contains(&"message".to_string()),
        "应包含第一层级 'message' 事件"
    );
    assert!(
        event_names.contains(&"message.group".to_string()),
        "应包含第二层级 'message.group' 事件"
    );
    assert!(
        event_names.contains(&"message.group.normal".to_string()),
        "应包含第三层级 'message.group.normal' 事件"
    );
    assert!(
        event_names.contains(&"raw".to_string()),
        "应包含 'raw' 事件"
    );
}

/// 测试 6：通知事件路由 — 验证 notice 类型事件的层级路由
#[tokio::test]
async fn test_event_router_notice_routing() {
    // 创建共享的事件总线
    let bus = Arc::new(EventBus::new(64));
    // 创建事件路由器
    let router = EventRouter::new(bus.clone());
    // 创建订阅者
    let mut sub = bus.subscribe();

    // 构造一条群成员增加的通知事件
    let notice_event = json!({
        "post_type": "notice",
        "notice_type": "group_increase",
        "sub_type": "approve",
        "group_id": 123456,
        "user_id": 789012
    });

    // 通过路由器分发事件
    router.route(notice_event.clone());

    // 收集所有路由产生的事件名称
    let mut event_names = Vec::new();
    // 通知事件应产生 4 个事件: notice, notice.group_increase, notice.group_increase.approve, raw
    for _ in 0..4 {
        if let Some(event) = sub.recv().await {
            event_names.push(event.name);
        }
    }

    // 验证所有层级的事件名都已生成
    assert!(
        event_names.contains(&"notice".to_string()),
        "应包含第一层级 'notice' 事件"
    );
    assert!(
        event_names.contains(&"notice.group_increase".to_string()),
        "应包含第二层级 'notice.group_increase' 事件"
    );
    assert!(
        event_names.contains(&"notice.group_increase.approve".to_string()),
        "应包含第三层级 'notice.group_increase.approve' 事件"
    );
    assert!(
        event_names.contains(&"raw".to_string()),
        "应包含 'raw' 事件"
    );
}

/// 测试 7：请求事件路由 — 验证 request 类型事件的层级路由
#[tokio::test]
async fn test_event_router_request_routing() {
    // 创建共享的事件总线
    let bus = Arc::new(EventBus::new(64));
    // 创建事件路由器
    let router = EventRouter::new(bus.clone());
    // 创建订阅者
    let mut sub = bus.subscribe();

    // 构造一条加群请求事件
    let request_event = json!({
        "post_type": "request",
        "request_type": "group",
        "sub_type": "invite",
        "group_id": 123456,
        "user_id": 789012,
        "comment": "请加群"
    });

    // 通过路由器分发事件
    router.route(request_event.clone());

    // 收集所有路由产生的事件名称
    let mut event_names = Vec::new();
    // 请求事件应产生 4 个事件: request, request.group, request.group.invite, raw
    for _ in 0..4 {
        if let Some(event) = sub.recv().await {
            event_names.push(event.name);
        }
    }

    // 验证所有层级的事件名都已生成
    assert!(
        event_names.contains(&"request".to_string()),
        "应包含第一层级 'request' 事件"
    );
    assert!(
        event_names.contains(&"request.group".to_string()),
        "应包含第二层级 'request.group' 事件"
    );
    assert!(
        event_names.contains(&"request.group.invite".to_string()),
        "应包含第三层级 'request.group.invite' 事件"
    );
    assert!(
        event_names.contains(&"raw".to_string()),
        "应包含 'raw' 事件"
    );
}

/// 测试 8：元事件路由 — 验证 meta_event 类型事件的层级路由（含 lifecycle 特殊处理）
#[tokio::test]
async fn test_event_router_meta_event_routing() {
    // 创建共享的事件总线
    let bus = Arc::new(EventBus::new(64));
    // 创建事件路由器
    let router = EventRouter::new(bus.clone());
    // 创建订阅者
    let mut sub = bus.subscribe();

    // 构造一条 lifecycle connect 元事件
    let meta_event = json!({
        "post_type": "meta_event",
        "meta_event_type": "lifecycle",
        "sub_type": "connect"
    });

    // 通过路由器分发事件
    router.route(meta_event.clone());

    // 收集所有路由产生的事件名称
    let mut event_names = Vec::new();
    // lifecycle 元事件应产生 4 个事件:
    // meta_event, meta_event.lifecycle, meta_event.lifecycle.connect, raw
    for _ in 0..4 {
        if let Some(event) = sub.recv().await {
            event_names.push(event.name);
        }
    }

    // 验证所有层级的事件名都已生成
    assert!(
        event_names.contains(&"meta_event".to_string()),
        "应包含第一层级 'meta_event' 事件"
    );
    assert!(
        event_names.contains(&"meta_event.lifecycle".to_string()),
        "应包含第二层级 'meta_event.lifecycle' 事件"
    );
    assert!(
        event_names.contains(&"meta_event.lifecycle.connect".to_string()),
        "应包含第三层级 'meta_event.lifecycle.connect' 事件"
    );
    assert!(
        event_names.contains(&"raw".to_string()),
        "应包含 'raw' 事件"
    );
}

/// 测试 9：始终发送 raw 事件 — 无论事件类型如何，raw 事件始终被发布
#[tokio::test]
async fn test_event_router_always_emits_raw() {
    // 创建共享的事件总线
    let bus = Arc::new(EventBus::new(64));
    // 创建事件路由器
    let router = EventRouter::new(bus.clone());
    // 创建订阅者，专门监听 raw 事件
    let mut sub = bus.subscribe();

    // 构造一条简单的心跳元事件
    let heartbeat = json!({
        "post_type": "meta_event",
        "meta_event_type": "heartbeat",
        "status": {},
        "interval": 5000
    });

    // 通过路由器分发事件
    router.route(heartbeat.clone());

    // 使用精确匹配接收 raw 事件
    let raw_event = sub.recv_exact("raw").await;
    // 断言 raw 事件被成功接收
    assert!(raw_event.is_some(), "应该始终发布 'raw' 事件");
    // 验证 raw 事件携带的数据是原始 JSON
    let event = raw_event.unwrap();
    assert_eq!(event.data, heartbeat);
}

/// 测试 10：未知 post_type — 无法识别的类型应发布 unknown 和 raw 事件
#[tokio::test]
async fn test_event_router_unknown_post_type() {
    // 创建共享的事件总线
    let bus = Arc::new(EventBus::new(64));
    // 创建事件路由器
    let router = EventRouter::new(bus.clone());
    // 创建订阅者
    let mut sub = bus.subscribe();

    // 构造一条具有未知 post_type 的事件
    let unknown_event = json!({
        "post_type": "some_future_type",
        "data": "test"
    });

    // 通过路由器分发事件
    router.route(unknown_event.clone());

    // 收集所有路由产生的事件名称
    let mut event_names = Vec::new();
    // 未知类型应产生 2 个事件: unknown, raw
    for _ in 0..2 {
        if let Some(event) = sub.recv().await {
            event_names.push(event.name);
        }
    }

    // 验证 unknown 和 raw 事件都已发布
    assert!(
        event_names.contains(&"unknown".to_string()),
        "未知类型应发布 'unknown' 事件"
    );
    assert!(
        event_names.contains(&"raw".to_string()),
        "未知类型也应发布 'raw' 事件"
    );
}
