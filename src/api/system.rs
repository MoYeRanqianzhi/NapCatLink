//! # 系统 API 模块
//!
//! 封装系统级别的 API 调用，包括：
//! - 获取在线客户端列表（get_online_clients）
//! - 获取机器人 UIN 范围（get_robot_uin_range）
//! - 检查能否发送图片/语音（can_send_image / can_send_record）
//! - 获取 Cookies 和 CSRF Token（get_cookies / get_csrf_token / get_credentials）
//! - 设置输入状态（set_input_status）
//! - OCR 图片识别（ocr_image）
//! - 英译中（translate_en2zh）
//! - URL 安全检测（check_url_safely）
//! - 快速操作（.handle_quick_operation）
//! - NapCat 封包状态（nc_get_packet_status）

// 引入 serde_json 的 json! 宏和 Value 类型，用于构建 API 请求参数
use serde_json::{Value, json};

// 引入 API 客户端，用于发送 API 请求
use crate::api::client::ApiClient;

// 引入 SDK 的 Result 类型别名
use crate::error::Result;

/// 系统 API — 封装 OCR、翻译、安全检查等系统级功能
///
/// 通过持有 `ApiClient` 的克隆实例来发送请求。
/// `ApiClient` 内部使用 `Arc` 共享状态，Clone 成本极低。
#[derive(Clone)]
pub struct SystemApi {
    /// API 客户端实例（内部通过 Arc 共享状态，Clone 成本低）
    client: ApiClient,
}

impl SystemApi {
    /// 创建新的系统 API 实例
    ///
    /// # 参数
    ///
    /// - `client`: API 客户端实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的 `SystemApi` 实例
    pub fn new(client: ApiClient) -> Self {
        // 保存 API 客户端引用
        Self { client }
    }

    /// 获取当前在线的其他客户端列表
    ///
    /// 对应 OneBot action: `get_online_clients`
    ///
    /// # 参数
    ///
    /// - `no_cache`: 是否不使用缓存（可选，true 表示强制刷新）
    ///
    /// # 返回值
    ///
    /// 成功返回在线客户端列表 JSON（包含设备名称和类型信息）
    pub async fn get_online_clients(&self, no_cache: Option<bool>) -> Result<Value> {
        // 构建基本请求参数（空 JSON 对象）
        let mut params = json!({});
        // 如果指定了 no_cache 选项，添加到参数中
        if let Some(nc) = no_cache {
            params["no_cache"] = json!(nc);
        }
        // 调用 get_online_clients action 发送请求
        self.client.call("get_online_clients", params).await
    }

    /// 获取机器人 QQ 号范围
    ///
    /// 对应 OneBot action: `get_robot_uin_range`（NapCat 扩展）
    ///
    /// # 返回值
    ///
    /// 成功返回机器人 QQ 号范围 JSON
    pub async fn get_robot_uin_range(&self) -> Result<Value> {
        // 调用 get_robot_uin_range action，无参数
        self.client.call("get_robot_uin_range", json!({})).await
    }

    /// 检查是否能发送图片
    ///
    /// 对应 OneBot action: `can_send_image`
    ///
    /// # 返回值
    ///
    /// 成功返回 JSON（包含 `yes` 布尔字段）
    pub async fn can_send_image(&self) -> Result<Value> {
        // 调用 can_send_image action，无参数
        self.client.call("can_send_image", json!({})).await
    }

    /// 检查是否能发送语音
    ///
    /// 对应 OneBot action: `can_send_record`
    ///
    /// # 返回值
    ///
    /// 成功返回 JSON（包含 `yes` 布尔字段）
    pub async fn can_send_record(&self) -> Result<Value> {
        // 调用 can_send_record action，无参数
        self.client.call("can_send_record", json!({})).await
    }

    /// 获取指定域名的 Cookies
    ///
    /// 对应 OneBot action: `get_cookies`
    ///
    /// # 参数
    ///
    /// - `domain`: 目标域名（如 "qzone.qq.com"）
    ///
    /// # 返回值
    ///
    /// 成功返回 JSON（包含 `cookies` 字符串字段）
    pub async fn get_cookies(&self, domain: &str) -> Result<Value> {
        // 调用 get_cookies action，传入域名参数
        self.client.call("get_cookies", json!({"domain": domain})).await
    }

    /// 获取 CSRF Token
    ///
    /// 对应 OneBot action: `get_csrf_token`
    ///
    /// # 返回值
    ///
    /// 成功返回 JSON（包含 `token` 数值字段）
    pub async fn get_csrf_token(&self) -> Result<Value> {
        // 调用 get_csrf_token action，无参数
        self.client.call("get_csrf_token", json!({})).await
    }

    /// 获取 QQ 相关接口凭证
    ///
    /// 对应 OneBot action: `get_credentials`
    ///
    /// 等同于同时获取 Cookies 和 CSRF Token。
    ///
    /// # 参数
    ///
    /// - `domain`: 目标域名
    ///
    /// # 返回值
    ///
    /// 成功返回 JSON（包含 `cookies` 和 `csrf_token` 字段）
    pub async fn get_credentials(&self, domain: &str) -> Result<Value> {
        // 调用 get_credentials action，传入域名参数
        self.client.call("get_credentials", json!({"domain": domain})).await
    }

    /// 设置输入状态
    ///
    /// 对应 OneBot action: `set_input_status`（NapCat 扩展）
    ///
    /// 向对方展示 "正在输入..." 或 "正在说话..." 等状态。
    ///
    /// # 参数
    ///
    /// - `user_id`: 目标用户 QQ 号
    /// - `event_type`: 状态类型数值（如 1 表示正在输入）
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn set_input_status(
        &self,
        user_id: i64,
        event_type: i64,
    ) -> Result<Value> {
        // 调用 set_input_status action，传入用户 ID 和状态类型
        // 同时发送 snake_case (event_type) 和 camelCase (eventType) 两种键名以保持兼容性
        self.client.call("set_input_status", json!({
            "user_id": user_id,
            "event_type": event_type,
            "eventType": event_type,
        })).await
    }

    /// OCR 图片识别
    ///
    /// 对应 OneBot action: `ocr_image` 或 `.ocr_image`
    ///
    /// 当 `dot` 参数为 `true` 时，使用 `.ocr_image` action（带点前缀）；
    /// 当 `dot` 参数为 `false` 或 `None` 时，使用 `ocr_image` action。
    /// 此行为与 TS 版 `ocrImage(image, dot = false)` 一致。
    ///
    /// # 参数
    ///
    /// - `image`: 图片文件标识（file 字段值或 URL）
    /// - `dot`: 是否使用带点前缀的 action（可选，默认 false）
    ///
    /// # 返回值
    ///
    /// 成功返回 OCR 识别结果 JSON（包含 texts 数组等字段）
    pub async fn ocr_image(&self, image: &str, dot: Option<bool>) -> Result<Value> {
        // 根据 dot 参数决定使用哪个 action 名称
        // dot=true 时使用 ".ocr_image"（带点前缀），否则使用 "ocr_image"
        let action = if dot.unwrap_or(false) { ".ocr_image" } else { "ocr_image" };
        // 调用对应的 action，传入图片标识
        self.client.call(action, json!({"image": image})).await
    }

    /// 英文翻译为中文
    ///
    /// 对应 OneBot action: `translate_en2zh`（NapCat 扩展）
    ///
    /// # 参数
    ///
    /// - `words`: 要翻译的英文单词/短语数组切片
    ///
    /// # 返回值
    ///
    /// 成功返回翻译结果 JSON
    pub async fn translate_en2zh(&self, words: &[String]) -> Result<Value> {
        // 调用 translate_en2zh action，传入单词数组
        self.client.call("translate_en2zh", json!({"words": words})).await
    }

    /// 检查 URL 安全性
    ///
    /// 对应 OneBot action: `check_url_safely`（NapCat 扩展）
    ///
    /// # 参数
    ///
    /// - `url`: 要检查的 URL
    ///
    /// # 返回值
    ///
    /// 成功返回 URL 安全等级 JSON
    pub async fn check_url_safely(&self, url: &str) -> Result<Value> {
        // 调用 check_url_safely action，传入 URL
        self.client.call("check_url_safely", json!({"url": url})).await
    }

    /// 快速操作（对事件执行快速响应）
    ///
    /// 对应 OneBot action: `.handle_quick_operation`（隐藏 API，注意 action 前缀为点号）
    ///
    /// 根据事件上下文和操作参数执行快速响应，例如：
    /// - 对消息事件快速回复
    /// - 对好友请求快速同意/拒绝
    /// - 对群请求快速同意/拒绝
    ///
    /// # 参数
    ///
    /// - `context`: 事件上下文（原始事件 JSON）
    /// - `operation`: 操作参数（如 reply、approve 等）
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn handle_quick_operation(
        &self,
        context: Value,
        operation: Value,
    ) -> Result<Value> {
        // 调用 .handle_quick_operation action（注意 action 名前缀为点号）
        self.client.call(".handle_quick_operation", json!({
            "context": context,
            "operation": operation,
        })).await
    }

    /// 获取 NapCat 数据包状态
    ///
    /// 对应 OneBot action: `nc_get_packet_status`（NapCat 扩展）
    ///
    /// # 返回值
    ///
    /// 成功返回封包状态 JSON
    pub async fn nc_get_packet_status(&self) -> Result<Value> {
        // 调用 nc_get_packet_status action，无参数
        self.client.call("nc_get_packet_status", json!({})).await
    }

    /// 获取机型展示信息
    ///
    /// 对应 OneBot action: `_get_model_show`（注意前缀下划线）
    ///
    /// # 参数
    ///
    /// - `model`: 机型标识字符串
    ///
    /// # 返回值
    ///
    /// 成功返回机型展示信息 JSON
    pub async fn get_model_show(&self, model: &str) -> Result<Value> {
        // 调用 _get_model_show action，传入机型标识
        self.client.call("_get_model_show", json!({"model": model})).await
    }

    /// 设置机型展示信息
    ///
    /// 对应 OneBot action: `_set_model_show`（注意前缀下划线）
    ///
    /// # 参数
    ///
    /// - `model`: 机型标识字符串
    /// - `model_show`: 要展示的机型名称
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn set_model_show(&self, model: &str, model_show: &str) -> Result<Value> {
        // 调用 _set_model_show action，传入机型标识和展示名称
        self.client.call("_set_model_show", json!({"model": model, "model_show": model_show})).await
    }
}
