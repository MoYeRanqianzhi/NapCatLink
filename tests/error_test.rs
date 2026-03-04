//! # 错误类型测试模块
//!
//! 测试 `NapLinkError` 枚举的 Display 实现、错误码映射和各种错误变体。

// 引入错误类型
use napcat_link::error::NapLinkError;

/// 测试连接错误的 Display 输出格式
///
/// 验证 `NapLinkError::Connection` 变体能正确格式化为
/// "连接错误: {描述}" 的形式。
#[test]
fn test_connection_error_display() {
    // 创建一个连接错误实例，携带描述信息
    let err = NapLinkError::Connection("连接被拒绝".to_string());
    // 验证 Display 输出包含 "连接错误" 前缀
    assert_eq!(err.to_string(), "连接错误: 连接被拒绝");
}

/// 测试 API 超时错误的 Display 输出格式
///
/// 验证 `NapLinkError::ApiTimeout` 变体能正确格式化为
/// "API 调用 {method} 超时 ({timeout_ms}ms)" 的形式。
#[test]
fn test_api_timeout_error() {
    // 创建一个 API 超时错误实例
    let err = NapLinkError::ApiTimeout {
        // API 方法名
        method: "send_msg".to_string(),
        // 超时时间 15000 毫秒
        timeout_ms: 15000,
    };
    // 验证 Display 输出包含方法名和超时时间
    assert_eq!(err.to_string(), "API 调用 send_msg 超时 (15000ms)");
}

/// 测试 API 错误在有 wording 时的 Display 输出
///
/// 验证 `NapLinkError::Api` 变体在 `wording` 字段存在时，
/// 优先显示 `wording` 而非 `message`。
#[test]
fn test_api_error_with_wording() {
    // 创建一个带 wording 的 API 错误实例
    let err_with_wording = NapLinkError::Api {
        // API 方法名
        method: "send_msg".to_string(),
        // 错误码
        retcode: 1400,
        // 原始错误消息（技术性描述）
        message: "SEND_MSG_API_ERROR".to_string(),
        // 用户友好的错误描述
        wording: Some("消息发送失败，请检查参数".to_string()),
    };
    // 验证优先显示 wording 内容
    assert_eq!(err_with_wording.to_string(), "消息发送失败，请检查参数");

    // 创建一个不带 wording 的 API 错误实例
    let err_without_wording = NapLinkError::Api {
        // API 方法名
        method: "get_group_list".to_string(),
        // 错误码
        retcode: 1500,
        // 原始错误消息
        message: "internal error".to_string(),
        // 没有用户友好描述
        wording: None,
    };
    // 验证在没有 wording 时回退到 message
    assert_eq!(err_without_wording.to_string(), "internal error");
}

/// 测试所有错误变体的错误码映射
///
/// 验证 `NapLinkError::code()` 方法为每种错误类型返回正确的错误码字符串。
#[test]
fn test_error_code() {
    // 测试连接错误的错误码
    let connection_err = NapLinkError::Connection("test".to_string());
    assert_eq!(connection_err.code(), "E_CONNECTION");

    // 测试 API 超时错误的错误码
    let timeout_err = NapLinkError::ApiTimeout {
        method: "test".to_string(),
        timeout_ms: 1000,
    };
    assert_eq!(timeout_err.code(), "E_API_TIMEOUT");

    // 测试 API 调用失败的错误码
    let api_err = NapLinkError::Api {
        method: "test".to_string(),
        retcode: 100,
        message: "fail".to_string(),
        wording: None,
    };
    assert_eq!(api_err.code(), "E_API_FAILED");

    // 测试最大重连次数错误的错误码
    let reconnect_err = NapLinkError::MaxReconnectAttempts { attempts: 5 };
    assert_eq!(reconnect_err.code(), "E_MAX_RECONNECT");

    // 测试连接关闭错误的错误码
    let closed_err = NapLinkError::ConnectionClosed {
        code: 1000,
        reason: "normal".to_string(),
    };
    assert_eq!(closed_err.code(), "E_CONNECTION_CLOSED");

    // 测试无效配置错误的错误码
    let config_err = NapLinkError::InvalidConfig {
        field: "url".to_string(),
        reason: "empty".to_string(),
    };
    assert_eq!(config_err.code(), "E_INVALID_CONFIG");

    // 测试 URL 解析错误的错误码（通过 From trait 转换）
    let url_err: NapLinkError = url::Url::parse("not a url")
        .unwrap_err()
        .into();
    assert_eq!(url_err.code(), "E_URL_PARSE");
}
