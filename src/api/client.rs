//! # API 客户端模块
//!
//! 封装底层的 API 请求发送逻辑，包括：
//! - 构建 OneBot 11 格式的 API 请求 JSON
//! - 通过 WebSocket 发送请求
//! - 等待和接收 API 响应（基于 echo 标识匹配请求-响应对）
//! - 请求超时处理与自动重试

// 引入 Arc 智能指针，用于在多个异步任务之间共享数据
use std::sync::Arc;

// 引入 AtomicU64 原子计数器，用于生成唯一的请求 ID（线程安全）
use std::sync::atomic::{AtomicU64, Ordering};

// 引入 DashMap 并发哈希映射，用于存储待处理的请求（echo -> PendingRequest）
use dashmap::DashMap;

// 引入 serde_json::Value，用于处理动态 JSON 数据
use serde_json::Value;

// 引入 tokio oneshot channel，用于单次请求-响应配对
use tokio::sync::oneshot;

// 引入 tokio 的时间工具，用于超时控制和重试延迟
use tokio::time::{Duration, timeout};

// 引入连接句柄类型，用于通过 WebSocket 发送消息
use crate::connection::ConnectionHandle;

// 引入 SDK 配置类型
use crate::config::NapLinkConfig;

// 引入 SDK 的错误类型和 Result 类型别名
use crate::error::{NapLinkError, Result};

// 引入 API 请求结构体，用于构建 OneBot 11 格式的请求 JSON
use crate::types::api::ApiRequest;

/// 待处理请求信息 — 存储每个已发送但尚未收到响应的 API 请求的上下文
///
/// 当请求发送后，会在 pending map 中注册一个 PendingRequest，
/// 直到收到响应或超时后才被移除。
struct PendingRequest {
    /// oneshot channel 的发送端，用于将 API 响应传回给等待的调用者
    tx: oneshot::Sender<Result<Value>>,
    /// API 方法名（例如 "send_msg"、"get_group_list"），用于错误报告
    method: String,
    /// 请求创建时的 UNIX 时间戳（毫秒），用于清理过期请求
    created_at: u64,
}

/// API 客户端 — 管理 API 请求的发送、响应配对和超时控制
///
/// 核心机制：
/// 1. 每个请求生成唯一 echo ID
/// 2. 通过 `DashMap<String, PendingRequest>` 管理待处理请求
/// 3. 超时控制用 `tokio::time::timeout`
/// 4. 支持重试（仅重试 ApiTimeout 和 Api 错误）
///
/// # 线程安全
///
/// `ApiClient` 实现了 `Clone`，内部所有字段通过 `Arc` 共享，
/// 可以安全地在多个 tokio task 中使用。
#[derive(Clone)]
pub struct ApiClient {
    /// 连接句柄（用于通过 WebSocket 发送 JSON 消息）
    connection: ConnectionHandle,
    /// SDK 配置的共享引用（包含超时、重试等参数）
    config: Arc<NapLinkConfig>,
    /// 待处理请求映射表（echo ID -> PendingRequest），使用并发安全的 DashMap
    pending: Arc<DashMap<String, PendingRequest>>,
    /// 请求 ID 计数器（原子递增），保证每个请求的 echo 唯一
    request_counter: Arc<AtomicU64>,
}

impl ApiClient {
    /// 创建新的 API 客户端实例
    ///
    /// # 参数
    ///
    /// - `connection`: WebSocket 连接句柄，用于发送 API 请求
    /// - `config`: SDK 配置的 Arc 共享引用
    ///
    /// # 返回值
    ///
    /// 返回一个新的 `ApiClient` 实例
    pub fn new(connection: ConnectionHandle, config: Arc<NapLinkConfig>) -> Self {
        Self {
            // 保存连接句柄
            connection,
            // 保存配置引用
            config,
            // 创建空的并发映射表
            pending: Arc::new(DashMap::new()),
            // 初始化计数器为 0
            request_counter: Arc::new(AtomicU64::new(0)),
        }
    }

    /// 调用 API — 使用默认超时和重试配置
    ///
    /// 这是最常用的 API 调用方法，使用配置文件中的默认超时和重试次数。
    ///
    /// # 参数
    ///
    /// - `method`: API 动作名称（如 "send_msg"、"get_group_list"）
    /// - `params`: API 请求参数的 JSON 值
    ///
    /// # 返回值
    ///
    /// - `Ok(Value)`: API 响应中的 `data` 字段
    /// - `Err(NapLinkError)`: 调用失败（超时、API 返回错误等）
    pub async fn call(&self, method: &str, params: Value) -> Result<Value> {
        // 委托给 call_with_options，传入 None 使用默认配置
        self.call_with_options(method, params, None, None).await
    }

    /// 带选项的 API 调用 — 支持自定义超时和重试次数
    ///
    /// # 参数
    ///
    /// - `method`: API 动作名称
    /// - `params`: API 请求参数的 JSON 值
    /// - `timeout_ms`: 自定义超时时间（毫秒），None 则使用配置默认值
    /// - `retries`: 自定义重试次数，None 则使用配置默认值
    ///
    /// # 返回值
    ///
    /// - `Ok(Value)`: API 响应中的 `data` 字段
    /// - `Err(NapLinkError)`: 调用失败
    ///
    /// # 重试策略
    ///
    /// 仅重试以下两种错误类型：
    /// - `NapLinkError::ApiTimeout`: API 调用超时
    /// - `NapLinkError::Api`: API 返回错误状态
    ///
    /// 其他错误（如连接断开、JSON 序列化失败）会立即返回，不进行重试。
    pub async fn call_with_options(
        &self,
        method: &str,
        params: Value,
        timeout_ms: Option<u64>,
        retries: Option<u32>,
    ) -> Result<Value> {
        // 使用自定义超时或配置默认超时
        let timeout_ms = timeout_ms.unwrap_or(self.config.api.timeout_ms);
        // 使用自定义重试次数或配置默认重试次数
        let retries = retries.unwrap_or(self.config.api.retries);

        // 保存最后一次遇到的错误，用于在所有重试耗尽后返回
        let mut last_error = None;

        // 重试循环：attempt 从 0 开始，0 是首次尝试，1..=retries 是重试
        for attempt in 0..=retries {
            // 如果不是首次尝试（attempt > 0），先等待退避延迟
            if attempt > 0 {
                // 重试延迟计算：min(1000ms * 重试次数, 5000ms)，避免过长等待
                let delay = std::cmp::min(1000 * attempt as u64, 5000);
                // 异步等待退避延迟
                tokio::time::sleep(Duration::from_millis(delay)).await;
                // 记录重试日志
                tracing::debug!("重试 API 调用 {}: 第 {} 次", method, attempt);
            }

            // 发送单次请求（不含重试逻辑）
            match self.send_request(method, &params, timeout_ms).await {
                // 请求成功，直接返回响应数据
                Ok(data) => return Ok(data),
                // 请求失败，根据错误类型判断是否重试
                Err(e) => {
                    match &e {
                        // ApiTimeout 和 Api 错误可以重试
                        NapLinkError::ApiTimeout { .. } | NapLinkError::Api { .. } => {
                            // 保存错误并继续下一次重试
                            last_error = Some(e);
                            continue;
                        }
                        // 其他错误（连接断开、序列化失败等）不可重试，立即返回
                        _ => return Err(e),
                    }
                }
            }
        }

        // 所有重试耗尽，返回最后一次错误
        // unwrap 安全：至少执行了一次循环，last_error 一定有值
        Err(last_error.unwrap())
    }

    /// 发送单次 API 请求（不含重试逻辑）
    ///
    /// 内部方法，执行以下步骤：
    /// 1. 生成唯一 echo ID
    /// 2. 创建 oneshot channel 用于接收响应
    /// 3. 注册到 pending map
    /// 4. 构建并发送 API 请求 JSON
    /// 5. 在超时时间内等待响应
    ///
    /// # 参数
    ///
    /// - `method`: API 动作名称
    /// - `params`: API 请求参数的 JSON 引用
    /// - `timeout_ms`: 超时时间（毫秒）
    ///
    /// # 返回值
    ///
    /// - `Ok(Value)`: API 响应中的 `data` 字段
    /// - `Err(NapLinkError)`: 发送失败、超时或 API 返回错误
    async fn send_request(&self, method: &str, params: &Value, timeout_ms: u64) -> Result<Value> {
        // 步骤 1：生成唯一的 echo 标识
        let echo = self.generate_echo();

        // 步骤 2：创建 oneshot channel — tx 注册到 pending map，rx 用于等待响应
        let (tx, rx) = oneshot::channel();

        // 步骤 3：将请求注册到 pending map，等待 handle_response 来匹配响应
        self.pending.insert(echo.clone(), PendingRequest {
            // oneshot 发送端，响应到达时通过此 channel 通知调用者
            tx,
            // API 方法名，用于错误报告
            method: method.to_string(),
            // 当前时间戳，用于超时清理
            created_at: now_ms(),
        });

        // 步骤 4：构建 OneBot 11 格式的 API 请求 JSON
        let request = ApiRequest {
            // API 动作名称
            action: method.to_string(),
            // API 参数（克隆一份，因为可能需要在重试时再次使用）
            params: params.clone(),
            // echo 标识，服务端会在响应中回传此字段
            echo: echo.clone(),
        };

        // 将请求结构体序列化为 JSON 字符串
        let payload = serde_json::to_string(&request)?;

        // 通过 WebSocket 连接句柄发送 JSON 字符串
        if let Err(e) = self.connection.send(payload) {
            // 发送失败：从 pending map 中移除已注册的请求
            self.pending.remove(&echo);
            // 返回发送错误
            return Err(e);
        }

        // 步骤 5：在超时时间内异步等待 oneshot channel 的响应
        match timeout(Duration::from_millis(timeout_ms), rx).await {
            // 在超时前收到了响应
            Ok(Ok(result)) => {
                // result 是 Result<Value>，直接返回
                result
            }
            // oneshot sender 被 drop（不应在正常流程中发生）
            Ok(Err(_)) => {
                // channel 异常关闭，可能是 ApiClient 被 destroy 了
                Err(NapLinkError::Connection("响应 channel 异常关闭".into()))
            }
            // 超时未收到响应
            Err(_) => {
                // 从 pending map 中移除超时的请求（避免泄漏）
                self.pending.remove(&echo);
                // 返回 ApiTimeout 错误，携带方法名和超时时间
                Err(NapLinkError::ApiTimeout {
                    method: method.to_string(),
                    timeout_ms,
                })
            }
        }
    }

    /// 处理 API 响应 — 由 Dispatcher 调用
    ///
    /// 当 Dispatcher 收到一条带有 echo 字段的 WebSocket 消息时，
    /// 调用此方法将响应路由到对应的等待者。
    ///
    /// # 参数
    ///
    /// - `echo`: 响应中的 echo 字段值，用于匹配请求
    /// - `response`: 完整的 API 响应 JSON
    ///
    /// # 响应格式
    ///
    /// 成功响应：`{"status": "ok", "retcode": 0, "data": {...}, "echo": "xxx"}`
    /// 失败响应：`{"status": "failed", "retcode": -1, "message": "...", "echo": "xxx"}`
    pub fn handle_response(&self, echo: &str, response: &Value) {
        // 从 pending map 中取出匹配 echo 的请求（同时移除）
        if let Some((_, pending)) = self.pending.remove(echo) {
            // 提取响应中的 status 字段，缺失时使用空字符串
            let status = response.get("status").and_then(|v| v.as_str()).unwrap_or("");
            // 提取响应中的 retcode 字段，缺失时使用 -1（表示未知错误）
            let retcode = response.get("retcode").and_then(|v| v.as_i64()).unwrap_or(-1);

            // 判断响应是否成功：status 为 "ok" 或 retcode 为 0
            if status == "ok" || retcode == 0 {
                // 成功响应：提取 data 字段，缺失时使用 Value::Null
                let data = response.get("data").cloned().unwrap_or(Value::Null);
                // 通过 oneshot channel 将成功结果发送给等待者
                let _ = pending.tx.send(Ok(data));
            } else {
                // 失败响应：提取错误信息字段
                let message = response
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                // 提取用户友好的错误描述（可选字段）
                let wording = response
                    .get("wording")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                // 通过 oneshot channel 将错误结果发送给等待者
                let _ = pending.tx.send(Err(NapLinkError::Api {
                    // 原始请求的 API 方法名
                    method: pending.method,
                    // 服务端返回的错误码
                    retcode,
                    // 服务端返回的原始错误消息
                    message,
                    // 服务端返回的用户友好错误描述
                    wording,
                }));
            }
        } else {
            // pending map 中找不到对应 echo 的请求
            // 可能原因：请求已超时被清理、重复响应、外部发送的消息等
            tracing::warn!("收到未知请求的响应: echo={}", echo);
        }
    }

    /// 生成唯一请求 ID（echo 标识）
    ///
    /// 格式: `naplink_{时间戳毫秒}_{递增计数器}`
    /// 通过时间戳 + 原子计数器的组合保证全局唯一性。
    ///
    /// # 返回值
    ///
    /// 返回唯一的 echo 字符串
    fn generate_echo(&self) -> String {
        // 原子递增计数器，使用 Relaxed 排序（无需严格顺序保证，仅需唯一性）
        let counter = self.request_counter.fetch_add(1, Ordering::Relaxed);
        // 获取当前 UNIX 时间戳（毫秒）
        let ts = now_ms();
        // 拼接为唯一的 echo 字符串
        format!("naplink_{}_{}", ts, counter)
    }

    /// 销毁客户端 — 清理所有待处理请求
    ///
    /// 当 SDK 关闭或连接彻底断开时调用。
    /// 清空 pending map，所有还在等待响应的调用者会收到 channel 关闭错误（RecvError）。
    pub fn destroy(&self) {
        // 使用 retain 返回 false 来清空所有条目
        // 当 PendingRequest 的 tx 被 drop 后，对应的 rx 会收到 RecvError
        self.pending.retain(|_echo, _pending| {
            // 返回 false 表示不保留此条目，DashMap 会移除并 drop 它
            false
        });
    }

    /// 清理超时的待处理请求 — 防止内存泄漏
    ///
    /// 定期调用此方法可以清理那些因异常原因（如 timeout 竞态）
    /// 未被正常移除的过期请求。
    ///
    /// 清理阈值为配置超时时间的 2 倍，给予足够的容忍度。
    pub fn cleanup_stale(&self) {
        // 获取当前 UNIX 时间戳（毫秒）
        let now = now_ms();
        // 计算最大允许存活时间（配置超时的 2 倍）
        let max_age = self.config.api.timeout_ms * 2;
        // 遍历 pending map，移除超过最大存活时间的请求
        self.pending.retain(|echo, pending| {
            // 检查请求是否已超过最大存活时间
            if now - pending.created_at > max_age {
                // 记录警告日志，标明清理的超时请求
                tracing::warn!("清理超时请求: {} (echo={})", pending.method, echo);
                // 返回 false 移除此条目
                false
            } else {
                // 请求仍在有效期内，保留
                true
            }
        });
    }

    /// 获取当前待处理请求的数量
    ///
    /// 用于监控和调试，了解有多少 API 请求正在等待响应。
    ///
    /// # 返回值
    ///
    /// 返回 pending map 中的条目数量
    pub fn pending_count(&self) -> usize {
        // 返回 DashMap 中的条目数量
        self.pending.len()
    }
}

/// 获取当前 UNIX 时间戳（毫秒）
///
/// 使用 `SystemTime::now()` 获取系统时间，转换为从 UNIX Epoch 起的毫秒数。
/// 若系统时间早于 UNIX Epoch（理论上不应发生），返回 0。
pub(crate) fn now_ms() -> u64 {
    // 获取从 UNIX Epoch 到现在的时间间隔
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        // 若系统时间异常，使用 Duration::default()（即 0）
        .unwrap_or_default()
        // 转换为毫秒并截断为 u64
        .as_millis() as u64
}
