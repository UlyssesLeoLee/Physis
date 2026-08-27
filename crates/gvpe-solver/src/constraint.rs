//! `ConstraintRow`：求解器迭代的最小行格式。
//!
//! 依据 [`GVPE-DOC-07`] §6.1（07_solver_design.md）：
//! 所有约束类型（`ContactConstraint`、摩擦行、未来的 `JointConstraint`）统一为 `ConstraintRow`
//! —— 求解器仅迭代该单一格式，与具体语义来源无关。
//!
//! [`GVPE-DOC-07`]: ../../../docs/02_modules/07_solver_design.md

use gvpe_core::BodyHandle;

/// 求解器单行：两 body 间的线性约束。
///
/// 字段语义（per `07_solver_design.md` §6.1）：
/// - `body_a` / `body_b`：参与约束的两 body。
/// - `jacobian`：约束的雅可比矩阵，平铺为 `[f32; 12]`。
///   布局：`[J_a_lin(3), J_a_ang(3), J_b_lin(3), J_b_ang(3)]`。
/// - `bias`：约束偏差项（穿透 / Baumgarte stabilization）；MVP 用 `beta * penetration / dt`。
/// - `compliance`：XPBD 风格的柔度参数；MVP 不使用但字段保留（§6.2）。
/// - `lambda`：累积冲量（warm-starting 入口 + 帧末保留）。
/// - `lower` / `upper`：冲量边界；摩擦锥 box 约束通过此对实现。
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct ConstraintRow {
    /// Body A 句柄。
    pub body_a: BodyHandle,
    /// Body B 句柄。
    pub body_b: BodyHandle,
    /// 约束雅可比：`[J_a_lin | J_a_ang | J_b_lin | J_b_ang]`，各 3 分量。
    pub jacobian: [f32; 12],
    /// 约束偏差项（Baumgarte stabilization）。
    pub bias: f32,
    /// XPBD 风格柔度参数；MVP = 0（§6.2 字段预留）。
    pub compliance: f32,
    /// 累积冲量；warm-starting 用作下一帧初值。
    pub lambda: f32,
    /// 冲量下界（摩擦行 = `-mu * |lambda_normal|`，单边 = 0）。
    pub lower: f32,
    /// 冲量上界（单边 normal = 0，无拉；摩擦 = `+mu * |lambda_normal|`）。
    pub upper: f32,
}

impl Default for ConstraintRow {
    fn default() -> Self {
        Self {
            body_a: BodyHandle::INVALID,
            body_b: BodyHandle::INVALID,
            jacobian: [0.0; 12],
            bias: 0.0,
            compliance: 0.0,
            lambda: 0.0,
            lower: 0.0,
            upper: 0.0,
        }
    }
}

impl ConstraintRow {
    /// 接触法向行（normal）。
    ///
    /// 标准布局：
    /// - `J_a_lin = -n`, `J_a_ang = -(r_a) × n`
    /// - `J_b_lin = +n`, `J_b_ang = +(r_b) × n`
    ///
    /// 求解后 `lambda >= 0`（无拉）。
    ///
    /// **Bias 约定（Box2D-style）**：调用方传 `bias = -beta * penetration / dt`。
    /// 约束 `J·v >= 0`（分离），故 penetration 进入 `J·v + bias` 时需带负号
    /// 才能让 SI 推 lambda 增大（推开 body）。
    /// 例：`penetration=0.5, beta=0.2, dt=1/60 → bias = -6.0`。
    #[inline]
    #[must_use]
    pub fn contact_normal(
        body_a: BodyHandle,
        body_b: BodyHandle,
        normal: gvpe_math::Vec3,
        r_a: gvpe_math::Vec3,
        r_b: gvpe_math::Vec3,
        bias: f32,
    ) -> Self {
        let neg_n = -normal;
        let ang_a = -r_a.cross(normal);
        let ang_b = r_b.cross(normal);
        let jacobian = [
            neg_n.x, neg_n.y, neg_n.z, ang_a.x, ang_a.y, ang_a.z, normal.x, normal.y, normal.z,
            ang_b.x, ang_b.y, ang_b.z,
        ];
        Self {
            body_a,
            body_b,
            jacobian,
            bias,
            compliance: 0.0,
            lambda: 0.0,
            lower: 0.0,
            upper: f32::INFINITY,
        }
    }

    /// 摩擦行（tangent）。
    ///
    /// `t` 为单位切向（与 normal 正交）。摩擦 box 上下界由 normal impulse 决定，
    /// 在 [`crate::friction`] 模块内根据法向累积冲量更新。
    #[inline]
    #[must_use]
    pub fn friction_tangent(
        body_a: BodyHandle,
        body_b: BodyHandle,
        tangent: gvpe_math::Vec3,
        r_a: gvpe_math::Vec3,
        r_b: gvpe_math::Vec3,
        bias: f32,
    ) -> Self {
        let neg_t = -tangent;
        let ang_a = -r_a.cross(tangent);
        let ang_b = r_b.cross(tangent);
        let jacobian = [
            neg_t.x, neg_t.y, neg_t.z, ang_a.x, ang_a.y, ang_a.z, tangent.x, tangent.y, tangent.z,
            ang_b.x, ang_b.y, ang_b.z,
        ];
        Self {
            body_a,
            body_b,
            jacobian,
            bias,
            compliance: 0.0,
            lambda: 0.0,
            lower: 0.0,
            upper: 0.0, // 由 [crate::friction] 在求解循环中按法向冲量更新
        }
    }

    /// 求解循环内单行：算新冲量并累加到 `lambda`。
    ///
    /// `effective_mass` = `1 / (J * M^{-1} * J^T)`（调用方已预算）。
    /// 公式：`lambda_new = clamp(lambda + delta, lower, upper)`，其中
    /// `delta = -effective_mass * (J * v + bias + compliance * lambda)`（MVP = 0 compliance）。
    ///
    /// 返回冲量增量；调用方用其更新 body 速度。
    #[inline]
    #[must_use]
    pub fn solve_single(&mut self, jv: f32, effective_mass: f32) -> f32 {
        // MVP：忽略 compliance 路径（保留字段为 XPBD 升级用，§6.2）。
        let delta_lambda = -effective_mass * (jv + self.bias);
        let new_lambda = (self.lambda + delta_lambda).clamp(self.lower, self.upper);
        let applied = new_lambda - self.lambda;
        self.lambda = new_lambda;
        applied
    }

    /// 求解循环末：将累积冲量重置为 0（用于一帧结束、冷启动）。
    ///
    /// 通常**不**调用 —— warm-starting 跨帧保留 `lambda`。仅在求解异常 / 重置时使用。
    #[inline]
    pub fn clear_impulse(&mut self) {
        self.lambda = 0.0;
    }
}
