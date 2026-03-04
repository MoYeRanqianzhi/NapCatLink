//! # API 调用模块
//!
//! 封装 OneBot 11 标准 API 和 NapCat 扩展 API 的调用接口，
//! 分为以下子模块：
//! - `client`：API 客户端，封装请求发送和响应接收
//! - `dispatcher`：消息分发器，区分 API 响应和事件，分别路由
//! - `message`：消息相关 API（发送消息、撤回消息等）
//! - `group`：群组相关 API（群管理、群成员操作等）
//! - `account`：账号相关 API（获取登录信息、好友列表等）
//! - `media`：媒体相关 API（图片、语音、视频处理）
//! - `file`：文件相关 API（文件上传、下载）
//! - `stream`：流式传输 API（大文件分片上传/下载）
//! - `request`：请求处理 API（处理好友申请、群邀请等）
//! - `system`：系统相关 API（获取版本信息、重启等）
//! - `napcat`：NapCat 扩展 API（NapCat 特有的增强功能）
//! - `raw`：原始 API 调用接口（直接发送自定义 JSON 请求）
//!
//! # 统一入口
//!
//! 所有 API 模块通过 `OneBotApi` 聚合器统一访问：
//! ```rust,no_run
//! # use napcat_link::api::OneBotApi;
//! # use napcat_link::api::client::ApiClient;
//! # fn example(client: ApiClient) {
//! let api = OneBotApi::new(client);
//! // api.message.send_group_message(...)
//! // api.group.set_group_ban(...)
//! // api.account.get_login_info()
//! // api.raw.call("custom_action", ...)
//! # }
//! ```

// API 客户端子模块：封装底层请求发送逻辑和请求-响应配对机制
pub mod client;

// 消息分发器子模块：区分 API 响应和事件推送，分别路由到对应处理器
pub mod dispatcher;

// 消息 API 子模块：发送私聊/群聊消息、撤回消息、获取消息等
pub mod message;

// 群组 API 子模块：群信息查询、群成员管理、群设置等
pub mod group;

// 账号 API 子模块：登录信息、好友列表、陌生人信息等
pub mod account;

// 媒体 API 子模块：图片、语音、视频等媒体资源处理
pub mod media;

// 文件 API 子模块：文件上传和下载操作
pub mod file;

// 流式传输 API 子模块：大文件分片传输
pub mod stream;

// 请求处理 API 子模块：处理好友添加请求、群邀请请求等
pub mod request;

// 系统 API 子模块：获取运行状态、版本信息、重启服务等
pub mod system;

// NapCat 扩展 API 子模块：NapCat 框架特有的增强功能接口
pub mod napcat;

// 原始 API 子模块：支持直接发送自定义 JSON 格式的 API 请求
pub mod raw;

// 重导出核心类型，方便外部直接通过 `napcat_link::api::ApiClient` 等路径访问
pub use client::ApiClient;
pub use dispatcher::Dispatcher;

// 重导出各 API 模块的结构体类型
pub use message::MessageApi;
pub use group::GroupApi;
pub use account::AccountApi;
pub use media::MediaApi;
pub use file::FileApi;
pub use stream::StreamApi;
pub use request::RequestApi;
pub use system::SystemApi;
pub use napcat::NapCatApi;
pub use raw::RawActionApi;

/// OneBot API 聚合器 — 提供对所有 API 模块的统一访问入口
///
/// 将 10 个 API 模块整合到一个结构体中，通过公开字段直接访问各模块。
/// 每个字段都是对应 API 模块的实例，它们共享同一个 `ApiClient`（通过 Clone）。
///
/// # 使用方式
///
/// ```rust,no_run
/// # use napcat_link::api::OneBotApi;
/// # use napcat_link::api::client::ApiClient;
/// # async fn example(client: ApiClient) {
/// let api = OneBotApi::new(client);
///
/// // 发送群消息
/// // api.message.send_group_message(123456, vec![...]).await;
///
/// // 获取群成员列表
/// // api.account.get_group_member_list(123456).await;
///
/// // 调用自定义 action
/// // api.raw.call("custom_action", serde_json::json!({})).await;
/// # }
/// ```
pub struct OneBotApi {
    /// 消息 API — 消息发送、撤回、获取、已读标记、表情回应、戳一戳等
    pub message: MessageApi,
    /// 群组 API — 群禁言、踢人、设置群名片/群名/管理员、点赞等
    pub group: GroupApi,
    /// 账号 API — 登录信息、好友列表、群列表、群成员信息、版本信息等
    pub account: AccountApi,
    /// 媒体 API — 图片/语音/文件信息获取、消息段媒体 URL 填充
    pub media: MediaApi,
    /// 文件 API — 群文件/私聊文件上传下载、文件夹管理等
    pub file: FileApi,
    /// 流式传输 API — 大文件流式上传/下载（简化版）
    pub stream: StreamApi,
    /// 请求处理 API — 处理好友申请和群邀请请求
    pub request: RequestApi,
    /// 系统 API — 在线客户端、凭证获取、OCR、翻译、安全检查等
    pub system: SystemApi,
    /// NapCat 扩展 API — Rkey、好友扩展、群扩展、公告、AI 语音等
    pub napcat: NapCatApi,
    /// 原始 API — 支持调用任意 OneBot action
    pub raw: RawActionApi,
}

impl OneBotApi {
    /// 创建 OneBot API 聚合器实例
    ///
    /// 接收一个 `ApiClient` 实例，为每个 API 模块创建独立的实例。
    /// `ApiClient` 内部使用 `Arc` 共享状态，Clone 成本极低。
    ///
    /// # 参数
    ///
    /// - `client`: API 客户端实例
    ///
    /// # 返回值
    ///
    /// 返回一个包含所有 API 模块的 `OneBotApi` 实例
    pub fn new(client: ApiClient) -> Self {
        Self {
            // 为消息 API 模块创建实例（Clone ApiClient）
            message: MessageApi::new(client.clone()),
            // 为群组 API 模块创建实例（Clone ApiClient）
            group: GroupApi::new(client.clone()),
            // 为账号 API 模块创建实例（Clone ApiClient）
            account: AccountApi::new(client.clone()),
            // 为媒体 API 模块创建实例（Clone ApiClient）
            media: MediaApi::new(client.clone()),
            // 为文件 API 模块创建实例（Clone ApiClient）
            file: FileApi::new(client.clone()),
            // 为流式传输 API 模块创建实例（Clone ApiClient）
            stream: StreamApi::new(client.clone()),
            // 为请求处理 API 模块创建实例（Clone ApiClient）
            request: RequestApi::new(client.clone()),
            // 为系统 API 模块创建实例（Clone ApiClient）
            system: SystemApi::new(client.clone()),
            // 为 NapCat 扩展 API 模块创建实例（Clone ApiClient）
            napcat: NapCatApi::new(client.clone()),
            // 为原始 API 模块创建实例（最后一个不需要 Clone）
            raw: RawActionApi::new(client),
        }
    }
}
