# NapCatLink 快速入门

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
napcat-link = "0.1.0-alpha"
tokio = { version = "1", features = ["full"] }
```

## 最小示例

```rust
use napcat_link::{NapLink, MessageSegment};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = NapLink::builder("ws://127.0.0.1:3001")
        .token("your_token")
        .build()?;

    client.connect().await?;

    // 发送群消息
    client.api().message.send_group_message(
        123456789,
        vec![MessageSegment::text("Hello!")],
    ).await?;

    Ok(())
}
```

## 监听事件

```rust
let mut sub = client.subscribe();

tokio::spawn(async move {
    loop {
        if let Some(event) = sub.recv_filter("message.group").await {
            println!("群消息: {:?}", event.data);
        }
    }
});
```

## 事件名称格式

事件名称使用点号分隔的层级格式：

| 事件名称 | 说明 |
|----------|------|
| `message` | 所有消息事件 |
| `message.group` | 群消息事件 |
| `message.group.normal` | 普通群消息 |
| `message.private` | 私聊消息事件 |
| `message.private.friend` | 好友私聊消息 |
| `notice` | 所有通知事件 |
| `notice.group_increase` | 群成员增加通知 |
| `request` | 所有请求事件 |
| `request.friend` | 好友添加请求 |
| `meta_event` | 元事件（心跳等） |
| `connect` | WebSocket 连接成功 |
| `disconnect` | WebSocket 断开连接 |
| `reconnecting` | 正在重连 |
| `state_change` | 连接状态变更 |

### 过滤方式

- `recv()` — 接收所有事件
- `recv_filter("message")` — 前缀匹配（匹配 "message"、"message.group" 等）
- `recv_exact("message.group.normal")` — 精确匹配

## 配置选项

```rust
NapLink::builder("ws://127.0.0.1:3001")
    .token("token")                     // 认证 Token
    .connection_timeout_ms(30000)       // 连接超时（默认 30s）
    .ping_interval_ms(30000)            // 心跳间隔（默认 30s）
    .reconnect_enabled(true)            // 启用自动重连（默认 true）
    .reconnect_max_attempts(10)         // 最大重连次数（默认 5）
    .backoff_initial_ms(5000)           // 初始退避时间（默认 5s）
    .backoff_max_ms(60000)              // 最大退避时间（默认 60s）
    .backoff_multiplier(2.0)            // 退避乘数（默认 2.0）
    .api_timeout_ms(15000)              // API 超时（默认 15s）
    .api_retries(2)                     // API 重试次数（默认 2）
    .build()?
```

## 消息段类型

消息通过 `MessageSegment` 枚举构建：

```rust
use napcat_link::MessageSegment;

// 文本
MessageSegment::text("Hello");

// @某人
MessageSegment::at("123456789");

// @全体成员
MessageSegment::at_all();

// 图片
MessageSegment::image("https://example.com/image.png");

// 回复
MessageSegment::reply("message_id");

// 表情
MessageSegment::face("178");

// 语音
MessageSegment::record("https://example.com/voice.amr");

// 视频
MessageSegment::video("https://example.com/video.mp4");

// JSON 卡片
MessageSegment::json(r#"{"app":"com.example"}"#);

// Markdown（NapCat 扩展）
MessageSegment::markdown("# Hello\n**bold**");
```

## 组合消息

```rust
let message = vec![
    MessageSegment::reply("12345"),        // 引用回复
    MessageSegment::at("987654321"),       // @某人
    MessageSegment::text(" 你好！"),        // 文本
    MessageSegment::image("pic.jpg"),      // 图片
];

client.api().message.send_group_message(group_id, message).await?;
```

## 项目结构

```
NapCatLink/
├── Cargo.toml
├── src/
│   ├── lib.rs            # 库入口和 re-exports
│   ├── client.rs         # NapLink 客户端门面
│   ├── config.rs         # 配置和 Builder
│   ├── error.rs          # 错误类型
│   ├── types/            # 数据类型定义
│   │   ├── message.rs    # 消息段
│   │   ├── event/        # 事件类型
│   │   └── api.rs        # API 类型
│   ├── connection/       # 连接层
│   │   ├── actor.rs      # 连接 Actor
│   │   ├── state.rs      # 连接状态
│   │   ├── heartbeat.rs  # 心跳
│   │   └── reconnect.rs  # 重连
│   ├── event/            # 事件系统
│   │   ├── bus.rs        # 事件总线
│   │   └── router.rs     # 事件路由
│   └── api/              # API 层
│       ├── message.rs    # 消息 API
│       ├── group.rs      # 群管理 API
│       ├── account.rs    # 账号 API
│       └── ...           # 其他 API
├── examples/             # 示例代码
├── tests/                # 集成测试
└── docs/                 # 文档
```
