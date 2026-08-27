//! 力累加器（`ForceAccumulator`）。
//!
//! v0.3 MVP 阶段不存"每个 body 一份累加器"，而是直接复用
//! [`RigidBodyState::force`](crate::state::RigidBodyState::force) /
//! [`torque`](crate::state::RigidBodyState::torque) 字段。
//!
//! 本文件提供**纯函数** `apply_gravity` / `apply_damping`，
//! 在 [`DynamicsWorld::predict`](crate::world::DynamicsWorld::predict)
//! 阶段按需调用。保持"累加逻辑与状态分离"，便于 v0.4+ 切到
//! 独立累加器（per-body bump pool）时只换签名、不动调用方。
//!
//! ## 力源清单（v0.3 MVP）
//!
//! - `gravity` —— 重力（恒定向量，按 `mass * g` 缩放）
//! - `damping_linear` / `damping_angular` —— 线性 / 角速度阻尼
//!   （指数衰减近似：`v *= 1 / (1 + k * dt)`）
//! - `user_force` —— 调用方通过 [`RigidBodyState::add_force`] 注入
//!
//! ## 不做（v0.3 范围外）
//!
//! - 陀螺力（`ω × Iω`）—— 高速旋转刚体重要，v0.4+ 评估
//! - 空气阻力（`F_drag = -k * v * |v|`）—— v0.4+ 评估
//! - 弹簧 / 阻尼器 / 约束反力——属 `gvpe-constraint` crate

use gvpe_core::PhysicsProfile;
use gvpe_math::Vec3;

use crate::state::RigidBodyState;

/// 累加重力到 body 力累加器。
///
/// `gravity` 为世界系下的重力向量（典型 `(0, -9.81, 0)`），
/// 本函数等价于 `F += mass * gravity = gravity / inv_mass`。
///
/// static body 调用为 no-op。
#[inline]
pub fn apply_gravity(body: &mut RigidBodyState, gravity: Vec3) {
    if body.is_static() {
        return;
    }
    // `F = m * g = g / inv_mass`，对 inv_mass = 0 的 static 已短路。
    body.force = body.force + gravity * (1.0 / body.inv_mass);
}

/// 累加线性 / 角速度阻尼到 body 速度。
///
/// 用指数衰减近似（`v *= 1 / (1 + k * dt)`）：
///
/// - 半隐式 Euler 下二阶精度
/// - `k = 0` → 无阻尼（恒等变换）
/// - `k > 0` → 收敛到 0
///
/// **不**清零 `force` / `torque` 累加器，**不**移动 `position` / `rotation`。
#[inline]
pub fn apply_damping(body: &mut RigidBodyState, profile: &PhysicsProfile, dt: f32) {
    if body.is_static() {
        return;
    }
    let lin = profile.damping_linear;
    let ang = profile.damping_angular;
    if lin > 0.0 {
        let k = 1.0 / lin.mul_add(dt, 1.0);
        body.lin_vel = body.lin_vel * k;
    }
    if ang > 0.0 {
        let k = 1.0 / ang.mul_add(dt, 1.0);
        body.ang_vel = body.ang_vel * k;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gvpe_core::PhysicsProfile;
    use gvpe_math::Quat;

    #[test]
    fn apply_gravity_dynamic_accumulates_force() {
        let mut s = RigidBodyState::dynamic(
            Vec3::ZERO,
            Quat::IDENTITY,
            Vec3::ZERO,
            Vec3::ZERO,
            2.0,
            [1.0, 1.0, 1.0],
        );
        apply_gravity(&mut s, Vec3::new(0.0, -9.81, 0.0));
        // F = m * g = 2.0 * (-9.81) = -19.62
        assert!((s.force.y - (-19.62)).abs() < 1e-5);
    }

    #[test]
    fn apply_gravity_static_is_noop() {
        let mut s = RigidBodyState::fixed(Vec3::ZERO, Quat::IDENTITY);
        apply_gravity(&mut s, Vec3::new(0.0, -9.81, 0.0));
        assert_eq!(s.force, Vec3::ZERO);
    }

    #[test]
    fn apply_damping_decays_velocity() {
        let mut s = RigidBodyState::dynamic(
            Vec3::ZERO,
            Quat::IDENTITY,
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::ZERO,
            1.0,
            [1.0, 1.0, 1.0],
        );
        let p = PhysicsProfile::default_solid();
        // damping_linear default = 0.01
        apply_damping(&mut s, &p, 1.0);
        // v *= 1 / (1 + 0.01 * 1) = 1 / 1.01 ≈ 0.9901
        assert!(s.lin_vel.x < 10.0 && s.lin_vel.x > 9.9);
    }

    #[test]
    fn apply_damping_zero_damping_is_identity() {
        let mut s = RigidBodyState::dynamic(
            Vec3::ZERO,
            Quat::IDENTITY,
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::ZERO,
            1.0,
            [1.0, 1.0, 1.0],
        );
        let mut p = PhysicsProfile::default_solid();
        p.damping_linear = 0.0;
        p.damping_angular = 0.0;
        apply_damping(&mut s, &p, 1.0);
        assert_eq!(s.lin_vel, Vec3::new(1.0, 2.0, 3.0));
    }
}
