//! # API 调用模块
//!
//! 封装 OneBot 11 标准 API 和 NapCat 扩展 API 的调用接口，
//! 分为以下子模块：
//! - `client`：API 客户端，封装请求发送和响应接收
//! - `dispatcher`：API 请求分发器，管理请求-响应的异步匹配
//! - `message`：消息相关 API（发送消息、撤回消息等）
//! - `group`：群组相关 API（群管理、群成员操作等）
//! - `account`：账号相关 API（获取登录信息、好友列表等）
//! - `media`：媒体相关 API（图片、语音、视频处理）
//! - `file`：文件相关 API（文件上传、下载）
//! - `stream`：流式传输 API（大文件分片上传/下载）
//! - `request`：请求处理 API（处理好友申请、群邀请等）
//! - `system`：系统相关 API（获取版本信息、重启等）
//! - `napcat`：NapCat 扩展 API（NapCat 特有的增强功能）
//! - `raw`：原始 API 调用接口（直接发送自定义 JSON 请求）

// API 客户端子模块：封装底层请求发送逻辑
pub mod client;

// API 请求分发器子模块：管理请求 ID 到响应回调的映射
pub mod dispatcher;

// 消息 API 子模块：发送私聊/群聊消息、撤回消息、获取消息等
pub mod message;

// 群组 API 子模块：群信息查询、群成员管理、群设置等
pub mod group;

// 账号 API 子模块：登录信息、好友列表、陌生人信息等
pub mod account;

// 媒体 API 子模块：图片、语音、视频等媒体资源处理
pub mod media;

// 文件 API 子模块：文件上传和下载操作
pub mod file;

// 流式传输 API 子模块：大文件分片传输
pub mod stream;

// 请求处理 API 子模块：处理好友添加请求、群邀请请求等
pub mod request;

// 系统 API 子模块：获取运行状态、版本信息、重启服务等
pub mod system;

// NapCat 扩展 API 子模块：NapCat 框架特有的增强功能接口
pub mod napcat;

// 原始 API 子模块：支持直接发送自定义 JSON 格式的 API 请求
pub mod raw;
