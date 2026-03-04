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
    /// - `out_format`: 输出格式（如 "mp3"、"amr"、"wma"、"m4a"、"spx"、"ogg"、"wav"、"flac"）
    ///
    /// # 返回值
    ///
    /// 成功返回语音文件信息 JSON（包含 file 路径等字段）
    pub async fn get_record(&self, file: &str, out_format: &str) -> Result<Value> {
        // 调用 get_record action，传入语音文件标识和输出格式
        self.client.call("get_record", json!({
            "file": file,
            "out_format": out_format,
        })).await
    }

    /// 获取文件信息
    ///
    /// 对应 OneBot action: `get_file`
    ///
    /// # 参数
    ///
    /// - `file_id`: 文件 ID（由其他 API 或消息事件提供）
    ///
    /// # 返回值
    ///
    /// 成功返回文件信息 JSON（包含 name、url、size 等字段）
    pub async fn get_file(&self, file_id: &str) -> Result<Value> {
        // 调用 get_file action，传入文件 ID
        self.client.call("get_file", json!({
            "file_id": file_id,
        })).await
    }

    /// 填充消息段中的媒体 URL
    ///
    /// 遍历消息段数组，对 image、record、video、file 类型的消息段，
    /// 调用 `get_file` 获取真实下载 URL，并填充到消息段的 url 字段中。
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
                // 图片消息段：使用 file 字段调用 get_file 获取真实 URL
                MessageSegment::Image { file, url, .. } => {
                    // 尝试通过 get_file 获取文件信息
                    if let Ok(data) = self.get_file(file).await {
                        // 从返回数据中提取 url 字段
                        if let Some(real_url) = data.get("url").and_then(|v| v.as_str()) {
                            // 将真实 URL 填充到消息段的 url 字段
                            *url = Some(real_url.to_string());
                        }
                    }
                }
                // 语音消息段：使用 file 字段调用 get_file 获取真实 URL
                MessageSegment::Record { file, url } => {
                    // 尝试通过 get_file 获取文件信息
                    if let Ok(data) = self.get_file(file).await {
                        // 从返回数据中提取 url 字段
                        if let Some(real_url) = data.get("url").and_then(|v| v.as_str()) {
                            // 将真实 URL 填充到消息段的 url 字段
                            *url = Some(real_url.to_string());
                        }
                    }
                }
                // 视频消息段：使用 file 字段调用 get_file 获取真实 URL
                MessageSegment::Video { file, url } => {
                    // 尝试通过 get_file 获取文件信息
                    if let Ok(data) = self.get_file(file).await {
                        // 从返回数据中提取 url 字段
                        if let Some(real_url) = data.get("url").and_then(|v| v.as_str()) {
                            // 将真实 URL 填充到消息段的 url 字段
                            *url = Some(real_url.to_string());
                        }
                    }
                }
                // 文件消息段：使用 file 字段调用 get_file 获取真实 URL
                MessageSegment::File { file, url, .. } => {
                    // 尝试通过 get_file 获取文件信息
                    if let Ok(data) = self.get_file(file).await {
                        // 从返回数据中提取 url 字段
                        if let Some(real_url) = data.get("url").and_then(|v| v.as_str()) {
                            // 将真实 URL 填充到消息段的 url 字段
                            *url = Some(real_url.to_string());
                        }
                    }
                }
                // 其他消息段类型不需要填充媒体 URL，跳过
                _ => {}
            }
        }
    }
}
