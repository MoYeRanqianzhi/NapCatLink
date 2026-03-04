//! # 重连策略模块
//!
//! 实现 WebSocket 连接断开后的自动重连机制，包括：
//! - 指数退避算法（Exponential Backoff）
//! - 最大重试次数限制
//! - 重连状态跟踪和重置

// 引入 Duration 类型，用于表示重连延迟时间
use std::time::Duration;
// 引入重连配置类型
use crate::config::ReconnectConfig;

/// 重连服务 — 管理重连策略和状态
///
/// 根据配置的指数退避策略计算每次重连的延迟时间，
/// 并跟踪当前的重连尝试次数。
///
/// # 指数退避算法
///
/// 延迟时间 = min(initial_ms * multiplier^attempt, max_ms)
///
/// 例如：initial_ms=5000, multiplier=2.0, max_ms=60000
/// - 第 0 次: 5000ms
/// - 第 1 次: 10000ms
/// - 第 2 次: 20000ms
/// - 第 3 次: 40000ms
/// - 第 4 次: 60000ms（达到上限）
pub struct ReconnectService {
    /// 重连配置（包含启用状态、最大次数、退避参数）
    config: ReconnectConfig,
    /// 当前重连尝试次数（从 0 开始计数）
    current_attempt: u32,
}

impl ReconnectService {
    /// 创建新的重连服务实例
    ///
    /// # 参数
    ///
    /// - `config`: 重连策略配置
    ///
    /// # 返回值
    ///
    /// 返回初始化后的 ReconnectService，尝试次数为 0。
    pub fn new(config: ReconnectConfig) -> Self {
        Self {
            // 保存重连配置
            config,
            // 初始化尝试次数为 0
            current_attempt: 0,
        }
    }

    /// 获取下一次重连的延迟时间
    ///
    /// 根据指数退避算法计算延迟，并递增尝试次数计数器。
    ///
    /// # 返回值
    ///
    /// - `Some(Duration)`: 下一次重连应等待的时间
    /// - `None`: 重连已禁用或已达到最大重连次数，不应再尝试
    pub fn next_delay(&mut self) -> Option<Duration> {
        // 如果重连被禁用，直接返回 None
        if !self.config.enabled {
            return None;
        }

        // 如果已达到最大重连次数，返回 None 表示不再重连
        if self.current_attempt >= self.config.max_attempts {
            return None;
        }

        // 计算指数退避延迟时间: initial_ms * multiplier^attempt
        let delay_ms = (self.config.backoff.initial_ms as f64
            * self.config.backoff.multiplier.powi(self.current_attempt as i32))
            // 限制不超过最大退避时间
            .min(self.config.backoff.max_ms as f64) as u64;

        // 递增尝试次数计数器
        self.current_attempt += 1;

        // 返回计算得到的延迟时间
        Some(Duration::from_millis(delay_ms))
    }

    /// 重连成功后重置计数器
    ///
    /// 当连接成功建立后调用此方法，将尝试次数重置为 0，
    /// 以便下次断开连接时从头开始计算退避时间。
    pub fn reset(&mut self) {
        // 将尝试次数重置为 0
        self.current_attempt = 0;
    }

    /// 获取当前已尝试的重连次数
    ///
    /// # 返回值
    ///
    /// 返回当前的重连尝试次数（从 0 开始）。
    pub fn current_attempt(&self) -> u32 {
        // 返回当前尝试次数
        self.current_attempt
    }

    /// 判断是否还有剩余的重连机会
    ///
    /// # 返回值
    ///
    /// - `true`: 重连已启用且尚未达到最大次数
    /// - `false`: 重连已禁用或已达到最大次数
    pub fn has_remaining_attempts(&self) -> bool {
        // 重连启用 且 当前次数未达到最大限制
        self.config.enabled && self.current_attempt < self.config.max_attempts
    }

    /// 获取配置的最大重连次数
    ///
    /// # 返回值
    ///
    /// 返回配置中设定的最大重连尝试次数。
    pub fn max_attempts(&self) -> u32 {
        // 返回配置的最大重连次数
        self.config.max_attempts
    }
}
