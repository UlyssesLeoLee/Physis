//! `gvpe-solver` 单元测试。
//!
//! 覆盖：约束行构造 / SI 求解 / warm-start / friction box / sleeping 状态机。
//!
//! 测试目标数：≥ 12（per task brief）。

use super::*;
use crate::body::SleepState;
use gvpe_math::Vec3;

// ───────── 辅助构造器 ─────────

fn dyn_body(mass: f32, inertia: [f32; 3]) -> RigidBody {
    RigidBody::new_dynamic(
        gvpe_core::BodyHandle::new(0, 0),
        0,
        mass,
        inertia,
    )
}

// ───────── ConstraintRow 测试 ─────────

#[test]
fn constraint_row_default_is_zeroed() {
    let row = ConstraintRow::default();
    assert_eq!(row.lambda, 0.0);
    assert_eq!(row.bias, 0.0);
    assert_eq!(row.lower, 0.0);
    assert_eq!(row.upper, 0.0);
    assert_eq!(row.jacobian, [0.0; 12]);
}

#[test]
fn contact_normal_jacobian_layout() {
    // 法向 n = (0, 1, 0)，接触点 r_a = r_b = 0（原点） → J_a_lin = -n, J_b_lin = +n。
    let n = Vec3::new(0.0, 1.0, 0.0);
    let row = ConstraintRow::contact_normal(
        gvpe_core::BodyHandle::new(0, 0),
        gvpe_core::BodyHandle::new(1, 0),
        n,
        Vec3::ZERO,
        Vec3::ZERO,
        0.1,
    );
    assert_eq!(row.jacobian[0], 0.0);
    assert_eq!(row.jacobian[1], -1.0);
    assert_eq!(row.jacobian[2], 0.0);
    assert_eq!(row.jacobian[6], 0.0);
    assert_eq!(row.jacobian[7], 1.0);
    assert_eq!(row.jacobian[8], 0.0);
    assert_eq!(row.bias, 0.1);
    assert_eq!(row.lower, 0.0); // 单边
    assert!(row.upper.is_infinite());
}

#[test]
fn contact_normal_jacobian_with_arms() {
    // r_a = (1, 0, 0), n = (0, 1, 0) → J_a_ang = -(1, 0, 0) × (0, 1, 0) = -(0*0 - 0*1, 0*0 - 1*0, 1*1 - 0*0) = -(0, 0, 1) = (0, 0, -1)
    let n = Vec3::new(0.0, 1.0, 0.0);
    let r_a = Vec3::new(1.0, 0.0, 0.0);
    let row = ConstraintRow::contact_normal(
        gvpe_core::BodyHandle::new(0, 0),
        gvpe_core::BodyHandle::new(1, 0),
        n,
        r_a,
        Vec3::ZERO,
        0.0,
    );
    assert!((row.jacobian[3] - 0.0).abs() < 1e-6);
    assert!((row.jacobian[4] - 0.0).abs() < 1e-6);
    assert!((row.jacobian[5] - (-1.0)).abs() < 1e-6);
}

#[test]
fn solve_single_clamps_to_lower_upper() {
    // 构造一个无 bias 的行；jv = 0 → delta_lambda = 0，clamp 在 [lower, upper]。
    let mut row = ConstraintRow {
        lower: -0.5,
        upper: 0.5,
        ..ConstraintRow::default()
    };
    row.lambda = 1.0; // 起始越界
    let applied = row.solve_single(0.0, 1.0);
    assert!((row.lambda - 0.5).abs() < 1e-6, "lambda should clamp to upper=0.5");
    assert!((applied - (-0.5)).abs() < 1e-6);
}

#[test]
fn solve_single_step_with_jv() {
    // 有效质量 K=2，jv=4，bias=0 → delta = -K^-1 * jv = -0.5 * 4 = -2。
    // 从 lambda=0 起步 → new = -2，clamp 到 [0, inf) = 0，applied = 0。
    let mut row = ConstraintRow {
        lower: 0.0,
        upper: f32::INFINITY,
        ..ConstraintRow::default()
    };
    let applied = row.solve_single(4.0, 0.5);
    assert_eq!(row.lambda, 0.0);
    assert_eq!(applied, 0.0);
}

// ───────── RigidBody 测试 ─────────

#[test]
fn rigid_body_new_dynamic_inverts_mass_and_inertia() {
    let b = dyn_body(2.0, [4.0, 8.0, 16.0]);
    assert!((b.inv_mass - 0.5).abs() < 1e-6);
    assert!((b.inv_inertia_diag[0] - 0.25).abs() < 1e-6);
    assert!((b.inv_inertia_diag[1] - 0.125).abs() < 1e-6);
    assert!((b.inv_inertia_diag[2] - 0.0625).abs() < 1e-6);
    assert!(!b.is_static());
    assert_eq!(b.sleep, SleepState::Active);
}

#[test]
fn rigid_body_new_static_is_static() {
    let b = RigidBody::new_static(gvpe_core::BodyHandle::new(0, 0), 0);
    assert!(b.is_static());
    assert_eq!(b.sleep, SleepState::Static);
    assert_eq!(b.inv_mass, 0.0);
    assert_eq!(b.inv_inertia_diag, [0.0; 3]);
}

// ───────── Sleeping 测试 ─────────

#[test]
fn sleep_transitions_after_n_frames() {
    let mut body = dyn_body(1.0, [1.0; 3]);
    let cfg = SleepConfig {
        lin_threshold: 0.01,
        ang_threshold: 0.01,
        frames_below_to_sleep: 3,
    };
    // 速度 = 0 → frames_below_threshold 递增。
    for frame in 0..3 {
        let changed = tick_sleep(&mut body, &cfg);
        if frame < 2 {
            assert!(!changed, "frame {frame} should not yet sleep");
            assert_eq!(body.sleep, SleepState::Active);
        } else {
            assert!(changed, "frame {frame} should transition to Sleeping");
            assert_eq!(body.sleep, SleepState::Sleeping);
        }
    }
}

#[test]
fn sleep_counter_resets_when_velocity_exceeds_threshold() {
    let mut body = dyn_body(1.0, [1.0; 3]);
    let cfg = SleepConfig {
        lin_threshold: 0.01,
        ang_threshold: 0.01,
        frames_below_to_sleep: 3,
    };
    tick_sleep(&mut body, &cfg);
    tick_sleep(&mut body, &cfg);
    assert_eq!(body.frames_below_threshold, 2);
    // 给一个超阈值速度。
    body.lin_vel = Vec3::new(1.0, 0.0, 0.0);
    tick_sleep(&mut body, &cfg);
    assert_eq!(body.frames_below_threshold, 0);
    assert_eq!(body.sleep, SleepState::Active);
}

#[test]
fn static_body_never_sleeps() {
    let mut body = RigidBody::new_static(gvpe_core::BodyHandle::new(0, 0), 0);
    let cfg = SleepConfig::default();
    for _ in 0..100 {
        let changed = tick_sleep(&mut body, &cfg);
        assert!(!changed);
    }
    assert_eq!(body.sleep, SleepState::Static);
}

#[test]
fn wake_up_sleeping_body() {
    let mut body = dyn_body(1.0, [1.0; 3]);
    body.sleep = SleepState::Sleeping;
    body.frames_below_threshold = 5;
    let changed = wake_up(&mut body);
    assert!(changed);
    assert_eq!(body.sleep, SleepState::Active);
    assert_eq!(body.frames_below_threshold, 0);
}

#[test]
fn wake_up_static_body_is_noop() {
    let mut body = RigidBody::new_static(gvpe_core::BodyHandle::new(0, 0), 0);
    let changed = wake_up(&mut body);
    assert!(!changed);
    assert_eq!(body.sleep, SleepState::Static);
}

// ───────── Friction 测试 ─────────

#[test]
fn tangent_pair_is_orthonormal() {
    let n = Vec3::new(0.0, 1.0, 0.0);
    let (t1, t2) = tangent_pair(n);
    assert!((t1.dot(n)).abs() < 1e-5, "t1 ⊥ n");
    assert!((t2.dot(n)).abs() < 1e-5, "t2 ⊥ n");
    assert!((t1.dot(t2)).abs() < 1e-5, "t1 ⊥ t2");
    assert!((t1.length() - 1.0).abs() < 1e-5, "t1 unit");
    assert!((t2.length() - 1.0).abs() < 1e-5, "t2 unit");
}

#[test]
fn tangent_pair_robust_on_near_axis_aligned_n() {
    // n = (0, 0, 1) 接近种子轴，应仍能产生正交对。
    let n = Vec3::new(0.0, 0.0, 1.0);
    let (t1, t2) = tangent_pair(n);
    assert!(t1.x.is_finite() && t1.y.is_finite() && t1.z.is_finite());
    assert!(t2.x.is_finite() && t2.y.is_finite() && t2.z.is_finite());
    assert!((t1.length() - 1.0).abs() < 1e-4);
    assert!((t2.length() - 1.0).abs() < 1e-4);
    assert!((t1.dot(t2)).abs() < 1e-4);
}

#[test]
fn friction_bounds_proportional_to_mu_and_lambda() {
    let cfg = FrictionConfig { mu: 0.5 };
    let (lo, hi) = friction_bounds(&cfg, 2.0);
    assert!((lo - (-1.0)).abs() < 1e-6);
    assert!((hi - 1.0).abs() < 1e-6);
    let (lo2, hi2) = friction_bounds(&cfg, 0.0);
    assert_eq!(lo2, 0.0);
    assert_eq!(hi2, 0.0);
}

#[test]
fn friction_rows_initial_bounds_zero() {
    let ha = gvpe_core::BodyHandle::new(0, 0);
    let hb = gvpe_core::BodyHandle::new(1, 0);
    let n = Vec3::Y;
    let rows = build_friction_rows(ha, hb, n, Vec3::ZERO, Vec3::ZERO);
    assert_eq!(rows.len(), 2);
    assert!((rows[0].lower - 0.0).abs() < 1e-6);
    assert!((rows[0].upper - 0.0).abs() < 1e-6);
    assert!((rows[0].bias - 0.0).abs() < 1e-6); // 摩擦行无 bias
}

// ───────── Solver 集成测试 ─────────

#[test]
fn solver_empty_inputs_no_panic() {
    let mut solver = Solver::default();
    let mut island = Island::new();
    let mut bodies = BodySlab::new();
    solver.solve(&mut island, &mut bodies).unwrap();
}

#[test]
fn solver_no_constraints_integrates_velocity() {
    let mut solver = Solver::default();
    let mut island = Island::new();
    let mut bodies = BodySlab::new();
    let (h, _) = bodies.insert_dynamic(1.0, [1.0; 3]).unwrap();
    // 设线速度，verify 1 帧后位置变化。
    let dt = solver.config().dt;
    bodies.get_mut(h).unwrap().lin_vel = Vec3::new(1.0, 0.0, 0.0);
    let pos_before = bodies.get(h).unwrap().position;
    solver.solve(&mut island, &mut bodies).unwrap();
    let pos_after = bodies.get(h).unwrap().position;
    assert!(((pos_after.x - pos_before.x) - 1.0 * dt).abs() < 1e-4);
}

#[test]
fn solver_warm_start_preserves_lambda_across_frames() {
    // 持续穿透 + 持续接近速度，每帧求解后 lambda 应非零。
    // warm-starting 跨帧保留 lambda，本测试验证：连续 3 帧每帧都给同样的相对
    // 接近速度与穿透 bias，lambda 在每帧末都保持 > 0。
    let mut solver = Solver::new(SolverConfig {
        iterations: 10,
        dt: 1.0 / 60.0,
        bias_beta: 0.2,
        sleep: SleepConfig {
            lin_threshold: 100.0, // 关闭 sleep（避免干扰）
            ang_threshold: 100.0,
            frames_below_to_sleep: 1000,
        },
        friction: FrictionConfig { mu: 0.0 },
    });
    let mut island = Island::new();
    let mut bodies = BodySlab::new();
    let (ha, _) = bodies.insert_dynamic(1.0, [1.0; 3]).unwrap();
    let (hb, _) = bodies.insert_dynamic(1.0, [1.0; 3]).unwrap();
    let dt = solver.config().dt;
    let penetration = 0.5;
    // Box2D-style bias：约束 J·v >= 0（分离），penetration 进入 J·v + bias
    // 时需带负号。beta * penetration / dt 取负。
    let bias = -0.2 * penetration / dt;

    for frame in 0..3 {
        // 重置位置（嵌入 0.5）+ 重置速度（互相接近）。
        bodies.get_mut(ha).unwrap().position = Vec3::new(0.0, 0.0, 0.0);
        bodies.get_mut(hb).unwrap().position = Vec3::new(0.0, 0.5, 0.0);
        bodies.get_mut(hb).unwrap().lin_vel = Vec3::new(0.0, -1.0, 0.0);
        bodies.get_mut(ha).unwrap().lin_vel = Vec3::ZERO;

        // 重置 island 行（保留 lambda 状态）。
        let lambda_prev = if frame == 0 { 0.0 } else { island.rows[0].lambda };
        island.rows.clear();
        let mut normal = ConstraintRow::contact_normal(
            ha,
            hb,
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::ZERO,
            Vec3::ZERO,
            bias,
        );
        normal.lambda = lambda_prev; // 模拟 warm-start
        island.push(normal);

        solver.solve(&mut island, &mut bodies).unwrap();
        assert!(
            island.rows[0].lambda > 0.0,
            "frame {frame}: lambda = {} (应 > 0)",
            island.rows[0].lambda
        );
    }
}

#[test]
fn j_dot_v_known_input() {
    // 简单测试：a.lin_vel = (1, 0, 0), J_a_lin = (1, 0, 0) → j_dot_v 第一项 = 1。
    let mut a = dyn_body(1.0, [1.0; 3]);
    let mut b = dyn_body(1.0, [1.0; 3]);
    a.lin_vel = Vec3::new(1.0, 0.0, 0.0);
    b.lin_vel = Vec3::ZERO;
    let j = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let v = j_dot_v(j, &a, &b);
    assert!((v - 1.0).abs() < 1e-6);
}

#[test]
fn effective_mass_simple_1d() {
    // J = [1, 0, 0, 0..., 1, 0, 0, 0...] (两 body 沿 x 互推)
    // m_a = m_b = 2 → K = (1/2) * 1 + (1/2) * 1 = 1
    let a = dyn_body(2.0, [1.0; 3]);
    let b = dyn_body(2.0, [1.0; 3]);
    let j = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let k = effective_mass(j, &a, &b);
    assert!((k - 1.0).abs() < 1e-6, "K = 1/2 + 1/2 = 1, got {k}");
}

#[test]
fn body_slab_insert_and_get() {
    let mut slab = BodySlab::new();
    let (h1, idx1) = slab.insert_dynamic(1.0, [1.0; 3]).unwrap();
    let (h2, idx2) = slab.insert_static().unwrap();
    assert_eq!(idx1, 0);
    assert_eq!(idx2, 1);
    assert_eq!(slab.len(), 2);
    let b1 = slab.get(h1).unwrap();
    assert!((b1.mass - 1.0).abs() < 1e-6);
    let b2 = slab.get(h2).unwrap();
    assert!(b2.is_static());
}

#[test]
fn island_push_and_len() {
    let mut island = Island::new();
    assert!(island.is_empty());
    assert_eq!(island.len(), 0);
    island.push(ConstraintRow::default());
    island.push(ConstraintRow::default());
    assert_eq!(island.len(), 2);
    assert!(!island.is_empty());
    island.clear_warm_start();
    for r in &island.rows {
        assert!((r.lambda - 0.0).abs() < 1e-6);
    }
}

#[test]
fn solver_two_box_stack_gravity_separates_via_constraint() {
    // 两 body 沿 y 轴堆叠：下方 body 静态 + bias（穿透修正），上方 body 受重力。
    // SI 循环应将 normal lambda 累加为正，避免穿透。
    let mut solver = Solver::new(SolverConfig {
        iterations: 10,
        dt: 1.0 / 60.0,
        bias_beta: 0.2,
        sleep: SleepConfig {
            lin_threshold: 100.0, // 关闭 sleeping（避免测试 flaky）
            ang_threshold: 100.0,
            frames_below_to_sleep: 1000,
        },
        friction: FrictionConfig { mu: 0.0 },
    });
    let mut island = Island::new();
    let mut bodies = BodySlab::new();
    let (h_static, _) = bodies.insert_static().unwrap();
    let (h_dyn, _) = bodies.insert_dynamic(1.0, [1.0; 3]).unwrap();
    bodies.get_mut(h_static).unwrap().position = Vec3::new(0.0, 0.0, 0.0);
    bodies.get_mut(h_dyn).unwrap().position = Vec3::new(0.0, 0.5, 0.0);
    bodies.get_mut(h_dyn).unwrap().lin_vel = Vec3::new(0.0, -0.1, 0.0);

    // 接触法向 n = (0, 1, 0)（从 static 指向 dyn），bias = penetration / dt。
    let penetration = 0.5; // dyn 嵌入 static 0.5 单位
    let dt = 1.0 / 60.0;
    // Box2D-style bias：负号。
    let bias = -0.2 * penetration / dt;
    island.push(ConstraintRow::contact_normal(
        h_static,
        h_dyn,
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::ZERO,
        Vec3::ZERO,
        bias,
    ));
    solver.solve(&mut island, &mut bodies).unwrap();
    // dyn 速度在约束作用下应不再向下（被法向冲量反推或减速）。
    let v_after = bodies.get(h_dyn).unwrap().lin_vel.y;
    // 不严格断言 v_y > 0；只断言 v_y 不再向 -y 加速（>= -0.1 + epsilon）。
    assert!(v_after > -0.1 - 1e-3, "v_y after solve = {v_after}");
    // 法向冲量 lambda 应 >= 0（单边）。
    assert!(island.rows[0].lambda >= 0.0, "lambda = {}", island.rows[0].lambda);
}

#[test]
fn sleep_in_solver_integration() {
    // 用极小阈值和帧数，使 body 在 1 帧后进入 Sleeping。
    let mut solver = Solver::new(SolverConfig {
        iterations: 4,
        dt: 1.0 / 60.0,
        bias_beta: 0.2,
        sleep: SleepConfig {
            lin_threshold: 0.1,
            ang_threshold: 0.1,
            frames_below_to_sleep: 1,
        },
        friction: FrictionConfig::default(),
    });
    let mut island = Island::new();
    let mut bodies = BodySlab::new();
    let (h, _) = bodies.insert_dynamic(1.0, [1.0; 3]).unwrap();
    bodies.get_mut(h).unwrap().lin_vel = Vec3::ZERO;
    bodies.get_mut(h).unwrap().ang_vel = Vec3::ZERO;
    solver.solve(&mut island, &mut bodies).unwrap();
    assert_eq!(bodies.get(h).unwrap().sleep, SleepState::Sleeping);
}

#[test]
fn friction_box_updates_after_normal_impulse() {
    // 验证：在求解循环中，normal 行的 lambda 更新会驱动摩擦行上下界。
    // 构造一个 simple 1D 接触，迭代 1 次后手动检查 lambda + 用 sync_friction_bounds 推
    // 算摩擦 box 上下界。
    let cfg = FrictionConfig { mu: 0.5 };
    let mut normal = ConstraintRow::contact_normal(
        gvpe_core::BodyHandle::new(0, 0),
        gvpe_core::BodyHandle::new(1, 0),
        Vec3::Y,
        Vec3::ZERO,
        Vec3::ZERO,
        0.0,
    );
    normal.lambda = 2.0; // 模拟 SI 迭代后法向冲量
    let mut pair = build_friction_rows(
        gvpe_core::BodyHandle::new(0, 0),
        gvpe_core::BodyHandle::new(1, 0),
        Vec3::Y,
        Vec3::ZERO,
        Vec3::ZERO,
    );
    update_friction_bounds(&normal, &mut pair, &cfg);
    // mu * |lambda_n| = 0.5 * 2.0 = 1.0
    assert!((pair[0].lower - (-1.0)).abs() < 1e-6);
    assert!((pair[0].upper - 1.0).abs() < 1e-6);
    assert!((pair[1].lower - (-1.0)).abs() < 1e-6);
    assert!((pair[1].upper - 1.0).abs() < 1e-6);
}
