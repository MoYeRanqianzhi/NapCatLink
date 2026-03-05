//! # 流式传输 API 模块
//!
//! 封装大文件的流式传输功能，包括：
//! - 流式上传文件（upload_file_stream）
//! - 获取上传流状态（通过 upload_file_stream + verify_only）
//! - 流式下载文件（download_file_stream）
//! - 流式下载文件并保存到本地（download_file_stream_to_file）
//! - 流式下载图片（download_file_image_stream）
//! - 流式下载图片并保存到本地（download_file_image_stream_to_file）
//! - 流式下载语音（download_file_record_stream）
//! - 流式下载语音并保存到本地（download_file_record_stream_to_file）
//! - 清理流式传输临时文件（clean_stream_temp_file）
//!
//! **注意**：当前版本的流式 API 简化为单次 action 调用，
//! 完整的分块上传/下载逻辑将在后续版本中实现。

// 引入 serde_json 的 json! 宏和 Value 类型，用于构建 API 请求参数
use serde_json::{Value, json};

// 引入 API 客户端，用于发送 API 请求
use crate::api::client::ApiClient;

// 引入 SDK 的 Result 类型别名
use crate::error::Result;

/// 默认上传分块大小：256KB（与 TS 版 upload 一致）
const DEFAULT_UPLOAD_CHUNK_SIZE: u64 = 256 * 1024;

/// 默认下载分块大小：64KB（与 TS 版 download 一致）
const DEFAULT_DOWNLOAD_CHUNK_SIZE: u64 = 64 * 1024;

/// 流式 API — 封装文件流式上传和下载操作
///
/// 通过持有 `ApiClient` 的克隆实例来发送请求。
/// `ApiClient` 内部使用 `Arc` 共享状态，Clone 成本极低。
///
/// **简化说明**：当前版本中，所有流式 API 均简化为单次 action 调用。
/// 完整的分块上传/下载、进度回调和断点续传将在后续版本中实现。
#[derive(Clone)]
pub struct StreamApi {
    /// API 客户端实例（内部通过 Arc 共享状态，Clone 成本低）
    client: ApiClient,
}

impl StreamApi {
    /// 创建新的流式传输 API 实例
    ///
    /// # 参数
    ///
    /// - `client`: API 客户端实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的 `StreamApi` 实例
    pub fn new(client: ApiClient) -> Self {
        // 保存 API 客户端引用
        Self { client }
    }

    /// 流式上传文件
    ///
    /// 对应 OneBot action: `upload_file_stream`
    ///
    /// **简化版本**：当前不做真正的分块上传，只调用 action 发送文件信息。
    /// 服务端将根据文件路径直接处理上传。
    ///
    /// # 参数
    ///
    /// - `file`: 本地文件路径
    /// - `params`: 额外上传参数（可选，JSON 对象，可包含 name、folder_id 等字段）
    ///
    /// # 返回值
    ///
    /// 成功返回包含 `stream_id` 或上传结果的 JSON 数据
    pub async fn upload_file_stream(
        &self,
        file: &str,
        params: Option<Value>,
    ) -> Result<Value> {
        // 构建基本请求参数：文件路径
        let mut p = json!({"file": file});
        // 如果提供了额外参数，将其字段合并到请求参数中
        if let Some(Value::Object(map)) = params {
            // 遍历额外参数对象中的每一个键值对
            for (k, v) in map {
                // 将额外参数的字段添加到请求参数中
                p[k] = v;
            }
        }
        // 调用 upload_file_stream action 发送请求
        self.client.call("upload_file_stream", p).await
    }

    /// 获取流式上传状态
    ///
    /// 通过调用 `upload_file_stream` action 并设置 `verify_only: true` 来查询状态
    /// （与 TS 版一致，不是独立的 action）
    ///
    /// # 参数
    ///
    /// - `stream_id`: 上传流 ID（由 upload_file_stream 返回）
    ///
    /// # 返回值
    ///
    /// 成功返回上传流状态 JSON（包含 status、progress 等字段）
    pub async fn get_upload_stream_status(&self, stream_id: &str) -> Result<Value> {
        // 调用 upload_file_stream action（与 TS 一致），带 verify_only 标志
        self.client.call("upload_file_stream", json!({
            "stream_id": stream_id,
            "verify_only": true,
        })).await
    }

    /// 流式下载文件
    ///
    /// 对应 OneBot action: `download_file_stream`
    ///
    /// **简化版本**：当前不做真正的分块下载，只调用 action 获取文件数据。
    ///
    /// # 参数
    ///
    /// - `file`: 文件标识
    /// - `chunk_size`: 分块大小（可选，默认 64KB）
    ///
    /// # 返回值
    ///
    /// 成功返回文件数据 JSON（包含 file_data 或 file_path 等字段）
    pub async fn download_file_stream(
        &self,
        file: &str,
        chunk_size: Option<u64>,
    ) -> Result<Value> {
        // 调用 download_file_stream action，传入文件标识和分块大小
        self.client.call("download_file_stream", json!({
            "file": file,
            "chunk_size": chunk_size.unwrap_or(DEFAULT_DOWNLOAD_CHUNK_SIZE),
        })).await
    }

    /// 流式下载文件并保存到本地
    ///
    /// 对应 OneBot action: `download_file_stream`
    ///
    /// **简化版本**：当前调用 download_file_stream 获取数据，
    /// 完整的客户端分块写入将在后续版本实现。
    ///
    /// # 参数
    ///
    /// - `file`: 文件标识
    /// - `chunk_size`: 分块大小（可选，默认 64KB）
    ///
    /// # 返回值
    ///
    /// 成功返回下载结果 JSON
    pub async fn download_file_stream_to_file(
        &self,
        file: &str,
        chunk_size: Option<u64>,
    ) -> Result<Value> {
        // 调用 download_file_stream action（与 TS 一致，不发送 save_path）
        // 客户端侧保存逻辑将在完整流式实现中添加
        self.client.call("download_file_stream", json!({
            "file": file,
            "chunk_size": chunk_size.unwrap_or(DEFAULT_DOWNLOAD_CHUNK_SIZE),
        })).await
    }

    /// 流式下载图片
    ///
    /// 对应 OneBot action: `download_file_image_stream`
    ///
    /// # 参数
    ///
    /// - `file`: 图片文件标识
    /// - `chunk_size`: 分块大小（可选，默认 64KB）
    ///
    /// # 返回值
    ///
    /// 成功返回图片数据 JSON
    pub async fn download_file_image_stream(
        &self,
        file: &str,
        chunk_size: Option<u64>,
    ) -> Result<Value> {
        // 调用 download_file_image_stream action，传入文件标识和分块大小
        self.client.call("download_file_image_stream", json!({
            "file": file,
            "chunk_size": chunk_size.unwrap_or(DEFAULT_DOWNLOAD_CHUNK_SIZE),
        })).await
    }

    /// 流式下载图片并保存到本地
    ///
    /// 对应 OneBot action: `download_file_image_stream`
    ///
    /// **简化版本**：当前调用 download_file_image_stream 获取数据，
    /// 完整的客户端分块写入将在后续版本实现。
    ///
    /// # 参数
    ///
    /// - `file`: 图片文件标识
    /// - `chunk_size`: 分块大小（可选，默认 64KB）
    ///
    /// # 返回值
    ///
    /// 成功返回图片数据 JSON
    pub async fn download_file_image_stream_to_file(
        &self,
        file: &str,
        chunk_size: Option<u64>,
    ) -> Result<Value> {
        // 调用 download_file_image_stream action
        // 客户端侧保存逻辑将在完整流式实现中添加
        self.client.call("download_file_image_stream", json!({
            "file": file,
            "chunk_size": chunk_size.unwrap_or(DEFAULT_DOWNLOAD_CHUNK_SIZE),
        })).await
    }

    /// 流式下载语音
    ///
    /// 对应 OneBot action: `download_file_record_stream`
    ///
    /// # 参数
    ///
    /// - `file`: 语音文件标识
    /// - `out_format`: 输出格式（可选，如 "mp3"）
    /// - `chunk_size`: 分块大小（可选，默认 64KB）
    ///
    /// # 返回值
    ///
    /// 成功返回语音数据 JSON
    pub async fn download_file_record_stream(
        &self,
        file: &str,
        out_format: Option<&str>,
        chunk_size: Option<u64>,
    ) -> Result<Value> {
        // 构建基本请求参数
        let mut params = json!({
            "file": file,
            "chunk_size": chunk_size.unwrap_or(DEFAULT_DOWNLOAD_CHUNK_SIZE),
        });
        // 如果指定了输出格式，添加到参数中
        if let Some(fmt) = out_format {
            params["out_format"] = json!(fmt);
        }
        // 调用 download_file_record_stream action
        self.client.call("download_file_record_stream", params).await
    }

    /// 流式下载语音并保存到本地
    ///
    /// 对应 OneBot action: `download_file_record_stream`
    ///
    /// **简化版本**：当前调用 download_file_record_stream 获取数据，
    /// 完整的客户端分块写入将在后续版本实现。
    ///
    /// # 参数
    ///
    /// - `file`: 语音文件标识
    /// - `out_format`: 输出格式（可选，如 "mp3"）
    /// - `chunk_size`: 分块大小（可选，默认 64KB）
    ///
    /// # 返回值
    ///
    /// 成功返回语音数据 JSON
    pub async fn download_file_record_stream_to_file(
        &self,
        file: &str,
        out_format: Option<&str>,
        chunk_size: Option<u64>,
    ) -> Result<Value> {
        // 复用 download_file_record_stream 的逻辑
        // 客户端侧保存逻辑将在完整流式实现中添加
        self.download_file_record_stream(file, out_format, chunk_size).await
    }

    /// 清理流式传输临时文件
    ///
    /// 对应 OneBot action: `clean_stream_temp_file`
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn clean_stream_temp_file(&self) -> Result<Value> {
        // 调用 clean_stream_temp_file action，无参数
        self.client.call("clean_stream_temp_file", json!({})).await
    }
}
