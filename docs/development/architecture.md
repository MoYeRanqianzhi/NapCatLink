# NapCatLink 架构文档

## 概述

NapCatLink 是基于 Rust + Tokio 的 NapCatQQ WebSocket 客户端 SDK，
采用 Actor 模型架构，实现与 NapCat 服务器的完整交互。

## 核心架构

### Actor 模型

```
NapLink (门面层)
├── ConnectionActor (独立 tokio::task)
│   ├── WebSocket 连接 (tokio-tungstenite)
│   ├── HeartbeatService (心跳检测)
│   └── ReconnectService (指数退避重连)
├── Dispatcher (消息路由)
│   ├── ApiClient (请求-响应配对)
│   └── EventRouter → EventBus (事件广播)
└── OneBotApi (业务 API 聚合)
    ├── MessageApi, GroupApi, AccountApi
    ├── MediaApi, FileApi, StreamApi
    ├── RequestApi, SystemApi, NapCatApi
    └── RawActionApi
```

### 数据流

1. 用户调用 `client.connect()` → ConnectionActor 建立 WebSocket
2. 服务器推送事件 → ConnectionActor → Dispatcher → EventRouter → EventBus → 用户订阅
3. 用户调用 API → ApiClient 生成 echo → WebSocket 发送 → 响应通过 echo 配对 → 返回结果

### 关键设计

- **零死锁**: 组件间通过 channel 通信，无共享可变状态
- **类型安全**: 消息段使用 Rust 枚举 + serde 自定义序列化
- **优雅降级**: 未知消息段/事件类型自动回退到 Unknown 变体
- **自动重连**: 指数退避策略，可配置最大重连次数

## 模块说明

| 模块 | 文件 | 职责 |
|------|------|------|
| config | `src/config.rs` | 配置类型和 Builder 模式 |
| error | `src/error.rs` | 错误层次 (thiserror) |
| types/message | `src/types/message.rs` | 消息段枚举和序列化 |
| types/event | `src/types/event/` | 事件类型定义 |
| types/api | `src/types/api.rs` | API 请求/响应类型 |
| connection/state | `src/connection/state.rs` | 连接状态枚举 |
| connection/heartbeat | `src/connection/heartbeat.rs` | 心跳检测服务 |
| connection/reconnect | `src/connection/reconnect.rs` | 指数退避重连策略 |
| connection/actor | `src/connection/actor.rs` | WebSocket 连接 Actor |
| event/bus | `src/event/bus.rs` | broadcast channel 事件总线 |
| event/router | `src/event/router.rs` | 事件名称路由 |
| api/client | `src/api/client.rs` | API 请求发送和 echo 配对 |
| api/dispatcher | `src/api/dispatcher.rs` | 消息分发器 |
| api/message | `src/api/message.rs` | 消息相关 API |
| api/group | `src/api/group.rs` | 群管理 API |
| api/account | `src/api/account.rs` | 账号信息 API |
| api/media | `src/api/media.rs` | 媒体资源 API |
| api/file | `src/api/file.rs` | 文件管理 API |
| api/stream | `src/api/stream.rs` | 流式传输 API |
| api/request | `src/api/request.rs` | 请求处理 API |
| api/system | `src/api/system.rs` | 系统功能 API |
| api/napcat | `src/api/napcat.rs` | NapCat 扩展 API |
| api/raw | `src/api/raw.rs` | 原始 action 调用 |
| client | `src/client.rs` | 客户端门面 |

## 依赖关系图

```
client.rs (NapLink)
    ├── config.rs (NapLinkConfig)
    ├── error.rs (NapLinkError, Result)
    ├── connection/
    │   ├── actor.rs (ConnectionActor)
    │   │   ├── state.rs (ConnectionState)
    │   │   ├── heartbeat.rs (HeartbeatService)
    │   │   └── reconnect.rs (ReconnectService)
    │   └── mod.rs (ConnectionHandle, ConnectionNotification)
    ├── event/
    │   ├── bus.rs (EventBus, EventSubscription, EventData)
    │   └── router.rs (EventRouter)
    └── api/
        ├── client.rs (ApiClient)
        ├── dispatcher.rs (Dispatcher)
        └── [10 个 API 模块]
```

## 线程模型

- **主线程**: 用户代码运行
- **ConnectionActor task**: 独立 tokio task，管理 WebSocket 连接生命周期
- **通知处理 task**: 独立 tokio task，从 ConnectionActor 接收通知并分发到事件总线
- **用户事件处理 task**: 用户可以 spawn 任意数量的事件处理任务

所有跨线程通信通过 `tokio::sync::mpsc` 和 `tokio::sync::broadcast` channel 实现，无锁设计。

## 错误处理

SDK 使用 `thiserror` 定义统一的错误枚举 `NapLinkError`，包括：

- `ConnectionError`: WebSocket 连接相关错误
- `ApiError`: API 调用失败（包含服务端返回的错误码和消息）
- `TimeoutError`: 操作超时
- `SerializationError`: JSON 序列化/反序列化错误
- `InvalidConfig`: 配置验证失败
