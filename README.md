# NapCatLink

> 现代化的 NapCatQQ WebSocket Rust 客户端 SDK

基于 OneBot 11 协议，通过 WebSocket 连接 NapCatQQ 框架，
提供完整的消息收发、事件监听、API 调用功能。

## 特性

- 基于 Tokio 的全异步架构
- 自动重连与指数退避策略
- 心跳检测机制
- 完整的 OneBot 11 API 支持（100+ action）
- NapCat 扩展 API（Rkey、AI 语音、群公告等）
- 强类型消息段和事件系统
- 流式文件上传/下载
- 零死锁设计（Actor 模型 + channel 通信）

## 快速开始

### 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
napcat-link = "0.1.0-alpha"
tokio = { version = "1", features = ["full"] }
```

### 最小示例

```rust
use napcat_link::{NapLink, MessageSegment};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = NapLink::builder("ws://127.0.0.1:3001")
        .token("your_token")
        .build()?;

    client.connect().await?;

    client.api().message.send_group_message(
        123456789,
        vec![MessageSegment::text("Hello from Rust!")],
    ).await?;

    Ok(())
}
```

### 监听事件

```rust
let mut sub = client.subscribe();

tokio::spawn(async move {
    loop {
        if let Some(event) = sub.recv_filter("message").await {
            println!("收到消息: {:?}", event.data);
        }
    }
});
```

## 示例

- [基础示例](examples/basic.rs) -- 连接并监听消息
- [Echo 机器人](examples/echo_bot.rs) -- 回复收到的每条消息
- [群管理示例](examples/group_admin.rs) -- 查询账号和群信息

运行示例：

```bash
cargo run --example basic
cargo run --example echo_bot
cargo run --example group_admin
```

## API 模块

| 模块 | 访问方式 | 功能 |
|------|----------|------|
| MessageApi | `api.message` | 消息发送、撤回、已读标记、表情回应 |
| GroupApi | `api.group` | 群禁言、踢人、设置群名片/群名 |
| AccountApi | `api.account` | 登录信息、好友/群列表、群成员信息 |
| MediaApi | `api.media` | 图片/语音/文件信息获取 |
| FileApi | `api.file` | 群文件上传下载、文件夹管理 |
| StreamApi | `api.stream` | 大文件流式传输 |
| RequestApi | `api.request` | 好友申请/群邀请处理 |
| SystemApi | `api.system` | OCR、翻译、安全检测、凭证获取 |
| NapCatApi | `api.napcat` | Rkey、群公告、AI 语音、在线状态 |
| RawActionApi | `api.raw` | 调用任意自定义 action |

## 配置

```rust
NapLink::builder("ws://127.0.0.1:3001")
    .token("token")                     // 认证 Token
    .reconnect_enabled(true)            // 自动重连（默认 true）
    .reconnect_max_attempts(10)         // 最大重连次数（默认 10）
    .api_timeout_ms(30000)              // API 超时（默认 30s）
    .ping_interval_ms(30000)            // 心跳间隔（默认 30s）
    .build()?
```

## 致谢

- [NapCatQQ](https://github.com/NapNeko/NapCatQQ) — 基于 NTQQ 的 Bot 框架，提供 OneBot 11 协议实现。本项目的核心功能围绕 NapCatQQ 构建。
- [NapLink](https://github.com/aspect-build/NapLink) — TypeScript 版 NapCatQQ SDK，本项目的参考实现。Rust 版的 API 设计、事件系统和连接管理均对照 NapLink 源码开发。

## 文档

- [快速入门](docs/development/getting-started.md)
- [架构文档](docs/development/architecture.md)
- [API 参考](docs/development/api-reference.md)

生成 Rust API 文档：

```bash
cargo doc --no-deps --open
```

## 许可证

MIT
