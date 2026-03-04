//! # 文件 API 模块
//!
//! 封装文件操作相关的 API 调用，包括：
//! - 上传群文件（upload_group_file）
//! - 上传私聊文件（upload_private_file）
//! - 获取群文件系统信息（get_group_file_system_info）
//! - 获取群根目录文件列表（get_group_root_files）
//! - 获取群子目录文件列表（get_group_files_by_folder）
//! - 获取群文件下载 URL（get_group_file_url）
//! - 删除群文件（delete_group_file）
//! - 创建群文件夹（create_group_file_folder）
//! - 删除群文件夹（delete_group_folder）
//! - 下载文件（download_file）

// 引入 serde_json 的 json! 宏和 Value 类型，用于构建 API 请求参数
use serde_json::{Value, json};

// 引入 API 客户端，用于发送 API 请求
use crate::api::client::ApiClient;

// 引入 SDK 的 Result 类型别名
use crate::error::Result;

/// 文件管理 API — 封装群文件上传、下载、删除等操作
///
/// 通过持有 `ApiClient` 的克隆实例来发送请求。
/// `ApiClient` 内部使用 `Arc` 共享状态，Clone 成本极低。
#[derive(Clone)]
pub struct FileApi {
    /// API 客户端实例（内部通过 Arc 共享状态，Clone 成本低）
    client: ApiClient,
}

impl FileApi {
    /// 创建新的文件 API 实例
    ///
    /// # 参数
    ///
    /// - `client`: API 客户端实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的 `FileApi` 实例
    pub fn new(client: ApiClient) -> Self {
        // 保存 API 客户端引用
        Self { client }
    }

    /// 上传群文件
    ///
    /// 对应 OneBot action: `upload_group_file`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    /// - `file`: 本地文件路径
    /// - `name`: 上传后的文件显示名称
    /// - `folder`: 目标文件夹 ID（可选，None 表示上传到根目录）
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn upload_group_file(
        &self,
        group_id: i64,
        file: &str,
        name: &str,
        folder: Option<&str>,
    ) -> Result<Value> {
        // 构建基本请求参数：群号、文件路径和显示名称
        let mut params = json!({
            "group_id": group_id,
            "file": file,
            "name": name,
        });
        // 如果指定了目标文件夹 ID，添加到参数中
        if let Some(f) = folder {
            // 设置 folder 字段为目标文件夹 ID
            params["folder"] = json!(f);
        }
        // 调用 upload_group_file action 发送请求
        self.client.call("upload_group_file", params).await
    }

    /// 上传私聊文件
    ///
    /// 对应 OneBot action: `upload_private_file`
    ///
    /// # 参数
    ///
    /// - `user_id`: 目标用户 QQ 号
    /// - `file`: 本地文件路径
    /// - `name`: 上传后的文件显示名称
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn upload_private_file(
        &self,
        user_id: i64,
        file: &str,
        name: &str,
    ) -> Result<Value> {
        // 调用 upload_private_file action，传入用户 ID、文件路径和文件名
        self.client.call("upload_private_file", json!({
            "user_id": user_id,
            "file": file,
            "name": name,
        })).await
    }

    /// 获取群文件系统信息
    ///
    /// 对应 OneBot action: `get_group_file_system_info`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    ///
    /// # 返回值
    ///
    /// 成功返回文件系统信息 JSON（包含 file_count、limit_count、used_space、total_space 等）
    pub async fn get_group_file_system_info(&self, group_id: i64) -> Result<Value> {
        // 调用 get_group_file_system_info action，传入群号
        self.client.call("get_group_file_system_info", json!({"group_id": group_id})).await
    }

    /// 获取群根目录文件列表
    ///
    /// 对应 OneBot action: `get_group_root_files`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    ///
    /// # 返回值
    ///
    /// 成功返回根目录文件和文件夹列表 JSON（包含 files 和 folders 数组）
    pub async fn get_group_root_files(&self, group_id: i64) -> Result<Value> {
        // 调用 get_group_root_files action，传入群号
        self.client.call("get_group_root_files", json!({"group_id": group_id})).await
    }

    /// 获取指定文件夹内的文件列表
    ///
    /// 对应 OneBot action: `get_group_files_by_folder`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    /// - `folder_id`: 文件夹 ID
    ///
    /// # 返回值
    ///
    /// 成功返回子目录文件和文件夹列表 JSON（包含 files 和 folders 数组）
    pub async fn get_group_files_by_folder(
        &self,
        group_id: i64,
        folder_id: &str,
    ) -> Result<Value> {
        // 调用 get_group_files_by_folder action，传入群号和文件夹 ID
        self.client.call("get_group_files_by_folder", json!({
            "group_id": group_id,
            "folder_id": folder_id,
        })).await
    }

    /// 获取群文件下载 URL
    ///
    /// 对应 OneBot action: `get_group_file_url`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    /// - `file_id`: 文件 ID
    /// - `busid`: 文件类型 ID（bus ID，从文件信息中获取）
    ///
    /// # 返回值
    ///
    /// 成功返回包含 `url` 字段的 JSON 数据
    pub async fn get_group_file_url(
        &self,
        group_id: i64,
        file_id: &str,
        busid: i32,
    ) -> Result<Value> {
        // 调用 get_group_file_url action，传入群号、文件 ID 和 bus ID
        self.client.call("get_group_file_url", json!({
            "group_id": group_id,
            "file_id": file_id,
            "busid": busid,
        })).await
    }

    /// 删除群文件
    ///
    /// 对应 OneBot action: `delete_group_file`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    /// - `file_id`: 文件 ID
    /// - `busid`: 文件类型 ID（bus ID）
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn delete_group_file(
        &self,
        group_id: i64,
        file_id: &str,
        busid: i32,
    ) -> Result<Value> {
        // 调用 delete_group_file action，传入群号、文件 ID 和 bus ID
        self.client.call("delete_group_file", json!({
            "group_id": group_id,
            "file_id": file_id,
            "busid": busid,
        })).await
    }

    /// 创建群文件夹
    ///
    /// 对应 OneBot action: `create_group_file_folder`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    /// - `name`: 文件夹名称
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn create_group_file_folder(
        &self,
        group_id: i64,
        name: &str,
    ) -> Result<Value> {
        // 调用 create_group_file_folder action，传入群号和文件夹名称
        self.client.call("create_group_file_folder", json!({
            "group_id": group_id,
            "name": name,
        })).await
    }

    /// 删除群文件夹
    ///
    /// 对应 OneBot action: `delete_group_folder`
    ///
    /// # 参数
    ///
    /// - `group_id`: 群号
    /// - `folder_id`: 文件夹 ID
    ///
    /// # 返回值
    ///
    /// 成功返回空数据
    pub async fn delete_group_folder(
        &self,
        group_id: i64,
        folder_id: &str,
    ) -> Result<Value> {
        // 调用 delete_group_folder action，传入群号和文件夹 ID
        self.client.call("delete_group_folder", json!({
            "group_id": group_id,
            "folder_id": folder_id,
        })).await
    }

    /// 下载文件到本地（支持多线程下载）
    ///
    /// 对应 OneBot action: `download_file`
    ///
    /// # 参数
    ///
    /// - `url`: 文件下载 URL
    /// - `thread_count`: 下载线程数（可选，服务端决定默认值）
    /// - `headers`: 自定义 HTTP 请求头（可选，JSON 格式，如 Cookie、User-Agent 等）
    ///
    /// # 返回值
    ///
    /// 成功返回包含 `file` 路径的 JSON 数据
    pub async fn download_file(
        &self,
        url: &str,
        thread_count: Option<i32>,
        headers: Option<Value>,
    ) -> Result<Value> {
        // 构建基本请求参数：文件下载 URL
        let mut params = json!({"url": url});
        // 如果指定了下载线程数，添加到参数中
        if let Some(tc) = thread_count {
            // 设置 thread_count 字段
            params["thread_count"] = json!(tc);
        }
        // 如果指定了自定义请求头，添加到参数中
        if let Some(h) = headers {
            // 设置 headers 字段（JSON 值，支持数组或对象格式）
            params["headers"] = h;
        }
        // 调用 download_file action 发送请求
        self.client.call("download_file", params).await
    }
}
