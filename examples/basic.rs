//! # 基础示例 -- 连接到 NapCat 并监听消息
//!
//! 演示 NapCatLink SDK 的基本用法，包括：
//! - 创建客户端实例
//! - 订阅事件
//! - 连接到 NapCat 服务器
//! - 接收并处理群消息事件
//!
//! ## 运行方式
//! ```bash
//! cargo run --example basic
//! ```
//!
//! ## 前置条件
//! - NapCatQQ 框架已启动并开放 WebSocket 端口（默认 3001）
//! - 替换代码中的 `your_token_here` 为实际的认证 Token

// 引入 NapLink 客户端和消息段类型
use napcat_link::{NapLink, MessageSegment};

/// 主函数 -- 异步入口
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化 tracing 日志系统，通过环境变量控制日志级别（默认 info）
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    // 使用 Builder 模式创建 NapLink 客户端实例
    let client = NapLink::builder("ws://127.0.0.1:3001")
        .token("your_token_here")  // 替换为你的 NapCat 认证 Token
        .build()?;

    // 订阅事件总线，获取事件订阅句柄（必须在 connect 之前订阅，否则可能丢失早期事件）
    let mut sub = client.subscribe();

    // 克隆 API 引用，以便在异步任务中使用（OneBotApi 实现了 Clone）
    let api = client.api().clone();

    // 在独立的 tokio 任务中持续监听消息事件
    tokio::spawn(async move {
        // 无限循环接收事件
        loop {
            // 使用前缀过滤器接收所有 "message" 开头的事件
            // 包括 "message.group.normal"、"message.private.friend" 等
            if let Some(event) = sub.recv_filter("message").await {
                // 打印接收到的事件名称
                println!("收到消息事件: {}", event.name);

                // 判断是否为群消息事件（事件名以 "message.group" 开头）
                if event.name.starts_with("message.group") {
                    // 从事件数据中提取 group_id 字段
                    if let Some(group_id) = event.data.get("group_id").and_then(|v| v.as_i64()) {
                        // 调用消息 API 向该群发送一条文本回复
                        let _ = api.message.send_group_message(
                            group_id,
                            vec![MessageSegment::text("收到！")],
                        ).await;
                    }
                }
            }
        }
    });

    // 连接到 NapCat WebSocket 服务器（异步阻塞直到连接成功或失败）
    client.connect().await?;

    // 连接成功提示
    println!("NapLink 已连接！按 Ctrl+C 退出。");

    // 等待用户按下 Ctrl+C 信号
    tokio::signal::ctrl_c().await?;

    // 断开 WebSocket 连接并清理资源
    client.disconnect();
    println!("已断开连接。");

    // 程序正常退出
    Ok(())
}
