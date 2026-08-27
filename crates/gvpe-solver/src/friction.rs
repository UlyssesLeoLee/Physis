//! 摩擦：库仑摩擦锥 → box 近似。
//!
//! 依据 [`GVPE-DOC-07`] §6.3（07_solver_design.md）：
//! - Coulomb 摩擦锥近似为 box 约束（边界由法向冲量 × 摩擦系数推导）。
//! - box 约束在 sequential-impulse 循环内一并处理，
//!   **避免**额外的摩擦锥独立求解 pass。
//!
//! [`GVPE-DOC-07`]: ../../../docs/02_modules/07_solver_design.md

use crate::constraint::ConstraintRow;
use gvpe_math::Vec3;

/// 摩擦配置。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrictionConfig {
    /// 摩擦系数 μ（Coulomb）。`mu * normal_impulse` = 最大切向力。
    pub mu: f32,
}

impl Default for FrictionConfig {
    fn default() -> Self {
        Self { mu: 0.3 }
    }
}

/// 给定接触法向 `n`，返回两个正交单位切向 `(t1, t2)`，满足 `t1 ⊥ t2 ⊥ n`。
///
/// 选 `t1` 为与 `n` 正交且最对齐世界轴的向量，再叉积得 `t2`。
/// 返回的两向量都是单位长度。
#[inline]
#[must_use]
pub fn tangent_pair(n: Vec3) -> (Vec3, Vec3) {
    // 选与 n 最不正交的轴作种子（避免数值退化）。
    let ax = n.x.abs();
    let ay = n.y.abs();
    let az = n.z.abs();
    let seed = if ax <= ay && ax <= az {
        Vec3::X
    } else if ay <= az {
        Vec3::Y
    } else {
        Vec3::Z
    };
    let t1 = n.cross(seed).normalize();
    let t2 = n.cross(t1);
    (t1, t2)
}

/// 计算摩擦 box 上下界：`[-mu * |lambda_n|, +mu * |lambda_n|]`。
///
/// 动态绑定：每帧 SI 循环中随法向冲量更新（Box2D-style）。
/// `lambda_n_acc` 为当前帧法向累积冲量（>= 0）。
#[inline]
#[must_use]
pub fn friction_bounds(cfg: &FrictionConfig, lambda_n_acc: f32) -> (f32, f32) {
    let max_friction = cfg.mu * lambda_n_acc.abs();
    (-max_friction, max_friction)
}

/// 同步摩擦行的 `lower` / `upper` 边界。
///
/// 摩擦行 = `lambda_tangent`（累加切向冲量）。在 SI 循环中，本函数在法向冲量
/// 更新后调用，更新两条摩擦行的 box 上下界。
pub fn update_friction_bounds(
    normal_row: &ConstraintRow,
    friction_rows: &mut [ConstraintRow; 2],
    cfg: &FrictionConfig,
) {
    let (lo, hi) = friction_bounds(cfg, normal_row.lambda);
    for row in friction_rows.iter_mut() {
        row.lower = lo;
        row.upper = hi;
    }
}

/// 为一个接触行生成 2 条摩擦行。
///
/// - 接触点 `r_a`（从 body A 中心到接触点）、`r_b`（从 body B 中心到接触点）；
/// - 法向 `n`（`a → b`，单位长度）；
/// - 摩擦行共享 `bias = 0`（无穿透修正）；
/// - 初始 `lower/upper = 0`，由 [`update_friction_bounds`] 在求解循环中按法向冲量更新。
pub fn build_friction_rows(
    body_a: gvpe_core::BodyHandle,
    body_b: gvpe_core::BodyHandle,
    n: Vec3,
    r_a: Vec3,
    r_b: Vec3,
) -> [ConstraintRow; 2] {
    let (t1, t2) = tangent_pair(n);
    let r1 = ConstraintRow::friction_tangent(body_a, body_b, t1, r_a, r_b, 0.0);
    let r2 = ConstraintRow::friction_tangent(body_a, body_b, t2, r_a, r_b, 0.0);
    [r1, r2]
}
