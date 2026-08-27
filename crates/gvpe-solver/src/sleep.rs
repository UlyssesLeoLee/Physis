//! Sleeping 状态机。
//!
//! 依据 [`GVPE-DOC-07`] §6.4（07_solver_design.md）：
//! - 线速度 + 角速度连续 N 帧低于阈值的 body 转入 `Sleeping` 状态。
//! - 处于 `Sleeping` 的 body 从 island 活跃计数中排除。
//! - 外部接触 / 外力 wake 重新激活。
//!
//! [`GVPE-DOC-07`]: ../../../docs/02_modules/07_solver_design.md

use crate::body::{RigidBody, SleepState};

/// Sleeping 配置。
///
/// 字段语义（per `07_solver_design.md` §6.4）：
/// - `lin_threshold`：线速度阈值（m/s），低于此值计为"低速"。
/// - `ang_threshold`：角速度阈值（rad/s），低于此值计为"低速"。
/// - `frames_below_to_sleep`：连续低于阈值的帧数；达到后转入 `Sleeping`。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SleepConfig {
    /// 线速度阈值（m/s）。
    pub lin_threshold: f32,
    /// 角速度阈值（rad/s）。
    pub ang_threshold: f32,
    /// 连续低于阈值的帧数；达到后转入 `Sleeping`。
    pub frames_below_to_sleep: u32,
}

impl Default for SleepConfig {
    fn default() -> Self {
        // Box2D-lite 启发：线速度 0.01 m/s、角速度 0.01 rad/s、10 帧。
        Self {
            lin_threshold: 0.01,
            ang_threshold: 0.01,
            frames_below_to_sleep: 10,
        }
    }
}

/// 一次 sleeping tick。
///
/// 调用方（求解器主循环）在每帧末对每个 body 调用一次：
/// - 若 body 为 `Static`：直接 `return false`（永不睡眠，永不 wake）。
/// - 若 body 为 `Active`：检查速度是否低于阈值，更新 `frames_below_threshold`；
///   达到 N 帧后转入 `Sleeping`。
/// - 若 body 为 `Sleeping`：检查外力 / 速度是否被外部 wake（`wake_up` API）；
///   本函数不主动 wake（wake 由外部 force / contact 触发）。
///
/// 返回 `body` 状态是否发生变化（用于 island 活跃计数维护）。
pub fn tick_sleep(body: &mut RigidBody, cfg: &SleepConfig) -> bool {
    if body.is_static() {
        return false;
    }
    let lin_low = body.lin_vel.length_squared() < cfg.lin_threshold * cfg.lin_threshold;
    let ang_low = body.ang_vel.length_squared() < cfg.ang_threshold * cfg.ang_threshold;
    match body.sleep {
        SleepState::Static => false,
        SleepState::Sleeping => false,
        SleepState::Active => {
            if lin_low && ang_low {
                body.frames_below_threshold = body.frames_below_threshold.saturating_add(1);
                if body.frames_below_threshold >= cfg.frames_below_to_sleep {
                    body.sleep = SleepState::Sleeping;
                    // 进入睡眠时清零速度，避免浮点漂移 wake。
                    body.lin_vel = gvpe_math::Vec3::ZERO;
                    body.ang_vel = gvpe_math::Vec3::ZERO;
                    body.accumulated_force = gvpe_math::Vec3::ZERO;
                    body.accumulated_torque = gvpe_math::Vec3::ZERO;
                    return true;
                }
                false
            } else {
                body.frames_below_threshold = 0;
                false
            }
        },
    }
}

/// 强制 wake body（外部 force / contact 触发）。
///
/// 转入 `Active`，重置 `frames_below_threshold = 0`。
/// 对 `Static` 体无效（永远静态）。
///
/// 返回 `body` 状态是否发生变化。
pub fn wake_up(body: &mut RigidBody) -> bool {
    if body.is_static() {
        return false;
    }
    if body.sleep == SleepState::Sleeping {
        body.sleep = SleepState::Active;
        body.frames_below_threshold = 0;
        return true;
    }
    false
}

/// 强制 sleep body（外部命令触发，如 debug）。
///
/// 转入 `Sleeping`，清零速度。
/// 对 `Static` 体无效。
///
/// 返回 `body` 状态是否发生变化。
pub fn force_sleep(body: &mut RigidBody) -> bool {
    if body.is_static() {
        return false;
    }
    if body.sleep == SleepState::Active {
        body.sleep = SleepState::Sleeping;
        body.lin_vel = gvpe_math::Vec3::ZERO;
        body.ang_vel = gvpe_math::Vec3::ZERO;
        body.accumulated_force = gvpe_math::Vec3::ZERO;
        body.accumulated_torque = gvpe_math::Vec3::ZERO;
        return true;
    }
    false
}
