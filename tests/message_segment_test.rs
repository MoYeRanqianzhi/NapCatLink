//! # 消息段类型测试模块
//!
//! 测试 `MessageSegment` 枚举的序列化、反序列化、快捷构造方法，
//! 以及未知类型的兜底处理和完整的往返（roundtrip）测试。

// 引入消息段类型
use napcat_link::types::message::MessageSegment;
// 引入 serde_json 用于 JSON 序列化/反序列化验证
use serde_json::{json, Value};

/// 测试文本消息段的序列化
///
/// 验证 `MessageSegment::Text` 变体序列化为正确的 JSON 格式：
/// `{ "type": "text", "data": { "text": "..." } }`
#[test]
fn test_text_segment_serialize() {
    // 创建一个文本消息段
    let seg = MessageSegment::text("你好世界");
    // 序列化为 JSON Value
    let json_val = serde_json::to_value(&seg).expect("文本消息段序列化应成功");
    // 验证 type 字段为 "text"
    assert_eq!(json_val["type"], "text");
    // 验证 data.text 字段为文本内容
    assert_eq!(json_val["data"]["text"], "你好世界");
    // 验证 data 对象中只有 text 一个字段
    let data_obj = json_val["data"].as_object().expect("data 应为对象");
    assert_eq!(data_obj.len(), 1, "文本消息段 data 应只包含 text 字段");
}

/// 测试 @消息段的序列化
///
/// 验证 `MessageSegment::At` 变体序列化为正确的 JSON 格式：
/// `{ "type": "at", "data": { "qq": "..." } }`
#[test]
fn test_at_segment_serialize() {
    // 创建一个 @某人消息段
    let seg = MessageSegment::at("123456789");
    // 序列化为 JSON Value
    let json_val = serde_json::to_value(&seg).expect("@消息段序列化应成功");
    // 验证 type 字段为 "at"
    assert_eq!(json_val["type"], "at");
    // 验证 data.qq 字段为 QQ 号
    assert_eq!(json_val["data"]["qq"], "123456789");
    // 验证 name 字段未出现在 JSON 中（因为是 None，被 skip_serializing_if 跳过）
    assert!(
        json_val["data"].get("name").is_none(),
        "name 为 None 时不应出现在 JSON 中"
    );
}

/// 测试 @全体成员的序列化
///
/// 验证 `MessageSegment::at_all()` 快捷方法创建的消息段
/// 序列化时 qq 字段值为 "all"。
#[test]
fn test_at_all_serialize() {
    // 使用快捷方法创建 @全体成员消息段
    let seg = MessageSegment::at_all();
    // 序列化为 JSON Value
    let json_val = serde_json::to_value(&seg).expect("@全体成员消息段序列化应成功");
    // 验证 type 字段为 "at"
    assert_eq!(json_val["type"], "at");
    // 验证 data.qq 字段为 "all"
    assert_eq!(json_val["data"]["qq"], "all");
}

/// 测试图片消息段的序列化
///
/// 验证 `MessageSegment::Image` 变体序列化为正确的 JSON 格式，
/// 且 None 的可选字段不出现在输出中。
#[test]
fn test_image_segment_serialize() {
    // 使用快捷方法创建图片消息段（仅提供 file 参数）
    let seg = MessageSegment::image("https://example.com/test.png");
    // 序列化为 JSON Value
    let json_val = serde_json::to_value(&seg).expect("图片消息段序列化应成功");
    // 验证 type 字段为 "image"
    assert_eq!(json_val["type"], "image");
    // 验证 data.file 字段为图片路径
    assert_eq!(json_val["data"]["file"], "https://example.com/test.png");
    // 验证可选字段不出现在 JSON 中
    assert!(
        json_val["data"].get("summary").is_none(),
        "summary 为 None 时不应出现"
    );
    assert!(
        json_val["data"].get("sub_type").is_none(),
        "sub_type 为 None 时不应出现"
    );
    assert!(
        json_val["data"].get("url").is_none(),
        "url 为 None 时不应出现"
    );
}

/// 测试从 JSON 反序列化文本消息段
///
/// 验证标准 OneBot 11 格式的 JSON 能正确反序列化为 `MessageSegment::Text` 变体。
#[test]
fn test_deserialize_text_segment() {
    // 构造标准 OneBot 11 文本消息段 JSON
    let json_str = r#"{"type":"text","data":{"text":"Hello OneBot"}}"#;
    // 反序列化为 MessageSegment
    let seg: MessageSegment = serde_json::from_str(json_str).expect("文本消息段反序列化应成功");
    // 验证反序列化结果为 Text 变体
    assert_eq!(
        seg,
        MessageSegment::Text {
            text: "Hello OneBot".to_string()
        }
    );
}

/// 测试带可选字段的图片消息段反序列化
///
/// 验证包含所有可选字段（summary、sub_type、url）的图片消息段 JSON
/// 能正确反序列化，且所有字段值都正确保留。
#[test]
fn test_deserialize_image_with_optional_fields() {
    // 构造包含所有可选字段的图片消息段 JSON
    let json_val = json!({
        "type": "image",
        "data": {
            "file": "abc123.image",
            "summary": "一张图片",
            "sub_type": "normal",
            "url": "https://example.com/download/abc123.png"
        }
    });
    // 从 Value 反序列化为 MessageSegment
    let seg: MessageSegment =
        serde_json::from_value(json_val).expect("带可选字段的图片消息段反序列化应成功");
    // 验证反序列化结果
    match seg {
        // 匹配 Image 变体并验证所有字段
        MessageSegment::Image {
            file,
            summary,
            sub_type,
            url,
        } => {
            // 验证必填字段
            assert_eq!(file, "abc123.image");
            // 验证可选字段都有值
            assert_eq!(summary.as_deref(), Some("一张图片"));
            assert_eq!(sub_type.as_deref(), Some("normal"));
            assert_eq!(
                url.as_deref(),
                Some("https://example.com/download/abc123.png")
            );
        }
        // 如果反序列化为其他变体，测试失败
        other => panic!("期望 Image 变体，实际得到: {:?}", other),
    }
}

/// 测试未知消息段类型的兜底反序列化
///
/// 验证当收到不在已知列表中的消息段类型时，
/// 能正确反序列化为 `MessageSegment::Unknown` 变体，
/// 保留原始的 type 和 data 数据不丢失。
#[test]
fn test_deserialize_unknown_segment() {
    // 构造一个未知类型的消息段 JSON
    let json_val = json!({
        "type": "weather",
        "data": {
            "city": "北京",
            "temperature": "25",
            "unit": "celsius"
        }
    });
    // 反序列化为 MessageSegment
    let seg: MessageSegment =
        serde_json::from_value(json_val).expect("未知消息段类型反序列化应成功");
    // 验证反序列化结果为 Unknown 变体
    match seg {
        MessageSegment::Unknown { r#type, data } => {
            // 验证原始类型名被保留
            assert_eq!(r#type, "weather");
            // 验证原始数据被完整保留
            assert_eq!(data["city"], "北京");
            assert_eq!(data["temperature"], "25");
            assert_eq!(data["unit"], "celsius");
        }
        // 如果反序列化为其他变体，测试失败
        other => panic!("期望 Unknown 变体，实际得到: {:?}", other),
    }
}

/// 测试快捷构造方法
///
/// 验证所有快捷构造方法（text、at、at_all、image、reply、face、record、video、json、markdown）
/// 能正确创建对应的消息段变体，且默认值设置正确。
#[test]
fn test_message_segment_helpers() {
    // 测试 text 快捷方法
    let text_seg = MessageSegment::text("hello");
    assert_eq!(
        text_seg,
        MessageSegment::Text {
            text: "hello".to_string()
        }
    );

    // 测试 at 快捷方法
    let at_seg = MessageSegment::at("12345");
    assert_eq!(
        at_seg,
        MessageSegment::At {
            qq: "12345".to_string(),
            name: None,
        }
    );

    // 测试 at_all 快捷方法
    let at_all_seg = MessageSegment::at_all();
    assert_eq!(
        at_all_seg,
        MessageSegment::At {
            qq: "all".to_string(),
            name: None,
        }
    );

    // 测试 image 快捷方法
    let image_seg = MessageSegment::image("test.png");
    assert_eq!(
        image_seg,
        MessageSegment::Image {
            file: "test.png".to_string(),
            summary: None,
            sub_type: None,
            url: None,
        }
    );

    // 测试 reply 快捷方法
    let reply_seg = MessageSegment::reply("999");
    assert_eq!(
        reply_seg,
        MessageSegment::Reply {
            id: "999".to_string()
        }
    );

    // 测试 face 快捷方法
    let face_seg = MessageSegment::face("178");
    assert_eq!(
        face_seg,
        MessageSegment::Face {
            id: "178".to_string()
        }
    );

    // 测试 record 快捷方法
    let record_seg = MessageSegment::record("voice.amr");
    assert_eq!(
        record_seg,
        MessageSegment::Record {
            file: "voice.amr".to_string(),
            url: None,
        }
    );

    // 测试 video 快捷方法
    let video_seg = MessageSegment::video("clip.mp4");
    assert_eq!(
        video_seg,
        MessageSegment::Video {
            file: "clip.mp4".to_string(),
            url: None,
        }
    );

    // 测试 json 快捷方法
    let json_seg = MessageSegment::json(r#"{"key":"value"}"#);
    assert_eq!(
        json_seg,
        MessageSegment::Json {
            data: r#"{"key":"value"}"#.to_string()
        }
    );

    // 测试 markdown 快捷方法
    let md_seg = MessageSegment::markdown("# Title");
    assert_eq!(
        md_seg,
        MessageSegment::Markdown {
            content: "# Title".to_string()
        }
    );
}

/// 测试所有消息段类型的序列化-反序列化往返（roundtrip）
///
/// 对每种消息段类型执行：创建 -> 序列化为 JSON -> 从 JSON 反序列化 -> 比较一致性。
/// 确保序列化和反序列化过程中不丢失任何数据。
#[test]
fn test_roundtrip_all_types() {
    // 构建包含所有已知消息段类型的测试列表
    let segments: Vec<MessageSegment> = vec![
        // 1. 文本消息段
        MessageSegment::Text {
            text: "roundtrip text".to_string(),
        },
        // 2. @消息段（带可选 name）
        MessageSegment::At {
            qq: "10086".to_string(),
            name: Some("测试用户".to_string()),
        },
        // 3. @消息段（不带 name）
        MessageSegment::At {
            qq: "all".to_string(),
            name: None,
        },
        // 4. 表情消息段
        MessageSegment::Face {
            id: "178".to_string(),
        },
        // 5. 回复消息段
        MessageSegment::Reply {
            id: "54321".to_string(),
        },
        // 6. 图片消息段（带所有可选字段）
        MessageSegment::Image {
            file: "abc.image".to_string(),
            summary: Some("图片描述".to_string()),
            sub_type: Some("normal".to_string()),
            url: Some("https://example.com/abc.png".to_string()),
        },
        // 7. 图片消息段（仅必填字段）
        MessageSegment::Image {
            file: "minimal.image".to_string(),
            summary: None,
            sub_type: None,
            url: None,
        },
        // 8. 语音消息段
        MessageSegment::Record {
            file: "voice.amr".to_string(),
            url: Some("https://example.com/voice.amr".to_string()),
        },
        // 9. 视频消息段
        MessageSegment::Video {
            file: "video.mp4".to_string(),
            url: None,
        },
        // 10. 文件消息段
        MessageSegment::File {
            file: "document.pdf".to_string(),
            name: Some("报告.pdf".to_string()),
            url: Some("https://example.com/document.pdf".to_string()),
        },
        // 11. JSON 消息段
        MessageSegment::Json {
            data: r#"{"app":"com.tencent.miniapp"}"#.to_string(),
        },
        // 12. XML 消息段
        MessageSegment::Xml {
            data: "<xml>test</xml>".to_string(),
        },
        // 13. Markdown 消息段
        MessageSegment::Markdown {
            content: "# Hello\n**bold**".to_string(),
        },
        // 14. 戳一戳消息段
        MessageSegment::Poke {
            poke_type: "126".to_string(),
            id: "2003".to_string(),
        },
        // 15. 掷骰子消息段（带结果）
        MessageSegment::Dice {
            result: Some("6".to_string()),
        },
        // 16. 掷骰子消息段（不带结果）
        MessageSegment::Dice { result: None },
        // 17. 猜拳消息段
        MessageSegment::Rps {
            result: Some("1".to_string()),
        },
        // 18. 商城表情消息段
        MessageSegment::MFace {
            emoji_id: Some("12345".to_string()),
            emoji_package_id: Some("1".to_string()),
            key: Some("abc".to_string()),
            summary: Some("[商城表情]".to_string()),
        },
        // 19. 音乐分享消息段（QQ音乐）
        MessageSegment::Music {
            music_type: "qq".to_string(),
            id: Some("002J4UUk29y8BY".to_string()),
            url: None,
            audio: None,
            title: None,
            content: None,
            image: None,
        },
        // 20. 音乐分享消息段（自定义音乐）
        MessageSegment::Music {
            music_type: "custom".to_string(),
            id: None,
            url: Some("https://example.com/song".to_string()),
            audio: Some("https://example.com/song.mp3".to_string()),
            title: Some("测试歌曲".to_string()),
            content: Some("测试歌手".to_string()),
            image: Some("https://example.com/cover.png".to_string()),
        },
        // 21. 合并转发节点消息段（引用模式）
        MessageSegment::Node {
            id: Some("msg-id-123".to_string()),
            user_id: None,
            nickname: None,
            content: None,
        },
        // 22. 合并转发节点消息段（自定义节点模式）
        MessageSegment::Node {
            id: None,
            user_id: Some("10086".to_string()),
            nickname: Some("测试机器人".to_string()),
            content: Some(vec![
                MessageSegment::text("嵌套消息内容"),
                MessageSegment::face("178"),
            ]),
        },
        // 23. 合并转发消息段
        MessageSegment::Forward {
            id: "forward-id-456".to_string(),
        },
        // 24. 推荐联系人消息段
        MessageSegment::Contact {
            contact_type: "qq".to_string(),
            id: "123456".to_string(),
        },
        // 25. 位置消息段
        MessageSegment::Location {
            lat: "39.9042".to_string(),
            lon: "116.4074".to_string(),
            title: Some("北京天安门".to_string()),
            content: Some("北京市东城区".to_string()),
        },
        // 26. 小程序消息段
        MessageSegment::MiniApp {
            data: json!({
                "appid": "12345",
                "title": "测试小程序"
            }),
        },
        // 27. 未知消息段
        MessageSegment::Unknown {
            r#type: "custom_type".to_string(),
            data: json!({"foo": "bar", "num": 42}),
        },
    ];

    // 对每个消息段执行序列化 -> 反序列化往返测试
    for (index, original) in segments.iter().enumerate() {
        // 序列化为 JSON 字符串
        let json_str = serde_json::to_string(original)
            .unwrap_or_else(|e| panic!("第 {} 个消息段序列化失败: {}", index, e));
        // 从 JSON 字符串反序列化回 MessageSegment
        let deserialized: MessageSegment = serde_json::from_str(&json_str)
            .unwrap_or_else(|e| panic!("第 {} 个消息段反序列化失败: {}: JSON = {}", index, e, json_str));
        // 验证序列化-反序列化往返后数据一致
        assert_eq!(
            *original, deserialized,
            "第 {} 个消息段往返测试失败：\n原始: {:?}\n反序列化: {:?}\nJSON: {}",
            index, original, deserialized, json_str
        );
    }
}

/// 测试戳一戳消息段的 type 字段重命名
///
/// 验证 Poke 消息段的 `poke_type` 字段在 JSON 中被正确重命名为 "type"。
#[test]
fn test_poke_type_rename() {
    // 创建戳一戳消息段
    let seg = MessageSegment::Poke {
        poke_type: "126".to_string(),
        id: "2003".to_string(),
    };
    // 序列化为 JSON Value
    let json_val = serde_json::to_value(&seg).expect("戳一戳消息段序列化应成功");
    // 验证外层 type 为 "poke"
    assert_eq!(json_val["type"], "poke");
    // 验证 data 内部的 type 字段（由 poke_type 重命名）
    assert_eq!(json_val["data"]["type"], "126");
    // 验证 data 内部的 id 字段
    assert_eq!(json_val["data"]["id"], "2003");
    // 验证 data 中不存在 poke_type 字段名
    assert!(
        json_val["data"].get("poke_type").is_none(),
        "poke_type 应被重命名为 type"
    );
}

/// 测试音乐消息段的 type 字段重命名
///
/// 验证 Music 消息段的 `music_type` 字段在 JSON 中被正确重命名为 "type"。
#[test]
fn test_music_type_rename() {
    // 创建 QQ 音乐分享消息段
    let seg = MessageSegment::Music {
        music_type: "qq".to_string(),
        id: Some("002J4UUk29y8BY".to_string()),
        url: None,
        audio: None,
        title: None,
        content: None,
        image: None,
    };
    // 序列化为 JSON Value
    let json_val = serde_json::to_value(&seg).expect("音乐消息段序列化应成功");
    // 验证外层 type 为 "music"
    assert_eq!(json_val["type"], "music");
    // 验证 data 内部的 type 字段（由 music_type 重命名）
    assert_eq!(json_val["data"]["type"], "qq");
    // 验证 data 中不存在 music_type 字段名
    assert!(
        json_val["data"].get("music_type").is_none(),
        "music_type 应被重命名为 type"
    );
}

/// 测试联系人消息段的 type 字段重命名
///
/// 验证 Contact 消息段的 `contact_type` 字段在 JSON 中被正确重命名为 "type"。
#[test]
fn test_contact_type_rename() {
    // 创建好友名片分享消息段
    let seg = MessageSegment::Contact {
        contact_type: "qq".to_string(),
        id: "123456".to_string(),
    };
    // 序列化为 JSON Value
    let json_val = serde_json::to_value(&seg).expect("联系人消息段序列化应成功");
    // 验证外层 type 为 "contact"
    assert_eq!(json_val["type"], "contact");
    // 验证 data 内部的 type 字段（由 contact_type 重命名）
    assert_eq!(json_val["data"]["type"], "qq");
    // 验证 data 中不存在 contact_type 字段名
    assert!(
        json_val["data"].get("contact_type").is_none(),
        "contact_type 应被重命名为 type"
    );
}

/// 测试消息段数组的序列化和反序列化
///
/// 模拟实际使用场景：一条消息包含多个消息段（回复 + @某人 + 文本 + 图片），
/// 验证数组级别的序列化和反序列化正确性。
#[test]
fn test_message_array_roundtrip() {
    // 构造一条包含多个消息段的消息（模拟真实聊天场景）
    let message: Vec<MessageSegment> = vec![
        // 先引用回复某条消息
        MessageSegment::reply("msg-id-100"),
        // @某个群成员
        MessageSegment::at("10086"),
        // 文本内容
        MessageSegment::text(" 看看这张图片："),
        // 附带一张图片
        MessageSegment::image("https://example.com/photo.jpg"),
    ];

    // 序列化为 JSON 字符串
    let json_str = serde_json::to_string(&message).expect("消息数组序列化应成功");
    // 从 JSON 字符串反序列化回 Vec<MessageSegment>
    let deserialized: Vec<MessageSegment> =
        serde_json::from_str(&json_str).expect("消息数组反序列化应成功");
    // 验证数组长度一致
    assert_eq!(message.len(), deserialized.len());
    // 验证每个消息段一致
    for (original, recovered) in message.iter().zip(deserialized.iter()) {
        assert_eq!(original, recovered);
    }
}

/// 测试 MiniApp 消息段的 flatten 行为
///
/// 验证 MiniApp 消息段的 data 字段作为 Value 正确序列化和反序列化，
/// 保持动态 JSON 结构完整。
#[test]
fn test_miniapp_flatten() {
    // 构造一个带复杂数据的 MiniApp 消息段
    let seg = MessageSegment::MiniApp {
        data: json!({
            "appid": "12345",
            "title": "测试小程序",
            "nested": {
                "key": "value"
            }
        }),
    };
    // 序列化为 JSON Value
    let json_val = serde_json::to_value(&seg).expect("MiniApp 消息段序列化应成功");
    // 验证 type 字段为 "miniapp"
    assert_eq!(json_val["type"], "miniapp");
    // 验证 data 中的字段直接可访问
    assert_eq!(json_val["data"]["appid"], "12345");
    assert_eq!(json_val["data"]["title"], "测试小程序");
    assert_eq!(json_val["data"]["nested"]["key"], "value");

    // 反序列化回来验证数据一致
    let recovered: MessageSegment =
        serde_json::from_value(json_val).expect("MiniApp 消息段反序列化应成功");
    assert_eq!(seg, recovered);
}

/// 测试 Node 消息段的嵌套消息段
///
/// 验证合并转发节点中的 content 字段（Vec<MessageSegment>）
/// 能正确序列化和反序列化嵌套的消息段数组。
#[test]
fn test_node_nested_content() {
    // 构造一个包含嵌套消息段的转发节点
    let seg = MessageSegment::Node {
        id: None,
        user_id: Some("10086".to_string()),
        nickname: Some("机器人".to_string()),
        content: Some(vec![
            MessageSegment::text("节点消息文本"),
            MessageSegment::image("node-image.png"),
            MessageSegment::face("178"),
        ]),
    };
    // 序列化为 JSON Value
    let json_val = serde_json::to_value(&seg).expect("Node 消息段序列化应成功");
    // 验证 type 字段
    assert_eq!(json_val["type"], "node");
    // 验证 content 是数组且长度为 3
    let content_arr = json_val["data"]["content"]
        .as_array()
        .expect("content 应为数组");
    assert_eq!(content_arr.len(), 3);
    // 验证嵌套消息段的类型
    assert_eq!(content_arr[0]["type"], "text");
    assert_eq!(content_arr[1]["type"], "image");
    assert_eq!(content_arr[2]["type"], "face");

    // 往返测试
    let json_str = serde_json::to_string(&seg).expect("序列化应成功");
    let recovered: MessageSegment =
        serde_json::from_str(&json_str).expect("反序列化应成功");
    assert_eq!(seg, recovered);
}

/// 测试所有消息段类型的 JSON type 字段正确性
///
/// 验证每种消息段序列化后的 "type" 字段值与 OneBot 11 协议一致。
#[test]
fn test_all_type_names() {
    // 构建消息段类型到期望 type 名称的映射表
    let test_cases: Vec<(MessageSegment, &str)> = vec![
        (MessageSegment::text("t"), "text"),
        (MessageSegment::at("1"), "at"),
        (MessageSegment::face("1"), "face"),
        (MessageSegment::reply("1"), "reply"),
        (MessageSegment::image("f"), "image"),
        (MessageSegment::record("f"), "record"),
        (MessageSegment::video("f"), "video"),
        (
            MessageSegment::File {
                file: "f".into(),
                name: None,
                url: None,
            },
            "file",
        ),
        (MessageSegment::json("d"), "json"),
        (
            MessageSegment::Xml {
                data: "d".into(),
            },
            "xml",
        ),
        (MessageSegment::markdown("c"), "markdown"),
        (
            MessageSegment::Poke {
                poke_type: "1".into(),
                id: "1".into(),
            },
            "poke",
        ),
        (MessageSegment::Dice { result: None }, "dice"),
        (MessageSegment::Rps { result: None }, "rps"),
        (
            MessageSegment::MFace {
                emoji_id: None,
                emoji_package_id: None,
                key: None,
                summary: None,
            },
            "mface",
        ),
        (
            MessageSegment::Music {
                music_type: "qq".into(),
                id: None,
                url: None,
                audio: None,
                title: None,
                content: None,
                image: None,
            },
            "music",
        ),
        (
            MessageSegment::Node {
                id: None,
                user_id: None,
                nickname: None,
                content: None,
            },
            "node",
        ),
        (
            MessageSegment::Forward {
                id: "fwd".into(),
            },
            "forward",
        ),
        (
            MessageSegment::Contact {
                contact_type: "qq".into(),
                id: "1".into(),
            },
            "contact",
        ),
        (
            MessageSegment::Location {
                lat: "0".into(),
                lon: "0".into(),
                title: None,
                content: None,
            },
            "location",
        ),
        (
            MessageSegment::MiniApp {
                data: json!({}),
            },
            "miniapp",
        ),
    ];

    // 遍历所有测试用例，验证 type 字段
    for (seg, expected_type) in &test_cases {
        // 序列化为 JSON Value
        let json_val: Value = serde_json::to_value(seg)
            .unwrap_or_else(|e| panic!("类型 '{}' 序列化失败: {}", expected_type, e));
        // 验证 type 字段值
        assert_eq!(
            json_val["type"].as_str().unwrap(),
            *expected_type,
            "消息段类型名不匹配"
        );
    }
}
