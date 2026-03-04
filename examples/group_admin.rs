//! # 群管理示例 -- 展示群管理 API 用法
//!
//! 演示 NapCatLink SDK 的账号信息查询 API，包括：
//! - 获取登录账号信息
//! - 获取群列表
//! - 获取好友列表
//! - 获取版本信息
//!
//! ## 运行方式
//! ```bash
//! cargo run --example group_admin
//! ```
//!
//! ## 注意事项
//! - 需要先确保 NapCatQQ 已启动并配置好 WebSocket 端口
//! - 替换代码中的 Token 为实际值

// 引入 NapLink 客户端
use napcat_link::NapLink;

/// 主函数 -- 异步入口
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化 tracing 日志系统
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    // 创建客户端实例
    let client = NapLink::builder("ws://127.0.0.1:3001")
        .token("your_token_here")  // 替换为你的 NapCat 认证 Token
        .build()?;

    // 连接到 NapCat 服务器
    client.connect().await?;
    println!("已连接！");

    // 获取 API 引用
    let api = client.api();

    // ---- 获取登录信息 ----
    // 返回当前登录 QQ 号的 user_id 和 nickname
    let login_info = api.account.get_login_info().await?;
    println!("登录信息: {:?}", login_info);

    // ---- 获取群列表 ----
    // 返回所有已加入的群组信息（group_id, group_name, member_count 等）
    let groups = api.account.get_group_list().await?;
    println!("群列表: {:?}", groups);

    // ---- 获取好友列表 ----
    // 返回所有好友信息（user_id, nickname, remark 等）
    let friends = api.account.get_friend_list().await?;
    println!("好友列表: {:?}", friends);

    // ---- 获取版本信息 ----
    // 返回 NapCat 的版本号和协议版本
    let version = api.account.get_version_info().await?;
    println!("版本: {:?}", version);

    // 断开连接并退出
    client.disconnect();
    println!("已断开连接。");

    Ok(())
}
