//! # 媒体 API 模块
//!
//! 封装媒体资源相关的 API 调用，包括：
//! - 获取图片信息和 URL（get_image）
//! - 获取语音文件（get_record）
//! - 获取文件信息（get_file）
//! - 填充消息段中的媒体 URL（hydrate_media）

// 引入 serde_json 的 json! 宏和 Value 类型，用于构建 API 请求参数
use serde_json::{Value, json};

// 引入 API 客户端，用于发送 API 请求
use super::client::ApiClient;

// 引入 SDK 的 Result 类型别名
use crate::error::Result;

// 引入消息段类型，用于 hydrate_media 方法
use crate::types::message::MessageSegment;

/// 媒体 API — 封装所有媒体资源相关的 OneBot 11 API 调用
///
/// 通过持有 `ApiClient` 的克隆实例来发送请求。
/// `ApiClient` 内部使用 `Arc` 共享状态，Clone 成本极低。
#[derive(Clone)]
pub struct MediaApi {
    /// API 客户端实例（内部通过 Arc 共享状态，Clone 成本低）
    client: ApiClient,
}

impl MediaApi {
    /// 创建新的媒体 API 实例
    ///
    /// # 参数
    ///
    /// - `client`: API 客户端实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的 `MediaApi` 实例
    pub fn new(client: ApiClient) -> Self {
        // 保存 API 客户端引用
        Self { client }
    }

    /// 获取图片信息
    ///
    /// 对应 OneBot action: `get_image`
    ///
    /// # 参数
    ///
    /// - `file`: 图片文件标识（收到的消息中的 file 字段值）
    ///
    /// # 返回值
    ///
    /// 成功返回图片信息 JSON（包含 size、filename、url 等字段）
    pub async fn get_image(&self, file: &str) -> Result<Value> {
        // 调用 get_image action，传入图片文件标识
        self.client.call("get_image", json!({
            "file": file,
        })).await
    }

    /// 获取语音文件
    ///
    /// 对应 OneBot action: `get_record`
    ///
    /// # 参数
    ///
    /// - `file`: 语音文件标识（收到的消息中的 file 字段值）
    /// - `out_format`: 输出格式（可选，如 "mp3"、"amr"、"wma"、"m4a"、"spx"、"ogg"、"wav"、"flac"）
    ///
    /// # 返回值
    ///
    /// 成功返回语音文件信息 JSON（包含 file 路径等字段）
    pub async fn get_record(&self, file: &str, out_format: Option<&str>) -> Result<Value> {
        // 构建请求参数：语音文件标识
        let mut params = json!({"file": file});
        // 如果指定了输出格式，添加到参数中
        if let Some(fmt) = out_format {
            params["out_format"] = json!(fmt);
        }
        // 调用 get_record action 发送请求
        self.client.call("get_record", params).await
    }

    /// 获取文件信息
    ///
    /// 对应 OneBot action: `get_file`
    ///
    /// # 参数
    ///
    /// - `file`: 文件标识（由其他 API 或消息事件提供）
    ///
    /// # 返回值
    ///
    /// 成功返回文件信息 JSON（包含 name、url、size 等字段）
    pub async fn get_file(&self, file: &str) -> Result<Value> {
        // 调用 get_file action，传入文件标识
        self.client.call("get_file", json!({
            "file": file,
        })).await
    }

    /// 填充消息段中的媒体 URL
    ///
    /// 遍历消息段数组，对 image、record、video、file 类型的消息段，
    /// 调用 API 获取真实下载 URL，并填充到消息段的 url 和 file 字段中。
    ///
    /// 降级策略（与 TS 版一致）：
    /// 1. 优先调用 `get_file` 获取通用文件 URL
    /// 2. 如果 get_file 返回无 URL，则根据类型降级：
    ///    - record/audio: 调用 `get_record(file, "mp3")`
    ///    - image: 调用 `get_image(file)`
    /// 3. 从返回数据中取 `file` 或 `url` 字段作为真实 URL
    ///
    /// 跳过条件：
    /// - file 字段以 `http://`、`https://` 或 `file://` 开头的消息段
    ///
    /// 此方法会原地修改传入的消息段数组。
    /// 如果某个媒体文件获取失败，会跳过该消息段（不中断其他消息段的处理）。
    ///
    /// # 参数
    ///
    /// - `message`: 消息段数组的可变引用
    pub async fn hydrate_media(&self, message: &mut [MessageSegment]) {
        // 遍历消息段数组的每一个元素
        for segment in message.iter_mut() {
            // 根据消息段类型判断是否需要填充 URL
            match segment {
                // 图片消息段
                MessageSegment::Image { file, url, .. } => {
                    // 跳过已经是 URL 或本地文件协议的 file
                    if file.starts_with("http://") || file.starts_with("https://") || file.starts_with("file://") {
                        continue;
                    }
                    // 尝试通过 get_file 获取文件信息
                    if let Ok(data) = self.get_file(file).await {
                        // 从返回数据中优先取 file 字段，其次取 url 字段（与 TS 一致：res?.file ?? res?.url）
                        let hydrated = data.get("file").and_then(|v| v.as_str())
                            .or_else(|| data.get("url").and_then(|v| v.as_str()));
                        if let Some(real_url) = hydrated {
                            // 同时更新 url 和 file 字段（与 TS 一致）
                            *url = Some(real_url.to_string());
                            *file = real_url.to_string();
                            continue;
                        }
                    }
                    // get_file 失败或无 URL 时，降级调用 get_image
                    if let Ok(data) = self.get_image(file).await {
                        let hydrated = data.get("file").and_then(|v| v.as_str())
                            .or_else(|| data.get("url").and_then(|v| v.as_str()));
                        if let Some(real_url) = hydrated {
                            *url = Some(real_url.to_string());
                            *file = real_url.to_string();
                        }
                    }
                }
                // 语音消息段
                MessageSegment::Record { file, url, .. } => {
                    if file.starts_with("http://") || file.starts_with("https://") || file.starts_with("file://") {
                        continue;
                    }
                    // 优先尝试 get_file
                    if let Ok(data) = self.get_file(file).await {
                        let hydrated = data.get("file").and_then(|v| v.as_str())
                            .or_else(|| data.get("url").and_then(|v| v.as_str()));
                        if let Some(real_url) = hydrated {
                            *url = Some(real_url.to_string());
                            *file = real_url.to_string();
                            continue;
                        }
                    }
                    // 降级调用 get_record，使用 mp3 格式（与 TS 一致）
                    if let Ok(data) = self.get_record(file, Some("mp3")).await {
                        let hydrated = data.get("file").and_then(|v| v.as_str())
                            .or_else(|| data.get("url").and_then(|v| v.as_str()));
                        if let Some(real_url) = hydrated {
                            *url = Some(real_url.to_string());
                            *file = real_url.to_string();
                        }
                    }
                }
                // 视频消息段
                MessageSegment::Video { file, url, .. } => {
                    if file.starts_with("http://") || file.starts_with("https://") || file.starts_with("file://") {
                        continue;
                    }
                    // 视频只使用 get_file，无降级
                    if let Ok(data) = self.get_file(file).await {
                        let hydrated = data.get("file").and_then(|v| v.as_str())
                            .or_else(|| data.get("url").and_then(|v| v.as_str()));
                        if let Some(real_url) = hydrated {
                            *url = Some(real_url.to_string());
                            *file = real_url.to_string();
                        }
                    }
                }
                // 文件消息段
                MessageSegment::File { file, url, .. } => {
                    if file.starts_with("http://") || file.starts_with("https://") || file.starts_with("file://") {
                        continue;
                    }
                    // 文件只使用 get_file，无降级
                    if let Ok(data) = self.get_file(file).await {
                        let hydrated = data.get("file").and_then(|v| v.as_str())
                            .or_else(|| data.get("url").and_then(|v| v.as_str()));
                        if let Some(real_url) = hydrated {
                            *url = Some(real_url.to_string());
                            *file = real_url.to_string();
                        }
                    }
                }
                // 其他消息段类型不需要填充媒体 URL，跳过
                _ => {}
            }
        }
    }
}
