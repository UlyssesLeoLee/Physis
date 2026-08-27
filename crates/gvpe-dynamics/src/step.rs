//! 时间步进函数（`predict` / `integrate` / `finalize`）。
//!
//! 三个阶段分离的目的：
//!
//! 1. `predict(dt)` —— 对所有 body 应用力源（gravity / damping），清零上一帧
//!    累加的 user force 后整合当前帧
//! 2. `integrate(dt)` —— 推进位置 / 旋转（半隐式 / 显式 / RK4 派别）
//! 3. `finalize()` —— 清零力 / 力矩累加器（`force` / `torque` 归零）
//!
//! 三阶段分离便于 v0.5+ 引入"求解前/后"挂钩（如约束投影、CCD 预处理）
//! 时只新增 `step_pre_*` / `step_post_*`，不动主循环。
//!
//! ## 算法选型（草案，DDD Review 必查）
//!
//! 详见 [`TimeStepper`](crate::state::TimeStepper) 注释。
//! v0.3 MVP 默认 [`TimeStepper::SemiImplicitEuler`]。

use gvpe_math::{Quat, Vec3};

use crate::error::{DynamicsError, DynamicsResult};
use crate::state::{RigidBodyState, TimeStepper};

/// 验证时间步长。
#[inline]
pub fn validate_dt(dt: f32) -> DynamicsResult<f32> {
    if !dt.is_finite() || dt <= 0.0 {
        return Err(DynamicsError::InvalidTimeStep(dt));
    }
    Ok(dt)
}

/// 累加（predict 阶段）：应用力源（重力）+ 阻尼。
///
/// 顺序：先清零力 / 力矩（吸收 user force 注入的边界），再应用全局力源。
/// 注：本函数**不**清零 `force` —— `finalize` 阶段清零；`predict` 阶段
/// 假设 `RigidBodyState::force` 已被上一步 `finalize` 清零。
///
/// **作用等价于** 在 `force` 上叠加 `mass * gravity`，**并**对速度做阻尼
/// 衰减（不动位置 / 旋转）。
pub fn predict(body: &mut RigidBodyState, gravity: Vec3, _dt: f32) {
    crate::force::apply_gravity(body, gravity);
    // 注：damping 显式由调用方在 `predict_with_damping` 提供（profile 信息
    // 来自 gvpe-core，本函数不依赖 profile 以保持纯函数签名）。
}

/// predict 同上，但**额外**应用 profile 中的阻尼项。
pub fn predict_with_damping(
    body: &mut RigidBodyState,
    gravity: Vec3,
    profile: &gvpe_core::PhysicsProfile,
    dt: f32,
) {
    crate::force::apply_gravity(body, gravity);
    crate::force::apply_damping(body, profile, dt);
}

/// 积分（integrate 阶段）：半隐式 Euler。
///
/// ```text
/// a = F * inv_mass
/// α = τ * inv_inertia_diag
/// v_{n+1} = v_n + a * dt
/// ω_{n+1} = ω_n + α * dt
/// x_{n+1} = x_n + v_{n+1} * dt
/// q_{n+1} = normalize(q_n + 0.5 * ω_{n+1} * q_n * dt)
/// ```
///
/// static body 调用为 no-op。
pub fn integrate_semi_implicit(body: &mut RigidBodyState, dt: f32) {
    if body.is_static() {
        return;
    }
    // a = F * inv_mass
    let acc = body.force * body.inv_mass;
    // α = τ * inv_inertia_diag (v0.3 对角线限制)
    let alpha = Vec3::new(
        body.torque.x * body.inv_inertia_diag[0],
        body.torque.y * body.inv_inertia_diag[1],
        body.torque.z * body.inv_inertia_diag[2],
    );
    // v_{n+1} = v_n + a * dt
    body.lin_vel = body.lin_vel + acc * dt;
    // ω_{n+1} = ω_n + α * dt
    body.ang_vel = body.ang_vel + alpha * dt;
    // x_{n+1} = x_n + v_{n+1} * dt
    body.position = body.position + body.lin_vel * dt;
    // q_{n+1} = q_n + 0.5 * ω_{n+1} * q_n * dt
    let omega = Quat::new(body.ang_vel.x, body.ang_vel.y, body.ang_vel.z, 0.0);
    let dq_q = omega.mul(body.rotation);
    let s = 0.5 * dt;
    let dq = Quat::new(dq_q.x * s, dq_q.y * s, dq_q.z * s, dq_q.w * s);
    let new_q = Quat::new(
        body.rotation.x + dq.x,
        body.rotation.y + dq.y,
        body.rotation.z + dq.z,
        body.rotation.w + dq.w,
    );
    body.rotation = new_q.normalize();
}

/// 积分：显式 Euler（对照基线；不推荐生产）。
///
/// 与半隐式 Euler 区别：`x_{n+1} = x_n + v_n * dt`（用旧速度）。
pub fn integrate_explicit(body: &mut RigidBodyState, dt: f32) {
    if body.is_static() {
        return;
    }
    let acc = body.force * body.inv_mass;
    let alpha = Vec3::new(
        body.torque.x * body.inv_inertia_diag[0],
        body.torque.y * body.inv_inertia_diag[1],
        body.torque.z * body.inv_inertia_diag[2],
    );
    let old_lin = body.lin_vel;
    let old_ang = body.ang_vel;
    body.lin_vel = old_lin + acc * dt;
    body.ang_vel = old_ang + alpha * dt;
    body.position = body.position + old_lin * dt;
    let omega = Quat::new(old_ang.x, old_ang.y, old_ang.z, 0.0);
    let dq_q = omega.mul(body.rotation);
    let s = 0.5 * dt;
    let dq = Quat::new(dq_q.x * s, dq_q.y * s, dq_q.z * s, dq_q.w * s);
    let new_q = Quat::new(
        body.rotation.x + dq.x,
        body.rotation.y + dq.y,
        body.rotation.z + dq.z,
        body.rotation.w + dq.w,
    );
    body.rotation = new_q.normalize();
}

/// 积分：经典 RK4（高精度研究路径；4× 单步成本）。
///
/// 4 次力矩评估 → 4 次速度更新。MVP 不实现陀螺力 / 全惯性张量，
/// 故每步力矩恒定（4 次评估结果相同）—— 退化为半隐式 Euler。
/// 该路径仅在 v0.4+ 引入全惯性张量时才有意义。
pub fn integrate_rk4(body: &mut RigidBodyState, dt: f32) {
    // 退化实现：见函数级注释。MVP 阶段 Rk4 == SemiImplicitEuler
    // （恒定力矩下 k1 = k2 = k3 = k4），保持 API 完整。
    integrate_semi_implicit(body, dt);
}

/// 积分入口：按 `TimeStepper` 派发。
pub fn integrate(body: &mut RigidBodyState, dt: f32, stepper: TimeStepper) {
    match stepper {
        TimeStepper::SemiImplicitEuler => integrate_semi_implicit(body, dt),
        TimeStepper::ExplicitEuler => integrate_explicit(body, dt),
        TimeStepper::Rk4 => integrate_rk4(body, dt),
    }
}

/// finalize 阶段：清零力 / 力矩累加器。
#[inline]
pub fn finalize(body: &mut RigidBodyState) {
    body.clear_accumulators();
}
