# NapCatLink-rs 设计文档

## 概述

NapCatLink 是一个 Rust 库，用于连接 NapCatQQ 机器人框架。基于 OneBot 11 协议，通过 WebSocket 与 NapCat 服务器通信，提供完整的消息收发、事件监听、API 调用功能。

- **语言版本：** Rust 2024 Edition
- **异步运行时：** Tokio
- **WebSocket：** tokio-tungstenite
- **序列化：** serde + serde_json

## 架构：Actor 模型

### 核心组件

1. **NapLink** — 门面层，用户交互入口
2. **ConnectionActor** — 独立 tokio::task，管理 WebSocket 连接生命周期
3. **ApiClient** — 请求-响应配对，通过 oneshot channel
4. **EventBus** — 基于 tokio::broadcast 的事件分发
5. **EventRouter** — 事件路由，按 post_type 分发到不同事件名
6. **HeartbeatService** — 心跳检测
7. **ReconnectService** — 指数退避重连

### 数据流

```
用户代码 → NapLink::connect()
  → ConnectionActor (独立 task)
    → WebSocket (tokio-tungstenite)
    → HeartbeatService (定时 ping)
    → ReconnectService (指数退避)
    → 收到消息 → Dispatcher
      → echo 存在 → ApiClient response_map (oneshot)
      → 无 echo → EventRouter → EventBus (broadcast)
        → 用户订阅回调
```

### 关键设计决策

- **事件系统：** broadcast channel，过滤在接收端
- **API 请求配对：** DashMap<String, oneshot::Sender> + 唯一 echo
- **消息段：** Rust 枚举 + serde tagged enum
- **错误处理：** thiserror 错误层次
- **配置：** Builder 模式

## 项目结构

```
NapCatLink/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── client.rs              # NapLink 主客户端
│   ├── config.rs              # 配置
│   ├── error.rs               # 错误类型
│   ├── event/                 # 事件系统
│   │   ├── mod.rs
│   │   ├── types.rs
│   │   ├── router.rs
│   │   └── bus.rs
│   ├── connection/            # 连接管理
│   │   ├── mod.rs
│   │   ├── actor.rs
│   │   ├── state.rs
│   │   ├── heartbeat.rs
│   │   └── reconnect.rs
│   ├── api/                   # API 层
│   │   ├── mod.rs
│   │   ├── client.rs
│   │   ├── message.rs
│   │   ├── group.rs
│   │   ├── account.rs
│   │   ├── media.rs
│   │   ├── file.rs
│   │   ├── stream.rs
│   │   ├── request.rs
│   │   ├── system.rs
│   │   ├── napcat.rs
│   │   └── raw.rs
│   ├── types/                 # 类型定义
│   │   ├── mod.rs
│   │   ├── message.rs
│   │   ├── event/
│   │   │   ├── mod.rs
│   │   │   ├── message.rs
│   │   │   ├── notice.rs
│   │   │   ├── request.rs
│   │   │   ├── meta.rs
│   │   │   └── shared.rs
│   │   └── api.rs
│   └── util/
│       ├── mod.rs
│       └── logger.rs
├── examples/
├── tests/
└── docs/
```

## 功能范围

完整复刻 NapLink TypeScript SDK 的所有功能，包括：

- WebSocket 连接、心跳、自动重连
- 完整的 OneBot 11 事件支持
- MessageApi、GroupApi、AccountApi、MediaApi、FileApi
- StreamApi、RequestApi、SystemApi、NapCatApi
- RawActionApi（165+ actions）
- 流式上传/下载（AsyncStream）

## 依赖

- tokio 1.x (full features)
- tokio-tungstenite 0.24+ (native-tls)
- serde 1.x (derive)
- serde_json 1.x
- tracing 0.1
- thiserror 2.x
- url 2.x
- futures-util 0.3
- dashmap 6.x
