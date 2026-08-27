//! 求解器核心：Sequential Impulse / PGS MVP。
//!
//! 依据 [`GVPE-DOC-07`] §6.1、§7.2、§9（07_solver_design.md）：
//! - 求解循环：warm-start 取上一帧的 `lambda` → 在物理岛内对所有行进行 N 次 GS 扫描
//!   → 每次扫描后投影冲量边界 → 积分。
//! - 摩擦 box 与接触 normal 共用同一 SI 循环（§6.3）。
//! - Sleeping 在 `solve` 末 tick（§6.4）。
//!
//! [`GVPE-DOC-07`]: ../../../docs/02_modules/07_solver_design.md

use crate::body::{BodySlab, RigidBody, SleepState};
use crate::constraint::ConstraintRow;
use crate::error::SolverError;
use crate::friction::FrictionConfig;
use crate::sleep::{tick_sleep, SleepConfig};
use gvpe_math::Vec3;

/// 求解器配置。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SolverConfig {
    /// SI 迭代次数；默认 10。
    pub iterations: u32,
    /// 时间步长（秒）；用于 Baumgarte bias `beta * penetration / dt`。
    pub dt: f32,
    /// Baumgarte 系数（接触穿透位置修正的 0..1 比例）。
    pub bias_beta: f32,
    /// 睡眠配置。
    pub sleep: SleepConfig,
    /// 摩擦配置。
    pub friction: FrictionConfig,
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            iterations: 10,
            dt: 1.0 / 60.0,
            bias_beta: 0.2,
            sleep: SleepConfig::default(),
            friction: FrictionConfig::default(),
        }
    }
}

/// 物理岛（约束集合）。
///
/// 求解器迭代的对象。约束按行（`ConstraintRow`）存储；warm-starting 在 `solve` 调用间
/// 跨帧保留 `row.lambda`。
#[derive(Default, Debug)]
pub struct Island {
    /// 约束行（顺序存储；每行可能为 normal 行或 tangent 行）。
    pub rows: Vec<ConstraintRow>,
}

impl Island {
    /// 构造空岛。
    #[must_use]
    pub fn new() -> Self {
        Self { rows: Vec::new() }
    }

    /// 预分配容量。
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            rows: Vec::with_capacity(capacity),
        }
    }

    /// 添加约束行。
    pub fn push(&mut self, row: ConstraintRow) {
        self.rows.push(row);
    }

    /// 当前约束行数。
    #[must_use]
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// 帧末清零所有行的 `lambda`（用于关闭 warm-starting；一般不调用）。
    pub fn clear_warm_start(&mut self) {
        for row in &mut self.rows {
            row.lambda = 0.0;
        }
    }
}

/// 求解器：持有配置。
///
/// 求解入口 [`Solver::solve`] 流程（per `07_solver_design.md` §9）：
/// 1. 跳过 Sleeping / Static body。
/// 2. 迭代 `iterations` 次：对每行计算 `J * v + bias`，求 effective mass，
///    累加冲量到 `lambda`，并用 `lower/upper` 投影；同时把冲量累加到
///    `body_scratch[body.slot_index]`。
/// 3. 积分（`x += v * dt`，`q += 0.5 * omega * q * dt`），并应用 scratch 冲量。
/// 4. tick sleeping。
#[derive(Debug)]
pub struct Solver {
    cfg: SolverConfig,
    /// 每 body 的线 / 角冲量 scratch（按 slot_index 索引）。
    /// 重用容量以避免每帧分配。
    scratch_lin: Vec<Vec3>,
    scratch_ang: Vec<Vec3>,
}

impl Default for Solver {
    fn default() -> Self {
        Self::new(SolverConfig::default())
    }
}

impl Solver {
    /// 用配置构造求解器。
    #[must_use]
    pub fn new(cfg: SolverConfig) -> Self {
        Self {
            cfg,
            scratch_lin: Vec::new(),
            scratch_ang: Vec::new(),
        }
    }

    /// 配置引用。
    #[must_use]
    pub fn config(&self) -> &SolverConfig {
        &self.cfg
    }

    /// 求解岛 + 积分 + sleep tick。
    ///
    /// 假设 `bodies` 包含所有参与约束的 body（`island.rows` 中引用的 body）。
    /// 约束行按存储顺序迭代；normal 行和 tangent 行不区分（统一 SI 循环）。
    ///
    /// 摩擦 box 同步：按约定，约束布局 = `[normal, t1, t2, normal, t1, t2, ...]`
    /// （每 3 行一接触组）。其它布局下跳过 box 更新（仅做静态上下界）。
    pub fn solve(&mut self, island: &mut Island, bodies: &mut BodySlab) -> Result<(), SolverError> {
        let n_bodies = bodies.len();
        if n_bodies == 0 {
            return Ok(());
        }
        // 重置 scratch。
        self.scratch_lin.clear();
        self.scratch_lin.resize(n_bodies, Vec3::ZERO);
        self.scratch_ang.clear();
        self.scratch_ang.resize(n_bodies, Vec3::ZERO);

        if !island.is_empty() {
            let iterations = self.cfg.iterations.max(1);

            for _ in 0..iterations {
                // 同步摩擦 box（按 normal→t1,t2 三元组）。
                self.sync_friction_bounds(island);

                for row in &mut island.rows {
                    self.solve_row(row, bodies)?;
                }
            }
        }

        // 积分活跃 body（应用 scratch + accumulated force）。
        self.integrate(bodies);

        // sleep tick。
        self.tick_sleep(bodies);

        Ok(())
    }

    /// 同步摩擦 box 上下界（每 3 行一接触组：normal/t1/t2）。
    ///
    /// Heuristic：normal 行的 `bias > 0`（穿透修正），
    /// 摩擦行 `bias == 0`。本函数找到该模式后调用 [`update_friction_bounds`]。
    fn sync_friction_bounds(&self, island: &mut Island) {
        let cfg_fric = &self.cfg.friction;
        let mut i = 0;
        while i + 2 < island.rows.len() {
            let normal_has_bias = island.rows[i].bias.abs() > f32::EPSILON;
            let t1_zero_bias = island.rows[i + 1].bias.abs() <= f32::EPSILON;
            let t2_zero_bias = island.rows[i + 2].bias.abs() <= f32::EPSILON;
            if normal_has_bias && t1_zero_bias && t2_zero_bias {
                let normal_lambda = island.rows[i].lambda;
                let max_friction = cfg_fric.mu * normal_lambda.abs();
                island.rows[i + 1].lower = -max_friction;
                island.rows[i + 1].upper = max_friction;
                island.rows[i + 2].lower = -max_friction;
                island.rows[i + 2].upper = max_friction;
                i += 3;
            } else {
                i += 1;
            }
        }
    }

    /// 单行 SI step。
    ///
    /// 不修改 body 速度；将 `lambda * J` 累加到 scratch，在 [`Solver::integrate`]
    /// 阶段统一应用（避免借用冲突）。
    fn solve_row(&mut self, row: &mut ConstraintRow, bodies: &BodySlab) -> Result<(), SolverError> {
        let a = bodies.get(row.body_a).ok_or(SolverError::BodyNotFound {
            index: row.body_a.index,
            generation: row.body_a.generation,
        })?;
        let b = bodies.get(row.body_b).ok_or(SolverError::BodyNotFound {
            index: row.body_b.index,
            generation: row.body_b.generation,
        })?;

        // J * v：12 维 J 点积 body 速度。
        let jv = j_dot_v(row.jacobian, a, b);

        // Effective mass K = J * M^{-1} * J^T。
        let k = effective_mass(row.jacobian, a, b);
        if k <= 0.0 || !k.is_finite() {
            return Err(SolverError::DegenerateJacobian);
        }
        let inv_k = 1.0 / k;

        // SI 步。
        let delta_lambda = -inv_k * (jv + row.bias);
        let new_lambda = (row.lambda + delta_lambda).clamp(row.lower, row.upper);
        let applied = new_lambda - row.lambda;
        row.lambda = new_lambda;

        // 把冲量记入 scratch（按 slot_index 索引）。
        // 安全：slot_index < n_bodies 来自 BodySlab 的构造。
        let sa = a.slot_index as usize;
        let sb = b.slot_index as usize;
        // a_impulse = +applied * J_a (因 J_a 形如 [-n, ..., +n])
        // b_impulse = -applied * J_b（用对称：J_b = -J_a_lin / 同样的 J_b_ang）
        // 实际公式：v_a += applied * J_a_lin / m_a, w_a += applied * I_a^-1 * J_a_ang
        //         v_b += applied * J_b_lin / m_b, w_b += applied * I_b^-1 * J_b_ang
        //         （其中 J_a_lin = -J_b_lin，对称）
        // 这里累加到 scratch，最后 integrate 阶段乘 inv_mass / inv_inertia。
        if sa < self.scratch_lin.len() {
            self.scratch_lin[sa] = self.scratch_lin[sa]
                + Vec3::new(
                    row.jacobian[0] * applied,
                    row.jacobian[1] * applied,
                    row.jacobian[2] * applied,
                );
            self.scratch_ang[sa] = self.scratch_ang[sa]
                + Vec3::new(
                    row.jacobian[3] * applied,
                    row.jacobian[4] * applied,
                    row.jacobian[5] * applied,
                );
        }
        if sb < self.scratch_lin.len() {
            self.scratch_lin[sb] = self.scratch_lin[sb]
                + Vec3::new(
                    row.jacobian[6] * applied,
                    row.jacobian[7] * applied,
                    row.jacobian[8] * applied,
                );
            self.scratch_ang[sb] = self.scratch_ang[sb]
                + Vec3::new(
                    row.jacobian[9] * applied,
                    row.jacobian[10] * applied,
                    row.jacobian[11] * applied,
                );
        }
        Ok(())
    }

    /// 积分活跃 body。
    fn integrate(&mut self, bodies: &mut BodySlab) {
        let dt = self.cfg.dt;
        // 先把所有 scratch 复制到 local 数组再 iterate mut bodies（避免借用冲突）。
        let lin: Vec<Vec3> = self.scratch_lin.clone();
        let ang: Vec<Vec3> = self.scratch_ang.clone();
        for body in bodies.iter_mut() {
            if body.is_static() {
                continue;
            }
            if body.sleep == SleepState::Sleeping {
                continue;
            }
            let idx = body.slot_index as usize;
            // 应用 scratch 冲量（按 inv_mass / inv_inertia 缩放）。
            if idx < lin.len() {
                let dlin = lin[idx];
                let dang = ang[idx];
                body.lin_vel = body.lin_vel + dlin * body.inv_mass;
                // 角冲量在世界系 → 用 inv_inertia（假设 body 主轴 = 世界轴）。
                body.ang_vel = body.ang_vel
                    + Vec3::new(
                        dang.x * body.inv_inertia_diag[0],
                        dang.y * body.inv_inertia_diag[1],
                        dang.z * body.inv_inertia_diag[2],
                    );
            }
            // 应用 accumulated force（半隐式 Euler 速度更新）。
            body.lin_vel = body.lin_vel + body.accumulated_force * (body.inv_mass * dt);
            // 力矩在 world 系：MVP 假设 inv_inertia 在 world 系（body 主轴对齐世界轴），
            // 直接乘即可。
            body.ang_vel = body.ang_vel
                + Vec3::new(
                    body.accumulated_torque.x * body.inv_inertia_diag[0] * dt,
                    body.accumulated_torque.y * body.inv_inertia_diag[1] * dt,
                    body.accumulated_torque.z * body.inv_inertia_diag[2] * dt,
                );

            // 位置 / 旋转积分。
            body.position = body.position + body.lin_vel * dt;
            let half_dt = 0.5 * dt;
            let omega_quat = gvpe_math::Quat::new(
                body.ang_vel.x * half_dt,
                body.ang_vel.y * half_dt,
                body.ang_vel.z * half_dt,
                0.0,
            );
            let dq = omega_quat.mul(body.rotation);
            body.rotation = gvpe_math::Quat::new(
                body.rotation.x + dq.x,
                body.rotation.y + dq.y,
                body.rotation.z + dq.z,
                body.rotation.w + dq.w,
            )
            .normalize();

            // 清零 accumulated force。
            body.accumulated_force = Vec3::ZERO;
            body.accumulated_torque = Vec3::ZERO;
        }
    }

    /// Tick sleep for all bodies.
    fn tick_sleep(&self, bodies: &mut BodySlab) {
        let cfg = &self.cfg.sleep;
        for body in bodies.iter_mut() {
            tick_sleep(body, cfg);
        }
    }
}

// ───────── 辅助函数（独立，测试可单独覆盖）─────────

/// `J · v`：12 维 Jacobian 点积两 body 的速度。
#[inline]
#[must_use]
pub fn j_dot_v(j: [f32; 12], a: &RigidBody, b: &RigidBody) -> f32 {
    // J = [J_a_lin(3), J_a_ang(3), J_b_lin(3), J_b_ang(3)]
    // v = [v_a_lin(3), w_a(3), v_b_lin(3), w_b(3)]
    let mut s = 0.0;
    s += j[0] * a.lin_vel.x + j[1] * a.lin_vel.y + j[2] * a.lin_vel.z;
    s += j[3] * a.ang_vel.x + j[4] * a.ang_vel.y + j[5] * a.ang_vel.z;
    s += j[6] * b.lin_vel.x + j[7] * b.lin_vel.y + j[8] * b.lin_vel.z;
    s += j[9] * b.ang_vel.x + j[10] * b.ang_vel.y + j[11] * b.ang_vel.z;
    s
}

/// Effective mass: `K = J · M⁻¹ · Jᵀ`。
#[inline]
#[must_use]
pub fn effective_mass(j: [f32; 12], a: &RigidBody, b: &RigidBody) -> f32 {
    let a_lin_sq = j[0] * j[0] + j[1] * j[1] + j[2] * j[2];
    let b_lin_sq = j[6] * j[6] + j[7] * j[7] + j[8] * j[8];
    let a_ang_term = j[3] * j[3] * a.inv_inertia_diag[0]
        + j[4] * j[4] * a.inv_inertia_diag[1]
        + j[5] * j[5] * a.inv_inertia_diag[2];
    let b_ang_term = j[9] * j[9] * b.inv_inertia_diag[0]
        + j[10] * j[10] * b.inv_inertia_diag[1]
        + j[11] * j[11] * b.inv_inertia_diag[2];
    a.inv_mass * a_lin_sq + b.inv_mass * b_lin_sq + a_ang_term + b_ang_term
}
