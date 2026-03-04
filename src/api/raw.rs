//! # 原始 API 调用模块
//!
//! 提供底层的原始 API 调用接口，允许用户直接发送自定义 JSON 格式的请求，
//! 适用于以下场景：
//! - 使用尚未在 SDK 中封装的新 API
//! - 调试和测试 API 行为
//! - 发送自定义扩展请求
//!
//! 同时定义了所有已知 NapCat action 名称的常量数组，
//! 方便用户查阅和验证 action 名称的合法性。

// 引入 serde_json 的 json! 宏和 Value 类型，用于处理动态 JSON 数据
use serde_json::{Value, json};

// 引入 API 客户端，用于发送 API 请求
use crate::api::client::ApiClient;

// 引入 SDK 的 Result 类型别名
use crate::error::Result;

/// 所有已知的 NapCat action 名称常量数组
///
/// 包含 OneBot 11 标准 API 和 NapCat 扩展 API 的全部 action 名称。
/// 可用于：
/// - 验证用户输入的 action 名称是否合法
/// - 自动补全和提示
/// - 生成 API 文档
///
/// **注意**：此列表可能不完全包含最新的 NapCat 扩展 API，
/// 建议使用 `RawActionApi::call` 直接调用未列出的 action。
pub const NAPCAT_ACTIONS: &[&str] = &[
    // ====== OneBot 11 标准 API ======
    // 消息相关
    "send_msg",                     // 发送消息（通用）
    "send_private_msg",             // 发送私聊消息
    "send_group_msg",               // 发送群聊消息
    "delete_msg",                   // 撤回消息
    "get_msg",                      // 获取消息
    "get_forward_msg",              // 获取合并转发消息
    "send_group_forward_msg",       // 发送群合并转发消息
    "send_private_forward_msg",     // 发送私聊合并转发消息
    // 表情回应
    "set_msg_emoji_like",           // 设置消息表情回应
    "fetch_emoji_like",             // 获取消息表情回应
    // 已读标记
    "mark_msg_as_read",             // 标记消息为已读
    "mark_group_msg_as_read",       // 标记群消息为已读
    "mark_private_msg_as_read",     // 标记私聊消息为已读
    "_mark_all_as_read",            // 标记所有消息为已读
    // 消息历史
    "get_group_msg_history",        // 获取群消息历史
    "get_friend_msg_history",       // 获取好友消息历史
    "get_recent_contact",           // 获取最近联系人
    // 精华消息
    "set_essence_msg",              // 设置精华消息
    "delete_essence_msg",           // 移除精华消息
    "get_essence_msg_list",         // 获取精华消息列表
    // 戳一戳
    "group_poke",                   // 群戳一戳
    "friend_poke",                  // 好友戳一戳
    "send_poke",                    // 发送戳一戳（通用）
    // 群管理
    "set_group_ban",                // 群禁言
    "set_group_whole_ban",          // 全员禁言
    "set_group_kick",               // 群踢人
    "set_group_leave",              // 退出群组
    "set_group_card",               // 设置群名片
    "set_group_name",               // 设置群名
    "set_group_admin",              // 设置群管理员
    "set_group_anonymous_ban",      // 匿名用户禁言
    "set_group_special_title",      // 设置群专属头衔
    "send_like",                    // 发送好友赞
    // 账号信息
    "get_login_info",               // 获取登录号信息
    "get_status",                   // 获取运行状态
    "get_friend_list",              // 获取好友列表
    "get_group_list",               // 获取群列表
    "get_group_info",               // 获取群信息
    "get_group_member_list",        // 获取群成员列表
    "get_group_member_info",        // 获取群成员信息
    "get_stranger_info",            // 获取陌生人信息
    "get_version_info",             // 获取版本信息
    // 媒体资源
    "get_image",                    // 获取图片信息
    "get_record",                   // 获取语音文件
    "get_file",                     // 获取文件信息
    // 文件操作
    "upload_group_file",            // 上传群文件
    "upload_private_file",          // 上传私聊文件
    "get_group_file_system_info",   // 获取群文件系统信息
    "get_group_root_files",         // 获取群根目录文件列表
    "get_group_files_by_folder",    // 获取群子目录文件列表
    "get_group_file_url",           // 获取群文件下载 URL
    "delete_group_file",            // 删除群文件
    "create_group_file_folder",     // 创建群文件夹
    "delete_group_folder",          // 删除群文件夹
    "download_file",                // 下载文件
    // 流式传输
    "upload_file_stream",           // 流式上传文件
    "download_file_stream",         // 流式下载文件
    "download_file_image_stream",   // 流式下载图片
    "download_file_record_stream",  // 流式下载语音
    // 请求处理
    "set_friend_add_request",       // 处理好友请求
    "set_group_add_request",        // 处理群请求
    // 系统
    "get_online_clients",           // 获取在线客户端列表
    "get_robot_uin_range",          // 获取机器人 QQ 号范围
    "can_send_image",               // 检查是否能发送图片
    "can_send_record",              // 检查是否能发送语音
    "get_cookies",                  // 获取 Cookies
    "get_csrf_token",               // 获取 CSRF Token
    "get_credentials",              // 获取 QQ 相关接口凭证
    "set_input_status",             // 设置输入状态
    "ocr_image",                    // OCR 图片识别
    ".ocr_image",                   // OCR 图片识别（隐藏接口）
    "translate_en2zh",              // 英译中
    "check_url_safely",             // URL 安全检测
    ".handle_quick_operation",      // 快速操作（隐藏接口）
    "nc_get_packet_status",         // 获取 NapCat 封包状态
    "_get_model_show",              // 获取在线机型展示
    "_set_model_show",              // 设置在线机型展示

    // ====== NapCat 扩展 API ======
    // Rkey
    "get_rkey",                     // 获取 Rkey
    "get_rkey_server",              // 获取 Rkey（从服务器）
    "nc_get_rkey",                  // 获取 NapCat Rkey
    // 好友扩展
    "set_friend_remark",            // 设置好友备注
    "delete_friend",                // 删除好友
    "get_unidirectional_friend_list", // 获取单向好友列表
    // 群扩展
    "set_group_remark",             // 设置群备注
    "get_group_info_ex",            // 获取群扩展信息
    "get_group_detail_info",        // 获取群详细信息
    "get_group_ignored_notifies",   // 获取群被忽略通知
    "get_group_shut_list",          // 获取群禁言列表
    // 消息转发
    "forward_friend_single_msg",    // 转发单条好友消息
    "forward_group_single_msg",     // 转发单条群消息
    "send_forward_msg",             // 发送合并转发消息（通用）
    // 群公告
    "_send_group_notice",           // 发送群公告
    "_get_group_notice",            // 获取群公告
    "_del_group_notice",            // 删除群公告
    // 在线状态
    "set_online_status",            // 设置在线状态
    "set_diy_online_status",        // 设置自定义在线状态
    // Ark 分享
    "send_ark_share",               // 发送 Ark 分享
    "send_group_ark_share",         // 发送群 Ark 分享
    "get_mini_app_ark",             // 获取小程序 Ark 消息
    // AI 语音
    "get_ai_characters",            // 获取 AI 语音角色
    "get_ai_record",                // 获取 AI 语音
    "send_group_ai_record",         // 发送群 AI 语音
    // 群签到
    "set_group_sign",               // 群签到
    "send_group_sign",              // 群签到（别名）
    // 杂项
    "fetch_custom_face",            // 获取自定义表情
    "get_emoji_likes",              // 获取表情回应
    "get_clientkey",                // 获取 ClientKey
    "click_inline_keyboard_button", // 点击内联键盘按钮
    // 额外 actions
    ".get_word_slices",             // 获取中文分词
    "ArkShareGroup",                // Ark 分享到群
    "ArkSharePeer",                 // Ark 分享到好友
    "bot_exit",                     // 机器人退出
    "reboot_normal",                // 正常重启
    "clean_cache",                  // 清理缓存
    "reload_event_filter",          // 重载事件过滤器
    "create_collection",            // 创建收藏
    "get_collection_list",          // 获取收藏列表
    "get_doubt_friends_add_request",    // 获取可疑好友请求
    "set_doubt_friends_add_request",    // 处理可疑好友请求
    "get_group_ignore_add_request",     // 获取被忽略的群请求
    "get_group_at_all_remain",      // 获取 @全体成员 剩余次数
    "get_group_system_msg",         // 获取群系统消息
    "get_group_honor_info",         // 获取群荣誉信息
    "get_profile_like",             // 获取个人资料点赞
    "get_qun_album_list",           // 获取群相册列表
    "move_group_file",              // 移动群文件
    "rename_group_file",            // 重命名群文件
    "trans_group_file",             // 转存群文件
    "set_qq_avatar",                // 设置 QQ 头像
    "set_qq_profile",               // 设置 QQ 个人资料
    "set_group_portrait",           // 设置群头像
    "send_packet",                  // 发送原始封包
    "set_group_add_option",         // 设置群加群选项
    "set_group_robot_add_option",   // 设置群机器人加群选项
    "set_group_album_media_like",   // 群相册媒体点赞
    "do_group_album_comment",       // 群相册评论
    "del_group_album_media",        // 删除群相册媒体
    "set_group_search",             // 设置群搜索
    "set_group_todo",               // 设置群待办
    "upload_image_to_qun_album",    // 上传图片到群相册
];

/// 原始 Action API — 直接调用任意 NapCat action
///
/// 当 SDK 中未封装某个 API 时，可以通过此接口直接发送自定义请求。
/// 这是最灵活的 API 调用方式，但需要用户自行构建参数 JSON。
///
/// `ApiClient` 内部使用 `Arc` 共享状态，Clone 成本极低。
#[derive(Clone)]
pub struct RawActionApi {
    /// API 客户端实例（内部通过 Arc 共享状态，Clone 成本低）
    client: ApiClient,
}

impl RawActionApi {
    /// 创建新的原始 API 调用器实例
    ///
    /// # 参数
    ///
    /// - `client`: API 客户端实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的 `RawActionApi` 实例
    pub fn new(client: ApiClient) -> Self {
        // 保存 API 客户端引用
        Self { client }
    }

    /// 调用任意原始 action
    ///
    /// 直接向 NapCatQQ 服务端发送指定 action 名称和参数的 API 请求。
    /// 使用此方法可以调用任何 OneBot 11 标准 API 或 NapCat 扩展 API，
    /// 包括尚未在 SDK 中封装的新 API。
    ///
    /// # 参数
    ///
    /// - `action`: API 动作名称（如 "send_msg"、"get_group_list" 等）
    /// - `params`: API 请求参数的 JSON 值
    ///
    /// # 返回值
    ///
    /// - `Ok(Value)`: API 响应中的 `data` 字段
    /// - `Err(NapLinkError)`: 调用失败（超时、API 返回错误等）
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use serde_json::json;
    /// # use napcat_link::api::raw::RawActionApi;
    /// # async fn example(raw: &RawActionApi) {
    /// // 调用一个自定义 action
    /// let result = raw.call("my_custom_action", json!({
    ///     "key": "value",
    /// })).await;
    /// # }
    /// ```
    pub async fn call(&self, action: &str, params: Value) -> Result<Value> {
        // 委托给 ApiClient::call 发送请求
        self.client.call(action, params).await
    }

    /// 调用无参数的 action
    ///
    /// 便捷方法，自动传入空 JSON 对象 `{}` 作为参数。
    /// 适用于不需要参数的 action，如 `get_login_info`、`get_friend_list` 等。
    ///
    /// # 参数
    ///
    /// - `action`: API 动作名称
    ///
    /// # 返回值
    ///
    /// - `Ok(Value)`: API 响应中的 `data` 字段
    /// - `Err(NapLinkError)`: 调用失败
    pub async fn call_no_params(&self, action: &str) -> Result<Value> {
        // 委托给 ApiClient::call，传入空 JSON 对象
        self.client.call(action, json!({})).await
    }

    /// 获取所有已知 action 名称列表
    ///
    /// 返回 `NAPCAT_ACTIONS` 常量数组的引用，
    /// 包含所有已知的 OneBot 11 标准 API 和 NapCat 扩展 API 名称。
    ///
    /// # 返回值
    ///
    /// 返回静态字符串切片数组的引用
    pub fn known_actions(&self) -> &[&str] {
        // 返回已知 action 名称常量数组
        NAPCAT_ACTIONS
    }
}
