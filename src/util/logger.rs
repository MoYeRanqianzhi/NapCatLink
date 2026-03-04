//! # 日志工具模块
//!
//! 封装 tracing 日志系统的初始化和配置，包括：
//! - 日志级别配置（通过环境变量 RUST_LOG 控制）
//! - 日志格式化（时间戳、模块路径、日志级别）
//! - 日志输出目标（标准输出）

// 引入 tracing-subscriber 的环境过滤器，支持通过 RUST_LOG 环境变量控制日志级别
use tracing_subscriber::EnvFilter;

/// 初始化 tracing 日志系统
///
/// 配置全局日志订阅者，设置日志输出格式和级别。
/// 优先使用 `RUST_LOG` 环境变量中的配置，如果未设置则使用传入的默认级别。
///
/// # 参数
///
/// - `level`: 默认日志级别字符串（例如 "info"、"debug"、"warn"）
///
/// # 注意
///
/// 此函数只能在程序中调用一次。如果多次调用，后续调用会被忽略（tracing 的内部机制）。
///
/// # 示例
///
/// ```rust,no_run
/// napcat_link::util::logger::init_tracing("info");
/// ```
#[allow(dead_code)]
pub fn init_tracing(level: &str) {
    // 尝试从 RUST_LOG 环境变量读取过滤配置，如果未设置则使用传入的默认级别
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level));
    // 初始化 tracing-subscriber 的格式化输出（标准输出 + 带环境过滤器）
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();
}
