//! # API 类型模块
//!
//! 定义 OneBot 11 标准 API 和 NapCat 扩展 API 的请求参数和响应体结构，
//! 包括通用的 API 响应包装类型和 API 请求格式。

// 引入 serde 的序列化/反序列化派生宏
use serde::{Deserialize, Serialize};

/// OneBot API 标准响应格式
///
/// 所有 OneBot 11 API 调用的返回值都遵循此格式。
/// 泛型参数 `T` 表示 `data` 字段的具体类型，默认为 `serde_json::Value`。
///
/// ## 响应状态
///
/// - `status = "ok"`：请求处理成功，`data` 中包含响应数据
/// - `status = "async"`：请求已提交异步处理，`data` 可能为空
/// - `status = "failed"`：请求处理失败，`message` 和 `wording` 中包含错误信息
///
/// ## JSON 示例
///
/// ```json
/// {
///     "status": "ok",
///     "retcode": 0,
///     "data": {"message_id": 12345},
///     "echo": "req-001"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T = serde_json::Value> {
    /// 响应状态：
    /// - "ok"：成功
    /// - "async"：异步处理中
    /// - "failed"：失败
    pub status: String,

    /// 返回码：0 表示成功，非 0 表示不同类型的错误
    pub retcode: i64,

    /// 响应数据（泛型类型，根据不同 API 返回不同结构的数据）
    pub data: T,

    /// 回声字段（可选，与请求中的 echo 对应，用于匹配请求和响应）
    #[serde(default)]
    pub echo: Option<String>,

    /// 错误信息（可选，仅在失败时存在，包含技术性的错误描述）
    #[serde(default)]
    pub message: Option<String>,

    /// 错误提示（可选，仅在失败时存在，包含面向用户的友好错误描述）
    #[serde(default)]
    pub wording: Option<String>,
}

/// OneBot API 请求格式
///
/// 通过 WebSocket 发送 API 调用时使用的请求结构。
/// 包含要调用的 API 动作名称、参数和用于匹配响应的回声标识。
///
/// ## JSON 示例
///
/// ```json
/// {
///     "action": "send_group_msg",
///     "params": {"group_id": 100200300, "message": [{"type":"text","data":{"text":"hello"}}]},
///     "echo": "req-001"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRequest {
    /// API 动作名称（如 "send_msg"、"send_group_msg"、"get_group_list" 等）
    pub action: String,

    /// API 请求参数（动态 JSON 值，不同 API 有不同的参数结构）
    pub params: serde_json::Value,

    /// 回声标识（用于在异步场景下将响应与请求进行匹配）
    pub echo: String,
}
