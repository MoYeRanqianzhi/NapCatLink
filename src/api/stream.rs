//! # 流式传输 API 模块
//!
//! 封装大文件的流式传输功能，包括：
//! - 流式上传文件（upload_file_stream）
//! - 获取上传流状态（get_upload_stream_status）
//! - 流式下载文件（download_file_stream）
//! - 流式下载文件并保存到本地（download_file_stream_to_file）
//! - 流式下载图片（download_file_image_stream）
//! - 流式下载语音（download_file_record_stream）
//!
//! **注意**：当前版本的流式 API 简化为单次 action 调用，
//! 完整的分块上传/下载逻辑将在后续版本中实现。

// 引入 serde_json 的 json! 宏和 Value 类型，用于构建 API 请求参数
use serde_json::{Value, json};

// 引入 API 客户端，用于发送 API 请求
use crate::api::client::ApiClient;

// 引入 SDK 的 Result 类型别名
use crate::error::Result;

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
    /// 对应 OneBot action: `get_upload_stream_status`
    ///
    /// # 参数
    ///
    /// - `stream_id`: 上传流 ID（由 upload_file_stream 返回）
    ///
    /// # 返回值
    ///
    /// 成功返回上传流状态 JSON（包含 status、progress 等字段）
    pub async fn get_upload_stream_status(&self, stream_id: &str) -> Result<Value> {
        // 调用 get_upload_stream_status action，传入流 ID
        self.client.call("get_upload_stream_status", json!({"stream_id": stream_id})).await
    }

    /// 流式下载文件
    ///
    /// 对应 OneBot action: `download_file_stream`
    ///
    /// **简化版本**：当前不做真正的分块下载，只调用 action 获取文件数据。
    ///
    /// # 参数
    ///
    /// - `file_id`: 文件 ID
    ///
    /// # 返回值
    ///
    /// 成功返回文件数据 JSON（包含 file_data 或 file_path 等字段）
    pub async fn download_file_stream(&self, file_id: &str) -> Result<Value> {
        // 调用 download_file_stream action，传入文件 ID
        self.client.call("download_file_stream", json!({"file_id": file_id})).await
    }

    /// 流式下载文件并保存到本地
    ///
    /// 对应 OneBot action: `download_file_stream`（带 save_path 参数）
    ///
    /// 通过传入 `save_path` 参数，让服务端直接将文件保存到指定路径。
    ///
    /// # 参数
    ///
    /// - `file_id`: 文件 ID
    /// - `path`: 本地保存路径
    ///
    /// # 返回值
    ///
    /// 成功返回下载结果 JSON（包含 file_path 等字段）
    pub async fn download_file_stream_to_file(
        &self,
        file_id: &str,
        path: &str,
    ) -> Result<Value> {
        // 调用 download_file_stream action，传入文件 ID 和本地保存路径
        self.client.call("download_file_stream", json!({
            "file_id": file_id,
            "save_path": path,
        })).await
    }

    /// 流式下载图片
    ///
    /// 对应 OneBot action: `download_file_image_stream`
    ///
    /// # 参数
    ///
    /// - `file_id`: 图片文件 ID
    ///
    /// # 返回值
    ///
    /// 成功返回图片数据 JSON
    pub async fn download_file_image_stream(&self, file_id: &str) -> Result<Value> {
        // 调用 download_file_image_stream action，传入文件 ID
        self.client.call("download_file_image_stream", json!({"file_id": file_id})).await
    }

    /// 流式下载语音
    ///
    /// 对应 OneBot action: `download_file_record_stream`
    ///
    /// # 参数
    ///
    /// - `file_id`: 语音文件 ID
    ///
    /// # 返回值
    ///
    /// 成功返回语音数据 JSON
    pub async fn download_file_record_stream(&self, file_id: &str) -> Result<Value> {
        // 调用 download_file_record_stream action，传入文件 ID
        self.client.call("download_file_record_stream", json!({"file_id": file_id})).await
    }
}
