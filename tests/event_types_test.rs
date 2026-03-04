//! # 事件类型系统测试模块
//!
//! 测试 OneBot 11 事件类型系统的反序列化和序列化功能，
//! 覆盖消息事件、通知事件、请求事件、元事件和 API 类型。

// 引入事件枚举和所有子类型
use napcat_link::types::event::OneBotEvent;
// 引入 API 类型
use napcat_link::types::api::{ApiRequest, ApiResponse};
// 引入 serde_json 用于 JSON 构造和断言
use serde_json::json;

/// 测试群消息事件的反序列化
///
/// 验证完整的 OneBot 11 群消息事件 JSON 能正确反序列化为
/// `OneBotEvent::GroupMessage` 变体，且所有字段值正确。
#[test]
fn test_deserialize_group_message() {
    // 构造完整的群消息事件 JSON（模拟 NapCat 上报的真实数据格式）
    let json_val = json!({
        "time": 1700000000,                    // 事件时间戳
        "self_id": 123456789,                  // 机器人 QQ 号
        "post_type": "message",                // 上报类型：消息
        "message_type": "group",               // 消息类型：群消息
        "sub_type": "normal",                  // 子类型：普通消息
        "message_id": 2001,                    // 消息 ID
        "group_id": 100200300,                 // 群号
        "user_id": 987654321,                  // 发送者 QQ 号
        "message": [                           // 消息内容（消息段数组）
            {"type": "text", "data": {"text": "大家好，这是一条群消息"}}
        ],
        "raw_message": "大家好，这是一条群消息",   // 原始消息文本
        "font": 0,                             // 字体 ID
        "sender": {                            // 发送者信息
            "user_id": 987654321,
            "nickname": "群成员A",
            "card": "群名片A",
            "role": "member"
        }
    });
    // 将 JSON 反序列化为 OneBotEvent 枚举
    let event: OneBotEvent = serde_json::from_value(json_val).expect("群消息事件反序列化应成功");
    // 验证反序列化结果为 GroupMessage 变体
    match event {
        OneBotEvent::GroupMessage(msg) => {
            // 验证基础字段
            assert_eq!(msg.time, 1700000000, "时间戳应正确");
            assert_eq!(msg.self_id, 123456789, "机器人 QQ 号应正确");
            assert_eq!(msg.post_type, "message", "上报类型应为 message");
            assert_eq!(msg.message_type, "group", "消息类型应为 group");
            assert_eq!(msg.sub_type, "normal", "子类型应为 normal");
            // 验证消息相关字段
            assert_eq!(msg.message_id, 2001, "消息 ID 应正确");
            assert_eq!(msg.group_id, 100200300, "群号应正确");
            assert_eq!(msg.user_id, 987654321, "发送者 QQ 号应正确");
            // 验证消息内容（应有 1 个文本消息段）
            assert_eq!(msg.message.len(), 1, "消息段数组长度应为 1");
            assert_eq!(msg.raw_message, "大家好，这是一条群消息", "原始消息应正确");
            // 验证发送者信息
            assert_eq!(msg.sender.nickname.as_deref(), Some("群成员A"), "昵称应正确");
            assert_eq!(msg.sender.card.as_deref(), Some("群名片A"), "群名片应正确");
            assert_eq!(msg.sender.role.as_deref(), Some("member"), "群角色应正确");
            // 验证匿名信息不存在（普通消息没有匿名信息）
            assert!(msg.anonymous.is_none(), "普通消息不应有匿名信息");
        }
        // 如果反序列化为其他变体，测试失败
        other => panic!("期望 GroupMessage 变体，实际得到: {:?}", other),
    }
}

/// 测试私聊消息事件的反序列化
///
/// 验证完整的 OneBot 11 私聊消息事件 JSON 能正确反序列化为
/// `OneBotEvent::PrivateMessage` 变体，且所有字段值正确。
#[test]
fn test_deserialize_private_message() {
    // 构造完整的私聊消息事件 JSON
    let json_val = json!({
        "time": 1700000001,                    // 事件时间戳
        "self_id": 123456789,                  // 机器人 QQ 号
        "post_type": "message",                // 上报类型：消息
        "message_type": "private",             // 消息类型：私聊
        "sub_type": "friend",                  // 子类型：好友消息
        "message_id": 1001,                    // 消息 ID
        "user_id": 987654321,                  // 发送者 QQ 号
        "message": [                           // 消息内容
            {"type": "text", "data": {"text": "你好，这是私聊消息"}},
            {"type": "face", "data": {"id": "178"}}
        ],
        "raw_message": "你好，这是私聊消息[CQ:face,id=178]",  // 原始消息文本
        "font": 0,                             // 字体 ID
        "sender": {                            // 发送者信息
            "user_id": 987654321,
            "nickname": "好友B",
            "sex": "male",
            "age": 25
        }
    });
    // 将 JSON 反序列化为 OneBotEvent 枚举
    let event: OneBotEvent = serde_json::from_value(json_val).expect("私聊消息事件反序列化应成功");
    // 验证反序列化结果为 PrivateMessage 变体
    match event {
        OneBotEvent::PrivateMessage(msg) => {
            // 验证基础字段
            assert_eq!(msg.time, 1700000001, "时间戳应正确");
            assert_eq!(msg.self_id, 123456789, "机器人 QQ 号应正确");
            assert_eq!(msg.message_type, "private", "消息类型应为 private");
            assert_eq!(msg.sub_type, "friend", "子类型应为 friend");
            // 验证消息相关字段
            assert_eq!(msg.message_id, 1001, "消息 ID 应正确");
            assert_eq!(msg.user_id, 987654321, "发送者 QQ 号应正确");
            // 验证消息内容（应有 2 个消息段：文本 + 表情）
            assert_eq!(msg.message.len(), 2, "消息段数组长度应为 2");
            // 验证发送者信息
            assert_eq!(msg.sender.nickname.as_deref(), Some("好友B"), "昵称应正确");
            assert_eq!(msg.sender.sex.as_deref(), Some("male"), "性别应正确");
            assert_eq!(msg.sender.age, Some(25), "年龄应正确");
            // 验证 target_id 默认为 None
            assert!(msg.target_id.is_none(), "target_id 默认应为 None");
        }
        // 如果反序列化为其他变体，测试失败
        other => panic!("期望 PrivateMessage 变体，实际得到: {:?}", other),
    }
}

/// 测试群消息撤回通知事件的反序列化
///
/// 验证群消息撤回通知 JSON 能正确反序列化为
/// `OneBotEvent::GroupRecall` 变体，且操作者、被撤回消息等字段正确。
#[test]
fn test_deserialize_notice_group_recall() {
    // 构造群消息撤回通知事件 JSON
    let json_val = json!({
        "time": 1700000002,                    // 事件时间戳
        "self_id": 123456789,                  // 机器人 QQ 号
        "post_type": "notice",                 // 上报类型：通知
        "notice_type": "group_recall",         // 通知类型：群消息撤回
        "group_id": 100200300,                 // 群号
        "user_id": 987654321,                  // 被撤回消息的发送者
        "operator_id": 111222333,              // 执行撤回操作的用户（管理员）
        "message_id": 3001                     // 被撤回的消息 ID
    });
    // 将 JSON 反序列化为 OneBotEvent 枚举
    let event: OneBotEvent = serde_json::from_value(json_val).expect("群撤回通知反序列化应成功");
    // 验证反序列化结果为 GroupRecall 变体
    match event {
        OneBotEvent::GroupRecall(notice) => {
            // 验证基础字段
            assert_eq!(notice.time, 1700000002, "时间戳应正确");
            assert_eq!(notice.self_id, 123456789, "机器人 QQ 号应正确");
            assert_eq!(notice.post_type, "notice", "上报类型应为 notice");
            assert_eq!(notice.notice_type, "group_recall", "通知类型应为 group_recall");
            // 验证通知详情字段
            assert_eq!(notice.group_id, 100200300, "群号应正确");
            assert_eq!(notice.user_id, 987654321, "被撤回消息的发送者 QQ 号应正确");
            assert_eq!(notice.operator_id, 111222333, "操作者 QQ 号应正确");
            assert_eq!(notice.message_id, 3001, "被撤回消息 ID 应正确");
        }
        // 如果反序列化为其他变体，测试失败
        other => panic!("期望 GroupRecall 变体，实际得到: {:?}", other),
    }
}

/// 测试好友添加请求事件的反序列化
///
/// 验证好友添加请求 JSON 能正确反序列化为
/// `OneBotEvent::FriendRequest` 变体，且验证信息和标识 flag 正确。
#[test]
fn test_deserialize_friend_request() {
    // 构造好友添加请求事件 JSON
    let json_val = json!({
        "time": 1700000003,                    // 事件时间戳
        "self_id": 123456789,                  // 机器人 QQ 号
        "post_type": "request",                // 上报类型：请求
        "request_type": "friend",              // 请求类型：好友添加
        "user_id": 555666777,                  // 请求者 QQ 号
        "comment": "你好，我想加你为好友",        // 验证留言
        "flag": "flag_friend_abc123"           // 请求标识（用于处理请求时引用）
    });
    // 将 JSON 反序列化为 OneBotEvent 枚举
    let event: OneBotEvent = serde_json::from_value(json_val).expect("好友请求反序列化应成功");
    // 验证反序列化结果为 FriendRequest 变体
    match event {
        OneBotEvent::FriendRequest(req) => {
            // 验证基础字段
            assert_eq!(req.time, 1700000003, "时间戳应正确");
            assert_eq!(req.self_id, 123456789, "机器人 QQ 号应正确");
            assert_eq!(req.post_type, "request", "上报类型应为 request");
            assert_eq!(req.request_type, "friend", "请求类型应为 friend");
            // 验证请求详情字段
            assert_eq!(req.user_id, 555666777, "请求者 QQ 号应正确");
            assert_eq!(req.comment, "你好，我想加你为好友", "验证留言应正确");
            assert_eq!(req.flag, "flag_friend_abc123", "请求标识 flag 应正确");
        }
        // 如果反序列化为其他变体，测试失败
        other => panic!("期望 FriendRequest 变体，实际得到: {:?}", other),
    }
}

/// 测试心跳元事件的反序列化
///
/// 验证心跳元事件 JSON 能正确反序列化为
/// `OneBotEvent::Heartbeat` 变体，且状态信息和间隔时间正确。
#[test]
fn test_deserialize_heartbeat() {
    // 构造心跳元事件 JSON
    let json_val = json!({
        "time": 1700000004,                    // 事件时间戳
        "self_id": 123456789,                  // 机器人 QQ 号
        "post_type": "meta_event",             // 上报类型：元事件
        "meta_event_type": "heartbeat",        // 元事件类型：心跳
        "status": {                            // 机器人状态
            "online": true,                    // 在线状态
            "good": true                       // 运行状况
        },
        "interval": 5000                       // 心跳间隔（毫秒）
    });
    // 将 JSON 反序列化为 OneBotEvent 枚举
    let event: OneBotEvent = serde_json::from_value(json_val).expect("心跳事件反序列化应成功");
    // 验证反序列化结果为 Heartbeat 变体
    match event {
        OneBotEvent::Heartbeat(hb) => {
            // 验证基础字段
            assert_eq!(hb.time, 1700000004, "时间戳应正确");
            assert_eq!(hb.self_id, 123456789, "机器人 QQ 号应正确");
            assert_eq!(hb.post_type, "meta_event", "上报类型应为 meta_event");
            assert_eq!(hb.meta_event_type, "heartbeat", "元事件类型应为 heartbeat");
            // 验证状态信息
            assert!(hb.status.online, "机器人应为在线状态");
            assert!(hb.status.good, "机器人运行状况应良好");
            // 验证心跳间隔
            assert_eq!(hb.interval, 5000, "心跳间隔应为 5000 毫秒");
        }
        // 如果反序列化为其他变体，测试失败
        other => panic!("期望 Heartbeat 变体，实际得到: {:?}", other),
    }
}

/// 测试 API 响应格式的反序列化
///
/// 验证 OneBot 11 标准 API 响应 JSON 能正确反序列化为
/// `ApiResponse` 结构体，包括成功和失败两种情况。
#[test]
fn test_api_response_deserialize() {
    // ----- 测试成功响应 -----
    // 构造一个成功的 API 响应 JSON（如 send_msg 返回的消息 ID）
    let success_json = json!({
        "status": "ok",                        // 响应状态：成功
        "retcode": 0,                          // 返回码：0 表示成功
        "data": {"message_id": 12345},         // 响应数据：发送成功后的消息 ID
        "echo": "req-001"                      // 回声标识：与请求匹配
    });
    // 反序列化为 ApiResponse（使用默认泛型 serde_json::Value）
    let resp: ApiResponse =
        serde_json::from_value(success_json).expect("成功响应反序列化应成功");
    // 验证各字段
    assert_eq!(resp.status, "ok", "状态应为 ok");
    assert_eq!(resp.retcode, 0, "返回码应为 0");
    assert_eq!(resp.data["message_id"], 12345, "data 中应包含 message_id");
    assert_eq!(resp.echo.as_deref(), Some("req-001"), "echo 应匹配请求标识");
    // 成功响应不应有错误信息
    assert!(resp.message.is_none(), "成功响应不应有 message 字段");
    assert!(resp.wording.is_none(), "成功响应不应有 wording 字段");

    // ----- 测试失败响应 -----
    // 构造一个失败的 API 响应 JSON
    let fail_json = json!({
        "status": "failed",                    // 响应状态：失败
        "retcode": 1400,                       // 返回码：参数错误
        "data": null,                          // 响应数据：空
        "echo": "req-002",                     // 回声标识
        "message": "INVALID_PARAM",            // 技术错误信息
        "wording": "参数无效"                   // 友好错误提示
    });
    // 反序列化为 ApiResponse
    let resp: ApiResponse =
        serde_json::from_value(fail_json).expect("失败响应反序列化应成功");
    // 验证各字段
    assert_eq!(resp.status, "failed", "状态应为 failed");
    assert_eq!(resp.retcode, 1400, "返回码应为 1400");
    assert!(resp.data.is_null(), "失败响应 data 应为 null");
    assert_eq!(resp.echo.as_deref(), Some("req-002"), "echo 应匹配请求标识");
    assert_eq!(resp.message.as_deref(), Some("INVALID_PARAM"), "技术错误信息应正确");
    assert_eq!(resp.wording.as_deref(), Some("参数无效"), "友好错误提示应正确");
}

/// 测试 API 请求格式的序列化
///
/// 验证 `ApiRequest` 结构体能正确序列化为标准的 OneBot 11 API 请求 JSON，
/// 且反序列化后数据保持一致（往返测试）。
#[test]
fn test_api_request_serialize() {
    // 构造一个发送群消息的 API 请求
    let request = ApiRequest {
        // API 动作名称：发送群消息
        action: "send_group_msg".to_string(),
        // 请求参数：群号和消息内容
        params: json!({
            "group_id": 100200300,
            "message": [
                {"type": "text", "data": {"text": "Hello from Rust!"}}
            ]
        }),
        // 回声标识：用于匹配响应
        echo: "req-send-001".to_string(),
    };
    // 序列化为 JSON Value
    let json_val = serde_json::to_value(&request).expect("API 请求序列化应成功");
    // 验证 action 字段
    assert_eq!(json_val["action"], "send_group_msg", "action 应为 send_group_msg");
    // 验证 params 中的群号
    assert_eq!(json_val["params"]["group_id"], 100200300, "群号应正确");
    // 验证 params 中的消息内容
    assert!(json_val["params"]["message"].is_array(), "message 应为数组");
    assert_eq!(
        json_val["params"]["message"][0]["type"], "text",
        "第一个消息段类型应为 text"
    );
    // 验证 echo 字段
    assert_eq!(json_val["echo"], "req-send-001", "echo 应正确");

    // 往返测试：序列化 -> 反序列化 -> 验证字段一致
    let json_str = serde_json::to_string(&request).expect("API 请求序列化为字符串应成功");
    let recovered: ApiRequest =
        serde_json::from_str(&json_str).expect("API 请求反序列化应成功");
    // 验证反序列化后各字段一致
    assert_eq!(recovered.action, request.action, "往返后 action 应一致");
    assert_eq!(recovered.params, request.params, "往返后 params 应一致");
    assert_eq!(recovered.echo, request.echo, "往返后 echo 应一致");
}

/// 测试生命周期元事件的反序列化
///
/// 验证生命周期事件（WebSocket 连接成功）JSON 能正确反序列化为
/// `OneBotEvent::Lifecycle` 变体。
#[test]
fn test_deserialize_lifecycle() {
    // 构造生命周期事件 JSON（连接成功类型）
    let json_val = json!({
        "time": 1700000005,                    // 事件时间戳
        "self_id": 123456789,                  // 机器人 QQ 号
        "post_type": "meta_event",             // 上报类型：元事件
        "meta_event_type": "lifecycle",        // 元事件类型：生命周期
        "sub_type": "connect"                  // 子类型：连接成功
    });
    // 将 JSON 反序列化为 OneBotEvent 枚举
    let event: OneBotEvent = serde_json::from_value(json_val).expect("生命周期事件反序列化应成功");
    // 验证反序列化结果为 Lifecycle 变体
    match event {
        OneBotEvent::Lifecycle(lc) => {
            // 验证基础字段
            assert_eq!(lc.time, 1700000005, "时间戳应正确");
            assert_eq!(lc.meta_event_type, "lifecycle", "元事件类型应为 lifecycle");
            assert_eq!(lc.sub_type, "connect", "子类型应为 connect");
        }
        // 如果反序列化为其他变体，测试失败
        other => panic!("期望 Lifecycle 变体，实际得到: {:?}", other),
    }
}

/// 测试群加入请求事件的反序列化
///
/// 验证群添加请求 JSON 能正确反序列化为
/// `OneBotEvent::GroupRequest` 变体。
#[test]
fn test_deserialize_group_request() {
    // 构造群添加请求事件 JSON
    let json_val = json!({
        "time": 1700000006,                    // 事件时间戳
        "self_id": 123456789,                  // 机器人 QQ 号
        "post_type": "request",                // 上报类型：请求
        "request_type": "group",               // 请求类型：群相关
        "sub_type": "add",                     // 子类型：主动加群
        "group_id": 100200300,                 // 群号
        "user_id": 888999000,                  // 请求者 QQ 号
        "comment": "请批准我加入",              // 验证留言
        "flag": "flag_group_xyz789"            // 请求标识
    });
    // 将 JSON 反序列化为 OneBotEvent 枚举
    let event: OneBotEvent = serde_json::from_value(json_val).expect("群请求反序列化应成功");
    // 验证反序列化结果为 GroupRequest 变体
    match event {
        OneBotEvent::GroupRequest(req) => {
            // 验证基础字段
            assert_eq!(req.request_type, "group", "请求类型应为 group");
            assert_eq!(req.sub_type, "add", "子类型应为 add");
            // 验证请求详情
            assert_eq!(req.group_id, 100200300, "群号应正确");
            assert_eq!(req.user_id, 888999000, "请求者 QQ 号应正确");
            assert_eq!(req.comment, "请批准我加入", "验证留言应正确");
            assert_eq!(req.flag, "flag_group_xyz789", "请求标识应正确");
        }
        // 如果反序列化为其他变体，测试失败
        other => panic!("期望 GroupRequest 变体，实际得到: {:?}", other),
    }
}

/// 测试好友添加通知事件的反序列化
///
/// 验证好友添加通知 JSON 能正确反序列化为
/// `OneBotEvent::FriendAdd` 变体。
#[test]
fn test_deserialize_friend_add_notice() {
    // 构造好友添加通知事件 JSON
    let json_val = json!({
        "time": 1700000007,                    // 事件时间戳
        "self_id": 123456789,                  // 机器人 QQ 号
        "post_type": "notice",                 // 上报类型：通知
        "notice_type": "friend_add",           // 通知类型：好友添加
        "user_id": 444555666                   // 新好友 QQ 号
    });
    // 将 JSON 反序列化为 OneBotEvent 枚举
    let event: OneBotEvent = serde_json::from_value(json_val).expect("好友添加通知反序列化应成功");
    // 验证反序列化结果为 FriendAdd 变体
    match event {
        OneBotEvent::FriendAdd(notice) => {
            // 验证基础字段
            assert_eq!(notice.post_type, "notice", "上报类型应为 notice");
            assert_eq!(notice.notice_type, "friend_add", "通知类型应为 friend_add");
            assert_eq!(notice.user_id, 444555666, "新好友 QQ 号应正确");
        }
        // 如果反序列化为其他变体，测试失败
        other => panic!("期望 FriendAdd 变体，实际得到: {:?}", other),
    }
}

/// 测试未知事件的兜底反序列化
///
/// 验证不符合任何已知事件格式的 JSON 能正确反序列化为
/// `OneBotEvent::Unknown` 变体，不会导致反序列化失败。
#[test]
fn test_deserialize_unknown_event() {
    // 构造一个不符合任何已知事件格式的 JSON
    let json_val = json!({
        "time": 1700000099,
        "self_id": 123456789,
        "post_type": "custom_event",           // 未知的上报类型
        "custom_field": "custom_value"         // 自定义字段
    });
    // 将 JSON 反序列化为 OneBotEvent 枚举
    let event: OneBotEvent = serde_json::from_value(json_val.clone()).expect("未知事件反序列化应成功");
    // 验证反序列化结果为 Unknown 变体
    match event {
        OneBotEvent::Unknown(val) => {
            // 验证原始 JSON 数据被完整保留
            assert_eq!(val["post_type"], "custom_event", "上报类型应被保留");
            assert_eq!(val["custom_field"], "custom_value", "自定义字段应被保留");
        }
        // 如果反序列化为其他变体，测试失败
        other => panic!("期望 Unknown 变体，实际得到: {:?}", other),
    }
}

/// 测试群文件上传通知事件的反序列化
///
/// 验证群文件上传通知 JSON 能正确反序列化，
/// 包括嵌套的 FileInfo 结构体。
#[test]
fn test_deserialize_group_upload_notice() {
    // 构造群文件上传通知事件 JSON
    let json_val = json!({
        "time": 1700000008,                    // 事件时间戳
        "self_id": 123456789,                  // 机器人 QQ 号
        "post_type": "notice",                 // 上报类型：通知
        "notice_type": "group_upload",         // 通知类型：群文件上传
        "group_id": 100200300,                 // 群号
        "user_id": 987654321,                  // 上传者 QQ 号
        "file": {                              // 文件信息
            "id": "file_abc123",               // 文件 ID
            "name": "report.pdf",              // 文件名
            "size": 2048576,                   // 文件大小（字节）
            "busid": 102                       // 总线 ID
        }
    });
    // 将 JSON 反序列化为 OneBotEvent 枚举
    let event: OneBotEvent = serde_json::from_value(json_val).expect("群文件上传通知反序列化应成功");
    // 验证反序列化结果为 GroupUpload 变体
    match event {
        OneBotEvent::GroupUpload(notice) => {
            // 验证基础字段
            assert_eq!(notice.notice_type, "group_upload", "通知类型应为 group_upload");
            assert_eq!(notice.group_id, 100200300, "群号应正确");
            assert_eq!(notice.user_id, 987654321, "上传者 QQ 号应正确");
            // 验证文件信息
            assert_eq!(notice.file.id, "file_abc123", "文件 ID 应正确");
            assert_eq!(notice.file.name, "report.pdf", "文件名应正确");
            assert_eq!(notice.file.size, 2048576, "文件大小应正确");
            assert_eq!(notice.file.busid, 102, "总线 ID 应正确");
        }
        // 如果反序列化为其他变体，测试失败
        other => panic!("期望 GroupUpload 变体，实际得到: {:?}", other),
    }
}

/// 测试戳一戳通知事件的反序列化
///
/// 验证戳一戳通知 JSON 能正确反序列化为
/// `OneBotEvent::Poke` 变体。
#[test]
fn test_deserialize_poke_notice() {
    // 构造戳一戳通知事件 JSON（群内戳一戳）
    let json_val = json!({
        "time": 1700000009,                    // 事件时间戳
        "self_id": 123456789,                  // 机器人 QQ 号
        "post_type": "notice",                 // 上报类型：通知
        "notice_type": "notify",               // 通知类型：通用通知
        "sub_type": "poke",                    // 子类型：戳一戳
        "group_id": 100200300,                 // 群号（群内戳一戳）
        "user_id": 987654321,                  // 发起者 QQ 号
        "target_id": 123456789                 // 被戳的目标 QQ 号
    });
    // 将 JSON 反序列化为 OneBotEvent 枚举
    let event: OneBotEvent = serde_json::from_value(json_val).expect("戳一戳通知反序列化应成功");
    // 验证反序列化结果为 Poke 变体
    match event {
        OneBotEvent::Poke(notice) => {
            // 验证基础字段
            assert_eq!(notice.notice_type, "notify", "通知类型应为 notify");
            assert_eq!(notice.sub_type, "poke", "子类型应为 poke");
            // 验证戳一戳详情
            assert_eq!(notice.group_id, Some(100200300), "群号应正确");
            assert_eq!(notice.user_id, 987654321, "发起者 QQ 号应正确");
            assert_eq!(notice.target_id, 123456789, "被戳目标 QQ 号应正确");
        }
        // 如果反序列化为其他变体，测试失败
        other => panic!("期望 Poke 变体，实际得到: {:?}", other),
    }
}
