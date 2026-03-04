# NapCatLink API 参考文档

## OneBotApi 聚合器

所有 API 通过 `client.api()` 返回的 `OneBotApi` 结构体访问。

```rust
let api = client.api();
// api.message  — 消息 API
// api.group    — 群管理 API
// api.account  — 账号 API
// api.media    — 媒体 API
// api.file     — 文件 API
// api.stream   — 流式传输 API
// api.request  — 请求处理 API
// api.system   — 系统 API
// api.napcat   — NapCat 扩展 API
// api.raw      — 原始 action 调用
```

---

## MessageApi — 消息 API

文件：`src/api/message.rs`

| 方法 | OneBot Action | 说明 |
|------|---------------|------|
| `send_message(params)` | `send_msg` | 发送消息（通用） |
| `send_private_message(user_id, message)` | `send_private_msg` | 发送私聊消息 |
| `send_group_message(group_id, message)` | `send_group_msg` | 发送群聊消息 |
| `delete_message(message_id)` | `delete_msg` | 撤回消息 |
| `get_message(message_id)` | `get_msg` | 获取消息详情 |
| `get_forward_message(id)` | `get_forward_msg` | 获取合并转发消息 |
| `send_group_forward_message(group_id, messages)` | `send_group_forward_msg` | 发送群合并转发 |
| `set_essence_message(message_id)` | `set_essence_msg` | 设置精华消息 |
| `delete_essence_message(message_id)` | `delete_essence_msg` | 移除精华消息 |
| `get_essence_message_list(group_id)` | `get_essence_msg_list` | 获取精华消息列表 |
| `mark_message_as_read(message_id)` | `mark_msg_as_read` | 标记消息已读 |
| `mark_group_msg_as_read(group_id)` | `mark_group_msg_as_read` | 标记群消息已读 |
| `mark_private_msg_as_read(user_id)` | `mark_private_msg_as_read` | 标记私聊已读 |
| `mark_all_as_read()` | `_mark_all_as_read` | 标记全部已读 |
| `get_group_at_all_remain(group_id)` | `get_group_at_all_remain` | 获取 @全体 剩余次数 |
| `get_group_system_msg()` | `get_group_system_msg` | 获取群系统消息 |
| `get_group_honor_info(group_id, honor_type)` | `get_group_honor_info` | 获取群荣誉信息 |
| `get_group_msg_history(group_id, seq, count)` | `get_group_msg_history` | 获取群消息历史 |
| `get_friend_msg_history(user_id, seq, count)` | `get_friend_msg_history` | 获取好友消息历史 |
| `get_recent_contact(count)` | `get_recent_contact` | 获取最近联系人 |
| `set_msg_emoji_like(message_id, emoji_id)` | `set_msg_emoji_like` | 消息表情回应 |
| `fetch_emoji_like(message_id, emoji_id, type)` | `fetch_emoji_like` | 获取表情回应详情 |
| `group_poke(group_id, user_id)` | `group_poke` | 群戳一戳 |
| `friend_poke(user_id)` | `friend_poke` | 好友戳一戳 |
| `send_poke(user_id, group_id)` | `send_poke` | 发送戳一戳（通用） |

---

## GroupApi — 群管理 API

文件：`src/api/group.rs`

| 方法 | OneBot Action | 说明 |
|------|---------------|------|
| `set_group_ban(group_id, user_id, duration)` | `set_group_ban` | 禁言指定成员 |
| `unset_group_ban(group_id, user_id)` | `set_group_ban` | 解除禁言 |
| `set_group_whole_ban(group_id, enable)` | `set_group_whole_ban` | 全员禁言 |
| `set_group_kick(group_id, user_id, reject)` | `set_group_kick` | 踢出群成员 |
| `set_group_leave(group_id, is_dismiss)` | `set_group_leave` | 退出群组 |
| `set_group_card(group_id, user_id, card)` | `set_group_card` | 设置群名片 |
| `set_group_name(group_id, name)` | `set_group_name` | 设置群名 |
| `set_group_admin(group_id, user_id, enable)` | `set_group_admin` | 设置/取消管理员 |
| `set_group_anonymous_ban(group_id, flag, duration)` | `set_group_anonymous_ban` | 匿名禁言 |
| `set_group_special_title(group_id, user_id, title, duration)` | `set_group_special_title` | 设置专属头衔 |
| `send_like(user_id, times)` | `send_like` | 好友点赞 |

---

## AccountApi — 账号信息 API

文件：`src/api/account.rs`

| 方法 | OneBot Action | 说明 |
|------|---------------|------|
| `get_login_info()` | `get_login_info` | 获取登录号信息 |
| `get_status()` | `get_status` | 获取运行状态 |
| `get_friend_list()` | `get_friend_list` | 获取好友列表 |
| `get_group_list()` | `get_group_list` | 获取群列表 |
| `get_group_info(group_id, no_cache)` | `get_group_info` | 获取群信息 |
| `get_group_member_list(group_id)` | `get_group_member_list` | 获取群成员列表 |
| `get_group_member_info(group_id, user_id, no_cache)` | `get_group_member_info` | 获取群成员信息 |
| `get_stranger_info(user_id, no_cache)` | `get_stranger_info` | 获取陌生人信息 |
| `get_version_info()` | `get_version_info` | 获取版本信息 |

---

## MediaApi — 媒体资源 API

文件：`src/api/media.rs`

| 方法 | OneBot Action | 说明 |
|------|---------------|------|
| `get_image(file)` | `get_image` | 获取图片信息和 URL |
| `get_record(file, out_format)` | `get_record` | 获取语音文件 |
| `get_file(file_id)` | `get_file` | 获取文件信息 |
| `hydrate_media(message)` | — | 批量填充消息段中的媒体 URL |

---

## FileApi — 文件管理 API

文件：`src/api/file.rs`

| 方法 | OneBot Action | 说明 |
|------|---------------|------|
| `upload_group_file(group_id, file, name, folder)` | `upload_group_file` | 上传群文件 |
| `upload_private_file(user_id, file, name)` | `upload_private_file` | 上传私聊文件 |
| `get_group_file_system_info(group_id)` | `get_group_file_system_info` | 获取群文件系统信息 |
| `get_group_root_files(group_id)` | `get_group_root_files` | 获取群根目录文件 |
| `get_group_files_by_folder(group_id, folder_id)` | `get_group_files_by_folder` | 获取子目录文件 |
| `get_group_file_url(group_id, file_id, busid)` | `get_group_file_url` | 获取群文件 URL |
| `delete_group_file(group_id, file_id, busid)` | `delete_group_file` | 删除群文件 |
| `create_group_file_folder(group_id, name)` | `create_group_file_folder` | 创建群文件夹 |
| `delete_group_folder(group_id, folder_id)` | `delete_group_folder` | 删除群文件夹 |
| `download_file(url, thread_count, headers)` | `download_file` | 下载文件 |

---

## StreamApi — 流式传输 API

文件：`src/api/stream.rs`

| 方法 | OneBot Action | 说明 |
|------|---------------|------|
| `upload_file_stream(file, params)` | `upload_file_stream` | 流式上传文件 |
| `get_upload_stream_status(stream_id)` | `get_upload_stream_status` | 获取上传流状态 |
| `download_file_stream(file_id)` | `download_file_stream` | 流式下载文件 |
| `download_file_stream_to_file(file_id, path)` | `download_file_stream` | 流式下载到本地 |
| `download_file_image_stream(file_id)` | `download_file_image_stream` | 流式下载图片 |
| `download_file_record_stream(file_id)` | `download_file_record_stream` | 流式下载语音 |

> 注：当前版本流式 API 简化为单次 action 调用，完整分块传输将在后续版本实现。

---

## RequestApi — 请求处理 API

文件：`src/api/request.rs`

| 方法 | OneBot Action | 说明 |
|------|---------------|------|
| `handle_friend_request(flag, approve, remark)` | `set_friend_add_request` | 处理好友请求 |
| `handle_group_request(flag, sub_type, approve, reason)` | `set_group_add_request` | 处理群请求 |

---

## SystemApi — 系统 API

文件：`src/api/system.rs`

| 方法 | OneBot Action | 说明 |
|------|---------------|------|
| `get_online_clients()` | `get_online_clients` | 获取在线客户端列表 |
| `get_robot_uin_range()` | `get_robot_uin_range` | 获取机器人 QQ 号范围 |
| `can_send_image()` | `can_send_image` | 检查是否能发送图片 |
| `can_send_record()` | `can_send_record` | 检查是否能发送语音 |
| `get_cookies(domain)` | `get_cookies` | 获取 Cookies |
| `get_csrf_token()` | `get_csrf_token` | 获取 CSRF Token |
| `get_credentials(domain)` | `get_credentials` | 获取凭证 |
| `set_input_status(user_id, event_type)` | `set_input_status` | 设置输入状态 |
| `ocr_image(image)` | `ocr_image` | OCR 图片识别 |
| `translate_en2zh(words)` | `translate_en2zh` | 英译中 |
| `check_url_safely(url)` | `check_url_safely` | URL 安全检测 |
| `handle_quick_operation(context, operation)` | `.handle_quick_operation` | 快速操作 |
| `nc_get_packet_status()` | `nc_get_packet_status` | 获取封包状态 |

---

## NapCatApi — NapCat 扩展 API

文件：`src/api/napcat.rs`

### Rkey

| 方法 | OneBot Action | 说明 |
|------|---------------|------|
| `get_rkey()` | `get_rkey` | 获取 Rkey |
| `get_rkey_server()` | `get_rkey_server` | 从服务器获取 Rkey |
| `nc_get_rkey()` | `nc_get_rkey` | 获取 NapCat Rkey |

### 好友扩展

| 方法 | OneBot Action | 说明 |
|------|---------------|------|
| `set_friend_remark(user_id, remark)` | `set_friend_remark` | 设置好友备注 |
| `delete_friend(user_id)` | `delete_friend` | 删除好友 |
| `get_unidirectional_friend_list()` | `get_unidirectional_friend_list` | 获取单向好友列表 |

### 群扩展

| 方法 | OneBot Action | 说明 |
|------|---------------|------|
| `set_group_remark(group_id, remark)` | `set_group_remark` | 设置群备注 |
| `get_group_info_ex(group_id)` | `get_group_info_ex` | 获取群扩展信息 |
| `get_group_detail_info(group_id)` | `get_group_detail_info` | 获取群详细信息 |
| `get_group_ignored_notifies(group_id)` | `get_group_ignored_notifies` | 获取被忽略通知 |
| `get_group_shut_list(group_id)` | `get_group_shut_list` | 获取群禁言列表 |

### 合并转发扩展

| 方法 | OneBot Action | 说明 |
|------|---------------|------|
| `send_private_forward_msg(user_id, messages)` | `send_private_forward_msg` | 私聊合并转发 |
| `forward_friend_single_msg(user_id, message_id)` | `forward_friend_single_msg` | 转发单条好友消息 |
| `forward_group_single_msg(group_id, message_id)` | `forward_group_single_msg` | 转发单条群消息 |
| `send_forward_msg(params)` | `send_forward_msg` | 合并转发（通用） |

### 群公告

| 方法 | OneBot Action | 说明 |
|------|---------------|------|
| `send_group_notice(group_id, content, image)` | `_send_group_notice` | 发送群公告 |
| `get_group_notice(group_id)` | `_get_group_notice` | 获取群公告列表 |
| `del_group_notice(group_id, notice_id)` | `_del_group_notice` | 删除群公告 |

### 在线状态

| 方法 | OneBot Action | 说明 |
|------|---------------|------|
| `set_online_status(status, ext, battery)` | `set_online_status` | 设置在线状态 |
| `set_diy_online_status(face_id, wording, face_type)` | `set_diy_online_status` | 自定义在线状态 |

### Ark 分享

| 方法 | OneBot Action | 说明 |
|------|---------------|------|
| `send_ark_share(params)` | `send_ark_share` | 发送 Ark 分享 |
| `send_group_ark_share(params)` | `send_group_ark_share` | 群 Ark 分享 |
| `get_mini_app_ark(params)` | `get_mini_app_ark` | 获取小程序 Ark |

### AI 语音

| 方法 | OneBot Action | 说明 |
|------|---------------|------|
| `get_ai_characters(group_id)` | `get_ai_characters` | 获取 AI 语音角色 |
| `get_ai_record(character, text, group_id)` | `get_ai_record` | 获取 AI 语音 |
| `send_group_ai_record(group_id, character, text)` | `send_group_ai_record` | 发送群 AI 语音 |

### 其他

| 方法 | OneBot Action | 说明 |
|------|---------------|------|
| `set_group_sign(group_id)` | `set_group_sign` | 群签到 |
| `send_group_sign(group_id)` | `send_group_sign` | 群签到（别名） |
| `fetch_custom_face()` | `fetch_custom_face` | 获取自定义表情 |
| `get_emoji_likes(params)` | `get_emoji_likes` | 获取表情回应 |
| `get_clientkey()` | `get_clientkey` | 获取 ClientKey |
| `click_inline_keyboard_button(params)` | `click_inline_keyboard_button` | 点击内联键盘 |

---

## RawActionApi — 原始 API

文件：`src/api/raw.rs`

| 方法 | 说明 |
|------|------|
| `call(action, params)` | 调用任意 action |
| `call_no_params(action)` | 调用无参数 action |
| `known_actions()` | 获取所有已知 action 名称 |

### 使用场景

当 SDK 中未封装某个 API 时，使用原始 API 直接调用：

```rust
use serde_json::json;

// 调用自定义 action
let result = client.api().raw.call("my_custom_action", json!({
    "key": "value",
})).await?;

// 调用无参数 action
let result = client.api().raw.call_no_params("get_login_info").await?;
```
