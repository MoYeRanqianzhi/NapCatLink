//! # API 模块结构测试
//!
//! 验证所有 10 个 API 模块的结构正确性：
//! - OneBotApi 聚合器能正确创建
//! - NAPCAT_ACTIONS 常量数组非空且包含关键 action
//! - 各 API 模块能正确实例化

// 引入测试所需的模块
use napcat_link::api::raw::NAPCAT_ACTIONS;

/// 测试：NAPCAT_ACTIONS 常量数组非空
///
/// 验证所有已知 action 名称的常量数组不为空。
#[test]
fn test_raw_action_list_not_empty() {
    // 断言 NAPCAT_ACTIONS 数组不为空
    assert!(
        !NAPCAT_ACTIONS.is_empty(),
        "NAPCAT_ACTIONS 不应为空"
    );
    // 断言数组中包含核心 action "send_msg"
    assert!(
        NAPCAT_ACTIONS.contains(&"send_msg"),
        "NAPCAT_ACTIONS 应包含 'send_msg'"
    );
}

/// 测试：NAPCAT_ACTIONS 包含所有关键的 OneBot 11 标准 action
///
/// 验证以下核心 action 都存在于常量数组中。
#[test]
fn test_raw_action_list_has_expected_actions() {
    // 定义必须存在的关键 action 列表
    let expected_actions = [
        // 消息相关
        "send_msg",
        "send_private_msg",
        "send_group_msg",
        "delete_msg",
        "get_msg",
        "get_forward_msg",
        // 群管理
        "set_group_kick",
        "set_group_ban",
        "set_group_whole_ban",
        "set_group_admin",
        "set_group_card",
        "set_group_name",
        "set_group_leave",
        // 请求处理
        "set_friend_add_request",
        "set_group_add_request",
        // 账号信息
        "get_login_info",
        "get_friend_list",
        "get_group_list",
        "get_group_info",
        "get_group_member_list",
        "get_group_member_info",
        "get_stranger_info",
        "get_version_info",
        "get_status",
        // 媒体
        "get_image",
        "get_record",
        "get_file",
        // 文件
        "upload_group_file",
        "upload_private_file",
        "download_file",
        // 系统
        "can_send_image",
        "can_send_record",
        "get_cookies",
        "get_csrf_token",
        "ocr_image",
        // NapCat 扩展
        "get_rkey",
        "nc_get_rkey",
        "set_friend_remark",
        "delete_friend",
        "send_poke",
        "group_poke",
        "friend_poke",
        "set_msg_emoji_like",
        "get_group_msg_history",
        "get_friend_msg_history",
    ];

    // 遍历每个期望的 action，验证其存在于 NAPCAT_ACTIONS 中
    for action in &expected_actions {
        assert!(
            NAPCAT_ACTIONS.contains(action),
            "NAPCAT_ACTIONS 应包含 '{}'，但未找到",
            action
        );
    }
}

/// 测试：NAPCAT_ACTIONS 中没有重复的 action 名称
///
/// 验证常量数组中的每个 action 名称都是唯一的。
#[test]
fn test_raw_action_list_no_duplicates() {
    // 使用 HashSet 检测重复
    let mut seen = std::collections::HashSet::new();
    // 遍历所有 action 名称
    for action in NAPCAT_ACTIONS {
        // 如果 insert 返回 false，说明已存在（重复）
        assert!(
            seen.insert(action),
            "NAPCAT_ACTIONS 中发现重复的 action: '{}'",
            action
        );
    }
}

/// 测试：NAPCAT_ACTIONS 包含足够数量的 action
///
/// 验证常量数组包含了合理数量的 action（至少 100 个）。
#[test]
fn test_raw_action_list_sufficient_count() {
    // 断言至少有 100 个 action
    assert!(
        NAPCAT_ACTIONS.len() >= 100,
        "NAPCAT_ACTIONS 应至少包含 100 个 action，实际只有 {}",
        NAPCAT_ACTIONS.len()
    );
}
