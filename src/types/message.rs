//! # 消息类型模块
//!
//! 定义 OneBot 11 协议中的消息段（MessageSegment）数据结构，
//! 包括文本、图片、表情、At、回复、转发等消息类型，
//! 以及 NapCat 扩展的 Markdown、MiniApp 等消息类型。
//!
//! ## 设计说明
//!
//! OneBot 11 协议的消息段格式为：
//! ```json
//! { "type": "text", "data": { "text": "hello" } }
//! ```
//!
//! 由于 serde 的 `#[serde(tag, content)]` 不支持在单个变体上使用 `untagged`，
//! 本模块通过自定义 `Serialize`/`Deserialize` 实现来支持已知类型的精确匹配
//! 和未知类型的 `Unknown` 兜底。

// 引入 serde 序列化/反序列化 trait 和派生宏
use serde::{Deserialize, Deserializer, Serialize, Serializer};
// 引入 serde_json 的 Value 类型，用于处理动态 JSON 数据
use serde_json::Value;

/// OneBot 11 消息段枚举
///
/// 表示 OneBot 11 协议中所有支持的消息段类型。
/// 每个变体对应一种消息段类型（如文本、图片、@、回复等），
/// 并携带该类型所需的数据字段。
///
/// ## 序列化格式
///
/// 所有消息段序列化为 `{ "type": "<类型名>", "data": { ... } }` 格式。
///
/// ## 未知类型兜底
///
/// 对于未知的消息段类型，使用 `Unknown` 变体捕获，保留原始 type 和 data。
#[derive(Debug, Clone, PartialEq)]
pub enum MessageSegment {
    /// 文本消息段
    ///
    /// 最基础的消息类型，包含纯文本内容。
    /// 对应 OneBot 11 type = "text"。
    Text {
        /// 文本内容
        text: String,
    },

    /// @某人消息段
    ///
    /// 在群聊中 @ 指定成员或全体成员。
    /// 对应 OneBot 11 type = "at"。
    /// 当 qq = "all" 时表示 @全体成员。
    At {
        /// 被 @ 的 QQ 号，"all" 表示全体成员
        qq: String,
        /// 被 @ 成员的昵称（可选，由服务端填充）
        #[allow(dead_code)]
        name: Option<String>,
    },

    /// 表情消息段
    ///
    /// QQ 系统表情，通过 ID 标识。
    /// 对应 OneBot 11 type = "face"。
    Face {
        /// 表情 ID
        id: String,
    },

    /// 回复消息段
    ///
    /// 引用回复指定消息。
    /// 对应 OneBot 11 type = "reply"。
    Reply {
        /// 被回复消息的 ID
        id: String,
    },

    /// 图片消息段
    ///
    /// 发送或接收图片，支持本地文件、URL、Base64 等格式。
    /// 对应 OneBot 11 type = "image"。
    Image {
        /// 图片文件路径、URL 或 Base64 编码
        file: String,
        /// 图片摘要/描述（可选，NapCat 扩展）
        summary: Option<String>,
        /// 图片子类型（可选，NapCat 扩展，如 "normal"、"sticker" 等）
        sub_type: Option<String>,
        /// 图片下载 URL（可选，接收消息时由服务端填充）
        url: Option<String>,
    },

    /// 语音消息段
    ///
    /// 发送或接收语音消息。
    /// 对应 OneBot 11 type = "record"。
    Record {
        /// 语音文件路径、URL 或 Base64 编码
        file: String,
        /// 语音下载 URL（可选，接收消息时由服务端填充）
        url: Option<String>,
    },

    /// 视频消息段
    ///
    /// 发送或接收短视频。
    /// 对应 OneBot 11 type = "video"。
    Video {
        /// 视频文件路径、URL 或 Base64 编码
        file: String,
        /// 视频下载 URL（可选，接收消息时由服务端填充）
        url: Option<String>,
    },

    /// 文件消息段
    ///
    /// 发送或接收文件（群文件/私聊文件）。
    /// 对应 OneBot 11 type = "file"。
    File {
        /// 文件路径或 URL
        file: String,
        /// 文件显示名称（可选）
        name: Option<String>,
        /// 文件下载 URL（可选，接收消息时由服务端填充）
        url: Option<String>,
    },

    /// JSON 消息段
    ///
    /// 发送 JSON 格式的卡片消息（如分享链接、小程序等）。
    /// 对应 OneBot 11 type = "json"。
    Json {
        /// JSON 数据字符串
        data: String,
    },

    /// XML 消息段
    ///
    /// 发送 XML 格式的富文本消息。
    /// 对应 OneBot 11 type = "xml"。
    Xml {
        /// XML 数据字符串
        data: String,
    },

    /// Markdown 消息段（NapCat 扩展）
    ///
    /// 发送 Markdown 格式的消息。
    /// 对应 NapCat 扩展 type = "markdown"。
    Markdown {
        /// Markdown 内容
        content: String,
    },

    /// 戳一戳消息段
    ///
    /// 发送窗口抖动/戳一戳效果。
    /// 对应 OneBot 11 type = "poke"。
    Poke {
        /// 戳一戳类型（序列化时重命名为 "type"）
        poke_type: String,
        /// 戳一戳 ID
        id: String,
    },

    /// 掷骰子消息段
    ///
    /// 发送随机骰子结果（1-6）。
    /// 对应 OneBot 11 type = "dice"。
    Dice {
        /// 骰子结果（可选，"1"-"6"，发送时可不指定由服务端随机生成）
        result: Option<String>,
    },

    /// 猜拳消息段
    ///
    /// 发送石头剪刀布结果。
    /// 对应 OneBot 11 type = "rps"。
    Rps {
        /// 猜拳结果（可选，发送时可不指定由服务端随机生成）
        result: Option<String>,
    },

    /// 商城表情消息段（NapCat 扩展）
    ///
    /// QQ 商城的大表情，通过表情包 ID 和表情 ID 标识。
    /// 对应 NapCat 扩展 type = "mface"。
    MFace {
        /// 表情 ID（可选）
        emoji_id: Option<String>,
        /// 表情包 ID（可选）
        emoji_package_id: Option<String>,
        /// 表情密钥（可选）
        key: Option<String>,
        /// 表情描述/摘要（可选）
        summary: Option<String>,
    },

    /// 音乐分享消息段
    ///
    /// 分享音乐卡片，支持 QQ音乐、网易云音乐等平台，也支持自定义音乐卡片。
    /// 对应 OneBot 11 type = "music"。
    Music {
        /// 音乐平台类型（序列化时重命名为 "type"，如 "qq"、"163"、"custom"）
        music_type: String,
        /// 音乐 ID（非自定义音乐时使用）
        id: Option<String>,
        /// 音乐链接 URL（自定义音乐时使用）
        url: Option<String>,
        /// 音频文件 URL（自定义音乐时使用）
        audio: Option<String>,
        /// 音乐标题（自定义音乐时使用）
        title: Option<String>,
        /// 音乐描述/歌手名（自定义音乐时使用）
        content: Option<String>,
        /// 封面图片 URL（自定义音乐时使用）
        image: Option<String>,
    },

    /// 合并转发节点消息段
    ///
    /// 用于构建合并转发消息的单个节点。
    /// 对应 OneBot 11 type = "node"。
    /// 支持引用已有消息（通过 id）或自定义节点（通过 user_id + nickname + content）。
    Node {
        /// 被引用的消息 ID（可选，引用模式）
        id: Option<String>,
        /// 发送者 QQ 号（可选，自定义节点模式）
        user_id: Option<String>,
        /// 发送者昵称（可选，自定义节点模式）
        nickname: Option<String>,
        /// 节点消息内容（可选，自定义节点模式，包含一组消息段）
        content: Option<Vec<MessageSegment>>,
    },

    /// 合并转发消息段
    ///
    /// 接收合并转发消息时使用，包含转发消息的 ID。
    /// 对应 OneBot 11 type = "forward"。
    Forward {
        /// 合并转发消息的 ID
        id: String,
    },

    /// 推荐联系人/群消息段
    ///
    /// 分享好友名片或群名片。
    /// 对应 OneBot 11 type = "contact"。
    Contact {
        /// 联系人类型（序列化时重命名为 "type"，"qq" 表示好友，"group" 表示群）
        contact_type: String,
        /// QQ 号或群号
        id: String,
    },

    /// 位置消息段
    ///
    /// 分享地理位置信息。
    /// 对应 OneBot 11 type = "location"。
    Location {
        /// 纬度
        lat: String,
        /// 经度
        lon: String,
        /// 位置标题/名称（可选）
        title: Option<String>,
        /// 位置详细描述/地址（可选）
        content: Option<String>,
    },

    /// 小程序消息段（NapCat 扩展）
    ///
    /// 发送 QQ 小程序卡片，数据为动态 JSON 格式。
    /// 对应 NapCat 扩展 type = "miniapp"。
    /// 使用 flatten 将 data 中的所有字段直接展开。
    MiniApp {
        /// 小程序数据（动态 JSON 对象，序列化时 flatten 展开）
        data: Value,
    },

    /// 未知消息段（兜底类型）
    ///
    /// 当接收到不在已知列表中的消息段类型时，使用此变体捕获。
    /// 保留原始的 type 字符串和 data JSON 值，确保数据不丢失。
    Unknown {
        /// 原始消息段类型名称
        r#type: String,
        /// 原始消息段数据（完整的 JSON 值）
        data: Value,
    },
}

// ============================================================================
// 辅助数据结构：用于各消息段变体的内部序列化/反序列化
// ============================================================================

/// 文本消息段数据
///
/// 对应 `{ "text": "..." }` 结构。
#[derive(Serialize, Deserialize)]
struct TextData {
    /// 文本内容
    text: String,
}

/// @消息段数据
///
/// 对应 `{ "qq": "...", "name": "..." }` 结构。
#[derive(Serialize, Deserialize)]
struct AtData {
    /// 被 @ 的 QQ 号或 "all"
    qq: String,
    /// 被 @ 成员的昵称（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

/// 表情消息段数据
///
/// 对应 `{ "id": "..." }` 结构。
#[derive(Serialize, Deserialize)]
struct FaceData {
    /// 表情 ID
    id: String,
}

/// 回复消息段数据
///
/// 对应 `{ "id": "..." }` 结构。
#[derive(Serialize, Deserialize)]
struct ReplyData {
    /// 被回复消息的 ID
    id: String,
}

/// 图片消息段数据
///
/// 对应 `{ "file": "...", "summary": "...", "sub_type": "...", "url": "..." }` 结构。
#[derive(Serialize, Deserialize)]
struct ImageData {
    /// 图片文件路径、URL 或 Base64
    file: String,
    /// 图片摘要（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
    /// 图片子类型（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    sub_type: Option<String>,
    /// 图片下载 URL（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
}

/// 语音消息段数据
///
/// 对应 `{ "file": "...", "url": "..." }` 结构。
#[derive(Serialize, Deserialize)]
struct RecordData {
    /// 语音文件路径或 URL
    file: String,
    /// 语音下载 URL（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
}

/// 视频消息段数据
///
/// 对应 `{ "file": "...", "url": "..." }` 结构。
#[derive(Serialize, Deserialize)]
struct VideoData {
    /// 视频文件路径或 URL
    file: String,
    /// 视频下载 URL（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
}

/// 文件消息段数据
///
/// 对应 `{ "file": "...", "name": "...", "url": "..." }` 结构。
#[derive(Serialize, Deserialize)]
struct FileData {
    /// 文件路径或 URL
    file: String,
    /// 文件名称（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    /// 文件下载 URL（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
}

/// JSON 消息段数据
///
/// 对应 `{ "data": "..." }` 结构。
#[derive(Serialize, Deserialize)]
struct JsonData {
    /// JSON 数据字符串
    data: String,
}

/// XML 消息段数据
///
/// 对应 `{ "data": "..." }` 结构。
#[derive(Serialize, Deserialize)]
struct XmlData {
    /// XML 数据字符串
    data: String,
}

/// Markdown 消息段数据
///
/// 对应 `{ "content": "..." }` 结构。
#[derive(Serialize, Deserialize)]
struct MarkdownData {
    /// Markdown 内容
    content: String,
}

/// 戳一戳消息段数据
///
/// 对应 `{ "type": "...", "id": "..." }` 结构。
/// 注意：`poke_type` 序列化时重命名为 "type"。
#[derive(Serialize, Deserialize)]
struct PokeData {
    /// 戳一戳类型（在 JSON 中为 "type" 字段）
    #[serde(rename = "type")]
    poke_type: String,
    /// 戳一戳 ID
    id: String,
}

/// 掷骰子消息段数据
///
/// 对应 `{ "result": "..." }` 结构。
#[derive(Serialize, Deserialize)]
struct DiceData {
    /// 骰子结果（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<String>,
}

/// 猜拳消息段数据
///
/// 对应 `{ "result": "..." }` 结构。
#[derive(Serialize, Deserialize)]
struct RpsData {
    /// 猜拳结果（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<String>,
}

/// 商城表情消息段数据
///
/// 对应 `{ "emoji_id": "...", "emoji_package_id": "...", "key": "...", "summary": "..." }` 结构。
#[derive(Serialize, Deserialize)]
struct MFaceData {
    /// 表情 ID（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    emoji_id: Option<String>,
    /// 表情包 ID（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    emoji_package_id: Option<String>,
    /// 表情密钥（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    key: Option<String>,
    /// 表情描述（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
}

/// 音乐分享消息段数据
///
/// 对应 `{ "type": "...", "id": "...", "url": "...", ... }` 结构。
/// 注意：`music_type` 序列化时重命名为 "type"。
#[derive(Serialize, Deserialize)]
struct MusicData {
    /// 音乐平台类型（在 JSON 中为 "type" 字段）
    #[serde(rename = "type")]
    music_type: String,
    /// 音乐 ID（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    /// 音乐链接 URL（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    /// 音频文件 URL（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    audio: Option<String>,
    /// 音乐标题（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    /// 音乐描述（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    /// 封面图片 URL（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<String>,
}

/// 合并转发节点消息段数据
///
/// 对应 `{ "id": "...", "user_id": "...", "nickname": "...", "content": [...] }` 结构。
#[derive(Serialize, Deserialize)]
struct NodeData {
    /// 被引用的消息 ID（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    /// 发送者 QQ 号（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    user_id: Option<String>,
    /// 发送者昵称（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    nickname: Option<String>,
    /// 节点消息内容（可选，消息段数组）
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<Vec<MessageSegment>>,
}

/// 合并转发消息段数据
///
/// 对应 `{ "id": "..." }` 结构。
#[derive(Serialize, Deserialize)]
struct ForwardData {
    /// 合并转发消息 ID
    id: String,
}

/// 推荐联系人消息段数据
///
/// 对应 `{ "type": "...", "id": "..." }` 结构。
/// 注意：`contact_type` 序列化时重命名为 "type"。
#[derive(Serialize, Deserialize)]
struct ContactData {
    /// 联系人类型（在 JSON 中为 "type" 字段）
    #[serde(rename = "type")]
    contact_type: String,
    /// QQ 号或群号
    id: String,
}

/// 位置消息段数据
///
/// 对应 `{ "lat": "...", "lon": "...", "title": "...", "content": "..." }` 结构。
#[derive(Serialize, Deserialize)]
struct LocationData {
    /// 纬度
    lat: String,
    /// 经度
    lon: String,
    /// 位置标题（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    /// 位置描述（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
}

// ============================================================================
// 原始 JSON 表示结构：用于自定义序列化/反序列化的中间格式
// ============================================================================

/// 消息段的原始 JSON 表示
///
/// 作为序列化和反序列化的中间格式，直接映射 OneBot 11 的
/// `{ "type": "...", "data": { ... } }` 结构。
#[derive(Serialize, Deserialize)]
struct RawSegment {
    /// 消息段类型名称
    r#type: String,
    /// 消息段数据（JSON 对象）
    data: Value,
}

// ============================================================================
// 自定义 Serialize 实现
// ============================================================================

/// 为 MessageSegment 实现自定义序列化
///
/// 将每个变体序列化为 `{ "type": "<类型名>", "data": { ... } }` 格式。
/// 对于 MiniApp 类型，data 字段使用 flatten 方式展开。
/// 对于 Unknown 类型，直接使用原始的 type 和 data 值。
impl Serialize for MessageSegment {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 根据不同的消息段变体，构造对应的原始 JSON 表示
        let raw = match self {
            // 文本消息段：type = "text"
            MessageSegment::Text { text } => RawSegment {
                r#type: "text".to_string(),
                data: serde_json::to_value(TextData {
                    text: text.clone(),
                })
                .map_err(serde::ser::Error::custom)?,
            },
            // @消息段：type = "at"
            MessageSegment::At { qq, name } => RawSegment {
                r#type: "at".to_string(),
                data: serde_json::to_value(AtData {
                    qq: qq.clone(),
                    name: name.clone(),
                })
                .map_err(serde::ser::Error::custom)?,
            },
            // 表情消息段：type = "face"
            MessageSegment::Face { id } => RawSegment {
                r#type: "face".to_string(),
                data: serde_json::to_value(FaceData { id: id.clone() })
                    .map_err(serde::ser::Error::custom)?,
            },
            // 回复消息段：type = "reply"
            MessageSegment::Reply { id } => RawSegment {
                r#type: "reply".to_string(),
                data: serde_json::to_value(ReplyData { id: id.clone() })
                    .map_err(serde::ser::Error::custom)?,
            },
            // 图片消息段：type = "image"
            MessageSegment::Image {
                file,
                summary,
                sub_type,
                url,
            } => RawSegment {
                r#type: "image".to_string(),
                data: serde_json::to_value(ImageData {
                    file: file.clone(),
                    summary: summary.clone(),
                    sub_type: sub_type.clone(),
                    url: url.clone(),
                })
                .map_err(serde::ser::Error::custom)?,
            },
            // 语音消息段：type = "record"
            MessageSegment::Record { file, url } => RawSegment {
                r#type: "record".to_string(),
                data: serde_json::to_value(RecordData {
                    file: file.clone(),
                    url: url.clone(),
                })
                .map_err(serde::ser::Error::custom)?,
            },
            // 视频消息段：type = "video"
            MessageSegment::Video { file, url } => RawSegment {
                r#type: "video".to_string(),
                data: serde_json::to_value(VideoData {
                    file: file.clone(),
                    url: url.clone(),
                })
                .map_err(serde::ser::Error::custom)?,
            },
            // 文件消息段：type = "file"
            MessageSegment::File { file, name, url } => RawSegment {
                r#type: "file".to_string(),
                data: serde_json::to_value(FileData {
                    file: file.clone(),
                    name: name.clone(),
                    url: url.clone(),
                })
                .map_err(serde::ser::Error::custom)?,
            },
            // JSON 消息段：type = "json"
            MessageSegment::Json { data } => RawSegment {
                r#type: "json".to_string(),
                data: serde_json::to_value(JsonData { data: data.clone() })
                    .map_err(serde::ser::Error::custom)?,
            },
            // XML 消息段：type = "xml"
            MessageSegment::Xml { data } => RawSegment {
                r#type: "xml".to_string(),
                data: serde_json::to_value(XmlData { data: data.clone() })
                    .map_err(serde::ser::Error::custom)?,
            },
            // Markdown 消息段：type = "markdown"
            MessageSegment::Markdown { content } => RawSegment {
                r#type: "markdown".to_string(),
                data: serde_json::to_value(MarkdownData {
                    content: content.clone(),
                })
                .map_err(serde::ser::Error::custom)?,
            },
            // 戳一戳消息段：type = "poke"
            MessageSegment::Poke { poke_type, id } => RawSegment {
                r#type: "poke".to_string(),
                data: serde_json::to_value(PokeData {
                    poke_type: poke_type.clone(),
                    id: id.clone(),
                })
                .map_err(serde::ser::Error::custom)?,
            },
            // 掷骰子消息段：type = "dice"
            MessageSegment::Dice { result } => RawSegment {
                r#type: "dice".to_string(),
                data: serde_json::to_value(DiceData {
                    result: result.clone(),
                })
                .map_err(serde::ser::Error::custom)?,
            },
            // 猜拳消息段：type = "rps"
            MessageSegment::Rps { result } => RawSegment {
                r#type: "rps".to_string(),
                data: serde_json::to_value(RpsData {
                    result: result.clone(),
                })
                .map_err(serde::ser::Error::custom)?,
            },
            // 商城表情消息段：type = "mface"
            MessageSegment::MFace {
                emoji_id,
                emoji_package_id,
                key,
                summary,
            } => RawSegment {
                r#type: "mface".to_string(),
                data: serde_json::to_value(MFaceData {
                    emoji_id: emoji_id.clone(),
                    emoji_package_id: emoji_package_id.clone(),
                    key: key.clone(),
                    summary: summary.clone(),
                })
                .map_err(serde::ser::Error::custom)?,
            },
            // 音乐分享消息段：type = "music"
            MessageSegment::Music {
                music_type,
                id,
                url,
                audio,
                title,
                content,
                image,
            } => RawSegment {
                r#type: "music".to_string(),
                data: serde_json::to_value(MusicData {
                    music_type: music_type.clone(),
                    id: id.clone(),
                    url: url.clone(),
                    audio: audio.clone(),
                    title: title.clone(),
                    content: content.clone(),
                    image: image.clone(),
                })
                .map_err(serde::ser::Error::custom)?,
            },
            // 合并转发节点消息段：type = "node"
            MessageSegment::Node {
                id,
                user_id,
                nickname,
                content,
            } => RawSegment {
                r#type: "node".to_string(),
                data: serde_json::to_value(NodeData {
                    id: id.clone(),
                    user_id: user_id.clone(),
                    nickname: nickname.clone(),
                    content: content.clone(),
                })
                .map_err(serde::ser::Error::custom)?,
            },
            // 合并转发消息段：type = "forward"
            MessageSegment::Forward { id } => RawSegment {
                r#type: "forward".to_string(),
                data: serde_json::to_value(ForwardData { id: id.clone() })
                    .map_err(serde::ser::Error::custom)?,
            },
            // 推荐联系人消息段：type = "contact"
            MessageSegment::Contact { contact_type, id } => RawSegment {
                r#type: "contact".to_string(),
                data: serde_json::to_value(ContactData {
                    contact_type: contact_type.clone(),
                    id: id.clone(),
                })
                .map_err(serde::ser::Error::custom)?,
            },
            // 位置消息段：type = "location"
            MessageSegment::Location {
                lat,
                lon,
                title,
                content,
            } => RawSegment {
                r#type: "location".to_string(),
                data: serde_json::to_value(LocationData {
                    lat: lat.clone(),
                    lon: lon.clone(),
                    title: title.clone(),
                    content: content.clone(),
                })
                .map_err(serde::ser::Error::custom)?,
            },
            // 小程序消息段：type = "miniapp"，data 使用 flatten 展开
            MessageSegment::MiniApp { data } => RawSegment {
                r#type: "miniapp".to_string(),
                data: data.clone(),
            },
            // 未知消息段：直接使用原始的 type 和 data
            MessageSegment::Unknown { r#type, data } => RawSegment {
                r#type: r#type.clone(),
                data: data.clone(),
            },
        };
        // 将构造的原始 JSON 表示序列化输出
        raw.serialize(serializer)
    }
}

// ============================================================================
// 自定义 Deserialize 实现
// ============================================================================

/// 为 MessageSegment 实现自定义反序列化
///
/// 从 `{ "type": "<类型名>", "data": { ... } }` 格式反序列化。
/// 先提取 type 字段，然后根据类型名将 data 反序列化为对应的结构体。
/// 未识别的类型名会被反序列化为 `Unknown` 变体。
impl<'de> Deserialize<'de> for MessageSegment {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // 先反序列化为原始的 { type, data } 结构
        let raw = RawSegment::deserialize(deserializer)?;

        // 根据 type 字段值匹配对应的消息段变体
        match raw.r#type.as_str() {
            // 文本消息段
            "text" => {
                // 将 data 反序列化为 TextData 结构
                let d: TextData =
                    serde_json::from_value(raw.data).map_err(serde::de::Error::custom)?;
                // 构造 Text 变体
                Ok(MessageSegment::Text { text: d.text })
            }
            // @消息段
            "at" => {
                // 将 data 反序列化为 AtData 结构
                let d: AtData =
                    serde_json::from_value(raw.data).map_err(serde::de::Error::custom)?;
                // 构造 At 变体
                Ok(MessageSegment::At {
                    qq: d.qq,
                    name: d.name,
                })
            }
            // 表情消息段
            "face" => {
                // 将 data 反序列化为 FaceData 结构
                let d: FaceData =
                    serde_json::from_value(raw.data).map_err(serde::de::Error::custom)?;
                // 构造 Face 变体
                Ok(MessageSegment::Face { id: d.id })
            }
            // 回复消息段
            "reply" => {
                // 将 data 反序列化为 ReplyData 结构
                let d: ReplyData =
                    serde_json::from_value(raw.data).map_err(serde::de::Error::custom)?;
                // 构造 Reply 变体
                Ok(MessageSegment::Reply { id: d.id })
            }
            // 图片消息段
            "image" => {
                // 将 data 反序列化为 ImageData 结构
                let d: ImageData =
                    serde_json::from_value(raw.data).map_err(serde::de::Error::custom)?;
                // 构造 Image 变体
                Ok(MessageSegment::Image {
                    file: d.file,
                    summary: d.summary,
                    sub_type: d.sub_type,
                    url: d.url,
                })
            }
            // 语音消息段
            "record" => {
                // 将 data 反序列化为 RecordData 结构
                let d: RecordData =
                    serde_json::from_value(raw.data).map_err(serde::de::Error::custom)?;
                // 构造 Record 变体
                Ok(MessageSegment::Record {
                    file: d.file,
                    url: d.url,
                })
            }
            // 视频消息段
            "video" => {
                // 将 data 反序列化为 VideoData 结构
                let d: VideoData =
                    serde_json::from_value(raw.data).map_err(serde::de::Error::custom)?;
                // 构造 Video 变体
                Ok(MessageSegment::Video {
                    file: d.file,
                    url: d.url,
                })
            }
            // 文件消息段
            "file" => {
                // 将 data 反序列化为 FileData 结构
                let d: FileData =
                    serde_json::from_value(raw.data).map_err(serde::de::Error::custom)?;
                // 构造 File 变体
                Ok(MessageSegment::File {
                    file: d.file,
                    name: d.name,
                    url: d.url,
                })
            }
            // JSON 消息段
            "json" => {
                // 将 data 反序列化为 JsonData 结构
                let d: JsonData =
                    serde_json::from_value(raw.data).map_err(serde::de::Error::custom)?;
                // 构造 Json 变体
                Ok(MessageSegment::Json { data: d.data })
            }
            // XML 消息段
            "xml" => {
                // 将 data 反序列化为 XmlData 结构
                let d: XmlData =
                    serde_json::from_value(raw.data).map_err(serde::de::Error::custom)?;
                // 构造 Xml 变体
                Ok(MessageSegment::Xml { data: d.data })
            }
            // Markdown 消息段
            "markdown" => {
                // 将 data 反序列化为 MarkdownData 结构
                let d: MarkdownData =
                    serde_json::from_value(raw.data).map_err(serde::de::Error::custom)?;
                // 构造 Markdown 变体
                Ok(MessageSegment::Markdown { content: d.content })
            }
            // 戳一戳消息段
            "poke" => {
                // 将 data 反序列化为 PokeData 结构
                let d: PokeData =
                    serde_json::from_value(raw.data).map_err(serde::de::Error::custom)?;
                // 构造 Poke 变体
                Ok(MessageSegment::Poke {
                    poke_type: d.poke_type,
                    id: d.id,
                })
            }
            // 掷骰子消息段
            "dice" => {
                // 将 data 反序列化为 DiceData 结构
                let d: DiceData =
                    serde_json::from_value(raw.data).map_err(serde::de::Error::custom)?;
                // 构造 Dice 变体
                Ok(MessageSegment::Dice { result: d.result })
            }
            // 猜拳消息段
            "rps" => {
                // 将 data 反序列化为 RpsData 结构
                let d: RpsData =
                    serde_json::from_value(raw.data).map_err(serde::de::Error::custom)?;
                // 构造 Rps 变体
                Ok(MessageSegment::Rps { result: d.result })
            }
            // 商城表情消息段
            "mface" => {
                // 将 data 反序列化为 MFaceData 结构
                let d: MFaceData =
                    serde_json::from_value(raw.data).map_err(serde::de::Error::custom)?;
                // 构造 MFace 变体
                Ok(MessageSegment::MFace {
                    emoji_id: d.emoji_id,
                    emoji_package_id: d.emoji_package_id,
                    key: d.key,
                    summary: d.summary,
                })
            }
            // 音乐分享消息段
            "music" => {
                // 将 data 反序列化为 MusicData 结构
                let d: MusicData =
                    serde_json::from_value(raw.data).map_err(serde::de::Error::custom)?;
                // 构造 Music 变体
                Ok(MessageSegment::Music {
                    music_type: d.music_type,
                    id: d.id,
                    url: d.url,
                    audio: d.audio,
                    title: d.title,
                    content: d.content,
                    image: d.image,
                })
            }
            // 合并转发节点消息段
            "node" => {
                // 将 data 反序列化为 NodeData 结构
                let d: NodeData =
                    serde_json::from_value(raw.data).map_err(serde::de::Error::custom)?;
                // 构造 Node 变体
                Ok(MessageSegment::Node {
                    id: d.id,
                    user_id: d.user_id,
                    nickname: d.nickname,
                    content: d.content,
                })
            }
            // 合并转发消息段
            "forward" => {
                // 将 data 反序列化为 ForwardData 结构
                let d: ForwardData =
                    serde_json::from_value(raw.data).map_err(serde::de::Error::custom)?;
                // 构造 Forward 变体
                Ok(MessageSegment::Forward { id: d.id })
            }
            // 推荐联系人消息段
            "contact" => {
                // 将 data 反序列化为 ContactData 结构
                let d: ContactData =
                    serde_json::from_value(raw.data).map_err(serde::de::Error::custom)?;
                // 构造 Contact 变体
                Ok(MessageSegment::Contact {
                    contact_type: d.contact_type,
                    id: d.id,
                })
            }
            // 位置消息段
            "location" => {
                // 将 data 反序列化为 LocationData 结构
                let d: LocationData =
                    serde_json::from_value(raw.data).map_err(serde::de::Error::custom)?;
                // 构造 Location 变体
                Ok(MessageSegment::Location {
                    lat: d.lat,
                    lon: d.lon,
                    title: d.title,
                    content: d.content,
                })
            }
            // 小程序消息段
            "miniapp" => {
                // MiniApp 的 data 字段直接作为 Value 保留
                Ok(MessageSegment::MiniApp { data: raw.data })
            }
            // 未知消息段类型：兜底处理，保留原始数据
            _ => Ok(MessageSegment::Unknown {
                r#type: raw.r#type,
                data: raw.data,
            }),
        }
    }
}

// ============================================================================
// 快捷构造方法
// ============================================================================

impl MessageSegment {
    /// 创建文本消息段
    ///
    /// # 参数
    ///
    /// * `text` - 文本内容，接受任何可转换为 String 的类型
    ///
    /// # 示例
    ///
    /// ```rust
    /// use napcat_link::types::message::MessageSegment;
    ///
    /// let seg = MessageSegment::text("Hello, world!");
    /// ```
    pub fn text(text: impl Into<String>) -> Self {
        // 构造 Text 变体，将参数转换为 String
        MessageSegment::Text { text: text.into() }
    }

    /// 创建 @某人消息段
    ///
    /// # 参数
    ///
    /// * `qq` - 被 @ 的 QQ 号，接受任何可转换为 String 的类型
    ///
    /// # 示例
    ///
    /// ```rust
    /// use napcat_link::types::message::MessageSegment;
    ///
    /// let seg = MessageSegment::at("123456789");
    /// ```
    pub fn at(qq: impl Into<String>) -> Self {
        // 构造 At 变体，name 默认为 None
        MessageSegment::At {
            qq: qq.into(),
            name: None,
        }
    }

    /// 创建 @全体成员消息段
    ///
    /// # 示例
    ///
    /// ```rust
    /// use napcat_link::types::message::MessageSegment;
    ///
    /// let seg = MessageSegment::at_all();
    /// ```
    pub fn at_all() -> Self {
        // 构造 At 变体，qq 为 "all" 表示全体成员
        MessageSegment::At {
            qq: "all".to_string(),
            name: None,
        }
    }

    /// 创建图片消息段
    ///
    /// # 参数
    ///
    /// * `file` - 图片文件路径、URL 或 Base64 编码，接受任何可转换为 String 的类型
    ///
    /// # 示例
    ///
    /// ```rust
    /// use napcat_link::types::message::MessageSegment;
    ///
    /// let seg = MessageSegment::image("https://example.com/image.png");
    /// ```
    pub fn image(file: impl Into<String>) -> Self {
        // 构造 Image 变体，可选字段默认为 None
        MessageSegment::Image {
            file: file.into(),
            summary: None,
            sub_type: None,
            url: None,
        }
    }

    /// 创建回复消息段
    ///
    /// # 参数
    ///
    /// * `id` - 被回复消息的 ID，接受任何可转换为 String 的类型
    ///
    /// # 示例
    ///
    /// ```rust
    /// use napcat_link::types::message::MessageSegment;
    ///
    /// let seg = MessageSegment::reply("12345");
    /// ```
    pub fn reply(id: impl Into<String>) -> Self {
        // 构造 Reply 变体
        MessageSegment::Reply { id: id.into() }
    }

    /// 创建表情消息段
    ///
    /// # 参数
    ///
    /// * `id` - 表情 ID，接受任何可转换为 String 的类型
    ///
    /// # 示例
    ///
    /// ```rust
    /// use napcat_link::types::message::MessageSegment;
    ///
    /// let seg = MessageSegment::face("178");
    /// ```
    pub fn face(id: impl Into<String>) -> Self {
        // 构造 Face 变体
        MessageSegment::Face { id: id.into() }
    }

    /// 创建语音消息段
    ///
    /// # 参数
    ///
    /// * `file` - 语音文件路径、URL 或 Base64 编码，接受任何可转换为 String 的类型
    ///
    /// # 示例
    ///
    /// ```rust
    /// use napcat_link::types::message::MessageSegment;
    ///
    /// let seg = MessageSegment::record("https://example.com/voice.amr");
    /// ```
    pub fn record(file: impl Into<String>) -> Self {
        // 构造 Record 变体，url 默认为 None
        MessageSegment::Record {
            file: file.into(),
            url: None,
        }
    }

    /// 创建视频消息段
    ///
    /// # 参数
    ///
    /// * `file` - 视频文件路径、URL 或 Base64 编码，接受任何可转换为 String 的类型
    ///
    /// # 示例
    ///
    /// ```rust
    /// use napcat_link::types::message::MessageSegment;
    ///
    /// let seg = MessageSegment::video("https://example.com/video.mp4");
    /// ```
    pub fn video(file: impl Into<String>) -> Self {
        // 构造 Video 变体，url 默认为 None
        MessageSegment::Video {
            file: file.into(),
            url: None,
        }
    }

    /// 创建 JSON 消息段
    ///
    /// # 参数
    ///
    /// * `data` - JSON 数据字符串，接受任何可转换为 String 的类型
    ///
    /// # 示例
    ///
    /// ```rust
    /// use napcat_link::types::message::MessageSegment;
    ///
    /// let seg = MessageSegment::json(r#"{"app":"com.example"}"#);
    /// ```
    pub fn json(data: impl Into<String>) -> Self {
        // 构造 Json 变体
        MessageSegment::Json { data: data.into() }
    }

    /// 创建 Markdown 消息段
    ///
    /// # 参数
    ///
    /// * `content` - Markdown 内容，接受任何可转换为 String 的类型
    ///
    /// # 示例
    ///
    /// ```rust
    /// use napcat_link::types::message::MessageSegment;
    ///
    /// let seg = MessageSegment::markdown("# Hello\n**bold text**");
    /// ```
    pub fn markdown(content: impl Into<String>) -> Self {
        // 构造 Markdown 变体
        MessageSegment::Markdown {
            content: content.into(),
        }
    }
}
