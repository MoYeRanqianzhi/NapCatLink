//! # Echo 机器人 -- 回复收到的每条消息
//!
//! 演示一个简单的回声机器人，将用户发送的文本原样返回。
//! 支持群聊和私聊两种场景，并启用自动重连功能。
//!
//! ## 运行方式
//! ```bash
//! cargo run --example echo_bot
//! ```
//!
//! ## 功能说明
//! - 接收群聊消息 -> 回复 "Echo: <原始消息>"
//! - 接收私聊消息 -> 回复 "Echo: <原始消息>"
//! - 空消息自动跳过
//! - 启用自动重连，最大重连 10 次

// 引入 NapLink 客户端和消息段类型
use napcat_link::{NapLink, MessageSegment};

/// 主函数 -- 异步入口
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化 tracing 日志系统
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    // 创建客户端，启用自动重连并设置最大重连次数为 10
    let client = NapLink::builder("ws://127.0.0.1:3001")
        .token("your_token_here")          // 替换为你的 NapCat 认证 Token
        .reconnect_enabled(true)            // 启用断线自动重连
        .reconnect_max_attempts(10)         // 最多重连 10 次
        .build()?;

    // 在连接之前订阅事件，确保不会丢失连接后的早期事件
    let mut sub = client.subscribe();

    // 克隆 API 引用用于异步任务
    let api = client.api().clone();

    // 在独立的 tokio 任务中运行消息处理循环
    tokio::spawn(async move {
        // 持续监听所有消息事件
        loop {
            // recv_filter("message") 会匹配所有以 "message" 开头的事件
            if let Some(event) = sub.recv_filter("message").await {
                // 获取事件数据引用
                let data = &event.data;

                // 从事件数据中提取原始消息文本（raw_message 字段）
                let raw_message = data.get("raw_message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                // 跳过空消息，避免无意义的回复
                if raw_message.is_empty() { continue; }

                // 构造回复消息：在原始消息前添加 "Echo: " 前缀
                let reply = vec![
                    MessageSegment::text(format!("Echo: {}", raw_message)),
                ];

                // 根据消息类型（群聊/私聊）调用不同的发送 API
                match data.get("message_type").and_then(|v| v.as_str()) {
                    // 群聊消息：从事件数据中获取 group_id 并发送群消息
                    Some("group") => {
                        if let Some(group_id) = data.get("group_id").and_then(|v| v.as_i64()) {
                            // 发送群消息，忽略发送失败的错误
                            let _ = api.message.send_group_message(group_id, reply).await;
                        }
                    }
                    // 私聊消息：从事件数据中获取 user_id 并发送私聊消息
                    Some("private") => {
                        if let Some(user_id) = data.get("user_id").and_then(|v| v.as_i64()) {
                            // 发送私聊消息，忽略发送失败的错误
                            let _ = api.message.send_private_message(user_id, reply).await;
                        }
                    }
                    // 其他类型消息：跳过不处理
                    _ => {}
                }
            }
        }
    });

    // 连接到 NapCat 服务器
    client.connect().await?;
    println!("Echo 机器人已启动！");

    // 等待 Ctrl+C 信号
    tokio::signal::ctrl_c().await?;

    // 断开连接并退出
    client.disconnect();
    println!("Echo 机器人已停止。");

    Ok(())
}
