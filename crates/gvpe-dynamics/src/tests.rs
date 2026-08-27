//! 集成测试（≥ 9 个，覆盖 v0.3 范围）。

use gvpe_core::{BodyHandle, IslandHandle, PhysicsProfile};
use gvpe_math::{Quat, Vec3};

use crate::state::{RigidBodyState, TimeStepper};
use crate::world::DynamicsWorld;

const EPS: f32 = 1e-3;

/// 便利构造：dynamic body 默认 profile。
fn dyn_profile() -> PhysicsProfile {
    let mut p = PhysicsProfile::default_solid();
    p.damping_linear = 0.0;
    p.damping_angular = 0.0;
    p
}

#[test]
fn t01_free_fall_matches_g() {
    // 自由落体：仅重力，零阻尼，验证 y 方向位移 / 速度符合 a = g。
    let mut world = DynamicsWorld::new();
    let p = dyn_profile();
    let h = world
        .spawn_dynamic(
            Vec3::new(0.0, 10.0, 0.0),
            Quat::IDENTITY,
            1.0,
            [1.0, 1.0, 1.0],
            p,
        )
        .unwrap();
    // 100 步 × dt=0.01 = t=1s
    let dt = 0.01;
    for _ in 0..100 {
        world.step(dt).unwrap();
    }
    let s = world.body(h).unwrap();
    // v_y = g * t = 9.81 * 1 = 9.81（向下为负）
    assert!((s.lin_vel.y - (-9.81)).abs() < 0.05, "v_y = {}", s.lin_vel.y);
    // y = y0 + 0.5 * a * t^2 = 10 - 0.5 * 9.81 ≈ 5.095
    assert!((s.position.y - 5.095).abs() < 0.1, "y = {}", s.position.y);
}

#[test]
fn t02_static_body_does_not_move() {
    let mut world = DynamicsWorld::new();
    let h = world.spawn_fixed(Vec3::new(0.0, 0.0, 0.0), Quat::IDENTITY).unwrap();
    for _ in 0..10 {
        world.step(0.1).unwrap();
    }
    let s = world.body(h).unwrap();
    assert_eq!(s.position, Vec3::new(0.0, 0.0, 0.0));
    assert_eq!(s.lin_vel, Vec3::ZERO);
}

#[test]
fn t03_constant_force_yields_acceleration() {
    // F = m * a → a = F / m；验证 a = (1, 0, 0)。
    let mut world = DynamicsWorld::new();
    let h = world
        .spawn_dynamic(
            Vec3::ZERO,
            Quat::IDENTITY,
            2.0,
            [1.0, 1.0, 1.0],
            dyn_profile(),
        )
        .unwrap();
    // 注入恒定力 F = (2, 0, 0)
    {
        let s = world.body_mut(h).unwrap();
        s.add_force(Vec3::new(2.0, 0.0, 0.0));
    }
    let dt = 0.01;
    world.step(dt).unwrap();
    let s = world.body(h).unwrap();
    // a = F / m = 2 / 2 = 1，v = a * dt = 0.01（半隐式 Euler）
    let vx = s.lin_vel.x;
    assert!((vx - 0.01).abs() < EPS, "v_x = {vx}");
    // x = v_new * dt = 0.01 * 0.01 = 0.0001
    let px = s.position.x;
    assert!((0.01f32.mul_add(-dt, px)).abs() < EPS, "x = {px}");
}

#[test]
fn t04_torque_yields_angular_acceleration() {
    // τ = I * α；I = (1, 1, 1)，τ = (0, 1, 0) → α = (0, 1, 0)
    let mut world = DynamicsWorld::new();
    let h = world
        .spawn_dynamic(
            Vec3::ZERO,
            Quat::IDENTITY,
            1.0,
            [1.0, 1.0, 1.0],
            dyn_profile(),
        )
        .unwrap();
    {
        let s = world.body_mut(h).unwrap();
        s.add_torque(Vec3::new(0.0, 1.0, 0.0));
    }
    let dt = 0.01;
    world.step(dt).unwrap();
    let s = world.body(h).unwrap();
    // ω = α * dt = (0, 0.01, 0)
    let wy = s.ang_vel.y;
    assert!((wy - 0.01).abs() < EPS, "ω_y = {wy}");
    // 旋转应当变化（q 与 IDENTITY 不再相等）
    assert_ne!(s.rotation, Quat::IDENTITY);
}

#[test]
fn t05_damping_decays_velocity_to_zero() {
    // 阻尼 1.0 → 1 步后速度 ≈ 0.5 倍（v *= 1/(1 + 1*0.5) = 2/3 ≈ 0.667，
    // 此处 dt=1.0 → k = 1/(1+1)=0.5，v 减半）
    let mut world = DynamicsWorld::new();
    let mut p = dyn_profile();
    p.damping_linear = 1.0;
    p.damping_angular = 1.0;
    let h = world
        .spawn_dynamic(
            Vec3::ZERO,
            Quat::IDENTITY,
            1.0,
            [1.0, 1.0, 1.0],
            p,
        )
        .unwrap();
    {
        let s = world.body_mut(h).unwrap();
        s.lin_vel = Vec3::new(10.0, 0.0, 0.0);
        s.ang_vel = Vec3::new(0.0, 5.0, 0.0);
    }
    world.step(1.0).unwrap();
    let s = world.body(h).unwrap();
    // v = 10 * 0.5 = 5
    let vx = s.lin_vel.x;
    assert!((vx - 5.0).abs() < EPS, "v_x = {vx}");
    // ω = 5 * 0.5 = 2.5
    let wy = s.ang_vel.y;
    assert!((wy - 2.5).abs() < EPS, "ω_y = {wy}");
}

#[test]
fn t06_multi_body_gravity_independent() {
    // 2 个 body 各自自由落体，验证彼此独立。
    let mut world = DynamicsWorld::new();
    let h1 = world
        .spawn_dynamic(
            Vec3::new(0.0, 10.0, 0.0),
            Quat::IDENTITY,
            1.0,
            [1.0, 1.0, 1.0],
            dyn_profile(),
        )
        .unwrap();
    let h2 = world
        .spawn_dynamic(
            Vec3::new(5.0, 10.0, 0.0),
            Quat::IDENTITY,
            2.0,
            [1.0, 1.0, 1.0],
            dyn_profile(),
        )
        .unwrap();
    let dt = 0.01;
    for _ in 0..100 {
        world.step(dt).unwrap();
    }
    // 两 body 应有相同速度（仅受重力，质量不影响加速度）
    let v1 = world.body(h1).unwrap().lin_vel;
    let v2 = world.body(h2).unwrap().lin_vel;
    assert!((v1.y - v2.y).abs() < 1e-4, "v1={v1:?} v2={v2:?}");
}

#[test]
fn t07_user_force_accumulates_with_gravity() {
    // 验证 user force 不会被 predict 阶段清零。
    let mut world = DynamicsWorld::new();
    let h = world
        .spawn_dynamic(
            Vec3::new(0.0, 0.0, 0.0),
            Quat::IDENTITY,
            1.0,
            [1.0, 1.0, 1.0],
            dyn_profile(),
        )
        .unwrap();
    // 第 1 步前注入 (1, 0, 0)
    {
        let s = world.body_mut(h).unwrap();
        s.add_force(Vec3::new(1.0, 0.0, 0.0));
    }
    world.step(0.01).unwrap();
    let v_after_1 = world.body(h).unwrap().lin_vel.x;
    // 第 2 步前再注入 (1, 0, 0)（此时 force 应已被 finalize 清零）
    {
        let s = world.body_mut(h).unwrap();
        s.add_force(Vec3::new(1.0, 0.0, 0.0));
    }
    world.step(0.01).unwrap();
    let v_after_2 = world.body(h).unwrap().lin_vel.x;
    // 两步增量为 (1/1) * 0.01 = 0.01（每步 x 速度 +0.01）
    assert!((v_after_1 - 0.01).abs() < EPS, "v1 = {v_after_1}");
    assert!(
        (v_after_2 - 0.02).abs() < EPS,
        "v2 = {v_after_2} (expected 0.02)"
    );
}

#[test]
fn t08_stepper_dispatch_changes_behavior() {
    // 半隐式 vs 显式 Euler 在长步长下应给出不同结果。
    let mut w1 = DynamicsWorld::new();
    let mut w2 = DynamicsWorld::new();
    let p = dyn_profile();
    let h1 = w1
        .spawn_dynamic(
            Vec3::ZERO,
            Quat::IDENTITY,
            1.0,
            [1.0, 1.0, 1.0],
            p,
        )
        .unwrap();
    let h2 = w2
        .spawn_dynamic(
            Vec3::ZERO,
            Quat::IDENTITY,
            1.0,
            [1.0, 1.0, 1.0],
            p,
        )
        .unwrap();
    // 单步大 dt = 0.5 + 恒定力 (1, 0, 0)
    {
        w1.body_mut(h1).unwrap().add_force(Vec3::new(1.0, 0.0, 0.0));
        w2.body_mut(h2).unwrap().add_force(Vec3::new(1.0, 0.0, 0.0));
    }
    w1.set_stepper(TimeStepper::SemiImplicitEuler);
    w2.set_stepper(TimeStepper::ExplicitEuler);
    w1.step(0.5).unwrap();
    w2.step(0.5).unwrap();
    // 半隐式：v = 1*0.5 = 0.5，x = 0.5 * 0.5 = 0.25
    // 显式：v = 1*0.5 = 0.5，x = 0 * 0.5 = 0
    let s1 = w1.body(h1).unwrap();
    let s2 = w2.body(h2).unwrap();
    let v1x = s1.lin_vel.x;
    let v2x = s2.lin_vel.x;
    let p1x = s1.position.x;
    let p2x = s2.position.x;
    assert!((v1x - 0.5).abs() < EPS, "semi v = {v1x}");
    assert!((v2x - 0.5).abs() < EPS, "expl v = {v2x}");
    // 位置应当不同
    assert!((p1x - 0.25).abs() < EPS, "semi x = {p1x}");
    assert!(p2x.abs() < EPS, "expl x = {p2x}");
}

#[test]
fn t09_invalid_dt_rejected() {
    let mut world = DynamicsWorld::new();
    assert!(world.step(0.0).is_err());
    assert!(world.step(-1.0).is_err());
    assert!(world.step(f32::NAN).is_err());
}

#[test]
fn t10_body_handle_stale_detection() {
    // 释放后再次借取应返回错误。
    let mut world = DynamicsWorld::new();
    let h = world
        .spawn_dynamic(
            Vec3::ZERO,
            Quat::IDENTITY,
            1.0,
            [1.0, 1.0, 1.0],
            dyn_profile(),
        )
        .unwrap();
    world.remove_body(h).unwrap();
    assert!(world.body(h).is_err());
    assert!(world.body_mut(h).is_err());
    assert!(world.profile(h).is_err());
    assert!(world.body_island(h).is_err());
}

#[test]
fn t11_island_handle_contains_body() {
    let mut world = DynamicsWorld::new();
    let h = world
        .spawn_dynamic(
            Vec3::new(1.0, 2.0, 3.0),
            Quat::IDENTITY,
            1.0,
            [1.0, 1.0, 1.0],
            dyn_profile(),
        )
        .unwrap();
    let island = world.body_island(h).unwrap();
    let bodies = world.island_bodies(island).unwrap();
    assert_eq!(bodies.len(), 1);
    assert_eq!(bodies[0], h);
}

#[test]
fn t12_invalid_island_handle() {
    let world = DynamicsWorld::new();
    let bad = IslandHandle::from_raw(999);
    assert!(world.island_bodies(bad).is_err());
}

#[test]
fn t13_state_validate_detects_bad_rotation() {
    let mut s = RigidBodyState::dynamic(
        Vec3::ZERO,
        Quat::IDENTITY,
        Vec3::ZERO,
        Vec3::ZERO,
        1.0,
        [1.0, 1.0, 1.0],
    );
    // 人为破坏旋转
    s.rotation = Quat::new(10.0, 0.0, 0.0, 0.0);
    assert!(s.validate().is_err());
}

#[test]
fn t14_world_reserve_capacity_works() {
    let mut world = DynamicsWorld::with_capacity(100);
    for _ in 0..50 {
        world
            .spawn_dynamic(
                Vec3::ZERO,
                Quat::IDENTITY,
                1.0,
                [1.0, 1.0, 1.0],
                dyn_profile(),
            )
            .unwrap();
    }
    assert_eq!(world.body_count(), 50);
    assert_eq!(world.island_count(), 50);
}

#[test]
fn t15_spawn_and_remove_keeps_consistent_state() {
    // 反复 spawn / remove，验证 body_count 与 island_count 同步。
    let mut world = DynamicsWorld::new();
    let mut hs: Vec<BodyHandle> = Vec::new();
    for i in 0..10 {
        let h = world
            .spawn_dynamic(
                Vec3::new(i as f32, 0.0, 0.0),
                Quat::IDENTITY,
                1.0,
                [1.0, 1.0, 1.0],
                dyn_profile(),
            )
            .unwrap();
        hs.push(h);
    }
    assert_eq!(world.body_count(), 10);
    // 移除 3 个
    for h in hs.iter().take(3) {
        world.remove_body(*h).unwrap();
    }
    assert_eq!(world.body_count(), 7);
    // 7 步后验证剩余 body 仍能正确积分
    world.step(0.01).unwrap();
    let s = world.body(hs[5]).unwrap();
    // 剩余 body 应有重力加速度产生的 y 速度
    let vy = s.lin_vel.y;
    assert!(vy < 0.0, "v_y = {vy}");
}
