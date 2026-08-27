//! 刚体状态：`RigidBodyState` + 力累加器。
//!
//! 状态布局：
//!
//! - 平移 [`Vec3`](gvpe_math::Vec3)
//! - 旋转 [`Quat`](gvpe_math::Quat)
//! - 线速度 `lin_vel` / 角速度 `ang_vel`
//! - 受力 `force` / 受力矩 `torque`（per-step 累加，每个 `step()` 末尾清零）
//! - 派生量 `inv_mass` / `inv_inertia_diag`（取自 `PhysicsProfile`，
//!   落地时缓存以避免热路径上反复除法）
//!
//! **半隐式 Euler（symplectic Euler）** 公式（v0.3 MVP 默认）：
//!
//! ```text
//! v_{n+1} = v_n + a * dt           // 先更新速度
//! x_{n+1} = x_n + v_{n+1} * dt     // 再用更新后的速度更新位置
//! ω_{n+1} = ω_n + α * dt
//! q_{n+1} = normalize(q_n + 0.5 * ω_{n+1} * q_n * dt)
//! ```
//!
//! 与显式 Euler 相比，半隐式 Euler 在能量守恒上明显更稳定（无能量漂移），
//! 对游戏运行时刚体场景足够（详见 `07_solver_design.md` §3.1，per 引用时
//! `git log -p --follow` 实证——本 crate 创建时无 design doc，链接为
//! DDD Review 必查项）。
//!
//! **不做**（v0.3 范围外）：
//!
//! - 全惯性张量（仅对角线，假设 body-aligned frame）—— v0.4+ 引入 3×3
//! - 陀螺力（gyroscopic torque `ω × Iω`）—— v0.4+ 评估
//! - 角速度临界阻尼（angular critical damping）—— v0.4+ 评估
//! - 累积冲量（impulse-based）—— v0.5+ 改 Sequential Impulse 时统一处理

use gvpe_math::{Quat, Vec3};

/// 时间步进算法（MVP 提供 3 种，便于 profile 比较）。
///
/// 选型草案（**无 design doc 实证**，DDD Review 必查）：
///
/// | 算法 | 能量守恒 | 复杂度 | 备注 |
/// |---|---|---|---|
/// | [`TimeStepper::SemiImplicitEuler`] | 稳定 | O(n) | MVP 默认 |
/// | [`TimeStepper::ExplicitEuler`] | 漂移 | O(n) | 对照基线 |
/// | [`TimeStepper::Rk4`] | 稳定 | O(4n) | 高精度研究路径 |
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TimeStepper {
    /// 半隐式 Euler（symplectic Euler），MVP 默认。
    #[default]
    SemiImplicitEuler = 0,
    /// 显式 Euler（forward Euler），对照基线。
    ExplicitEuler = 1,
    /// 经典 RK4，高精度研究路径。
    Rk4 = 2,
}

impl TimeStepper {
    /// 是否为 symplectic（能量稳定）算法。
    #[inline]
    #[must_use]
    pub const fn is_symplectic(self) -> bool {
        matches!(self, Self::SemiImplicitEuler)
    }
}

/// 刚体动力学状态。
///
/// **不变量**（[`Self::validate`] 检查）：
///
/// - `inv_mass >= 0`（`0.0` = static body —— 调用方应通过
///   `PhysicsProfile::is_static` 交叉检查）
/// - `inv_inertia_diag` 各分量 `>= 0`
/// - `force` / `torque` 不为 `NaN`
/// - `rotation` 单位四元数（`length ≈ 1`，容差 `1e-3`）
///
/// **热路径友好**：所有字段 `f32` / `Quat` / `Vec3`，POD，
/// 默认 `Default` 给"零状态 + static body"语义（`inv_mass = 0`）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RigidBodyState {
    /// 平移（世界系）。
    pub position: Vec3,
    /// 旋转（世界系，单位四元数）。
    pub rotation: Quat,
    /// 线速度（世界系）。
    pub lin_vel: Vec3,
    /// 角速度（世界系，body 坐标系假设下的对角惯性张量；v0.3 MVP 限制）。
    pub ang_vel: Vec3,
    /// 累计外力（per-step，每个 `step()` 末尾清零）。
    pub force: Vec3,
    /// 累计外力矩（per-step，每个 `step()` 末尾清零）。
    pub torque: Vec3,
    /// 质量倒数（`0.0` 表示 static body）。
    pub inv_mass: f32,
    /// 惯性张量对角线倒数 `[1/Ixx, 1/Iyy, 1/Izz]`（body-aligned，v0.3 MVP）。
    pub inv_inertia_diag: [f32; 3],
}

impl RigidBodyState {
    /// 构造 dynamic body 状态（`mass > 0`）。
    ///
    /// # Panics
    ///
    /// `mass` 或任一惯性分量 `<= 0` 时 panic。
    #[inline]
    #[must_use]
    pub fn dynamic(
        position: Vec3,
        rotation: Quat,
        lin_vel: Vec3,
        ang_vel: Vec3,
        mass: f32,
        inertia_diag: [f32; 3],
    ) -> Self {
        assert!(mass > 0.0, "dynamic body mass 必须 > 0");
        assert!(
            inertia_diag[0] > 0.0 && inertia_diag[1] > 0.0 && inertia_diag[2] > 0.0,
            "dynamic body 惯性分量必须 > 0"
        );
        Self {
            position,
            rotation: rotation.normalize(),
            lin_vel,
            ang_vel,
            force: Vec3::ZERO,
            torque: Vec3::ZERO,
            inv_mass: 1.0 / mass,
            inv_inertia_diag: [
                1.0 / inertia_diag[0],
                1.0 / inertia_diag[1],
                1.0 / inertia_diag[2],
            ],
        }
    }

    /// 构造 static body 状态（`mass == 0` → `inv_mass == 0`，力累加无效）。
    #[inline]
    #[must_use]
    pub fn fixed(position: Vec3, rotation: Quat) -> Self {
        Self {
            position,
            rotation: rotation.normalize(),
            lin_vel: Vec3::ZERO,
            ang_vel: Vec3::ZERO,
            force: Vec3::ZERO,
            torque: Vec3::ZERO,
            inv_mass: 0.0,
            inv_inertia_diag: [0.0, 0.0, 0.0],
        }
    }

    /// 是否为 static body（`inv_mass == 0`）。
    #[inline]
    #[must_use]
    pub fn is_static(&self) -> bool {
        self.inv_mass == 0.0
    }

    /// 累加外力（per-step，多次调用累加；`step()` 末尾清零）。
    ///
    /// static body 调用为 no-op（防误用），**不**报错。
    #[inline]
    pub fn add_force(&mut self, f: Vec3) {
        if self.is_static() {
            return;
        }
        self.force = self.force + f;
    }

    /// 累加外力矩（per-step，多次调用累加；`step()` 末尾清零）。
    ///
    /// static body 调用为 no-op（防误用），**不**报错。
    #[inline]
    pub fn add_torque(&mut self, t: Vec3) {
        if self.is_static() {
            return;
        }
        self.torque = self.torque + t;
    }

    /// 清零力累加器（在 `step()` 末尾调用）。
    #[inline]
    pub fn clear_accumulators(&mut self) {
        self.force = Vec3::ZERO;
        self.torque = Vec3::ZERO;
    }

    /// 验证状态不变式。
    ///
    /// 检查：
    /// - `inv_mass >= 0` 且非 `NaN`
    /// - `inv_inertia_diag` 各分量 `>= 0` 且非 `NaN`
    /// - `force` / `torque` 各分量非 `NaN`
    /// - `rotation` 长度与 1 偏差 < `1e-3`
    ///
    /// 失败时返回 [`crate::error::DynamicsError::StateNotFinite`]
    /// （MVP 不区分 NaN / Inf / rotation-unnormalized，DDD Review 必查）。
    pub fn validate(&self) -> Result<(), crate::error::DynamicsError> {
        use crate::error::DynamicsError;

        if self.inv_mass.is_nan() || self.inv_mass < 0.0 {
            return Err(DynamicsError::StateNotFinite {
                field: "inv_mass",
                value: self.inv_mass,
            });
        }
        for (i, &d) in self.inv_inertia_diag.iter().enumerate() {
            if d.is_nan() || d < 0.0 {
                return Err(DynamicsError::StateNotFinite {
                    field: match i {
                        0 => "inv_inertia_diag[0]",
                        1 => "inv_inertia_diag[1]",
                        _ => "inv_inertia_diag[2]",
                    },
                    value: d,
                });
            }
        }
        for &f in &[self.force.x, self.force.y, self.force.z] {
            if !f.is_finite() {
                return Err(DynamicsError::StateNotFinite {
                    field: "force",
                    value: f,
                });
            }
        }
        for &t in &[self.torque.x, self.torque.y, self.torque.z] {
            if !t.is_finite() {
                return Err(DynamicsError::StateNotFinite {
                    field: "torque",
                    value: t,
                });
            }
        }
        let qlen_sq = self.rotation.length_squared();
        let deviation = (qlen_sq - 1.0).abs();
        if deviation > 1e-3 {
            return Err(DynamicsError::StateNotFinite {
                field: "rotation",
                value: qlen_sq,
            });
        }
        Ok(())
    }
}

impl Default for RigidBodyState {
    /// `Default` = 零状态 + static body（`inv_mass = 0`，力累加 no-op）。
    ///
    /// 选用 static 而非 dynamic 是因为 `Default` 不携带 `mass` 信息，
    /// 与 [`PhysicsProfile::default_static`](gvpe_core::PhysicsProfile::default_static)
    /// 行为对齐。
    fn default() -> Self {
        Self::fixed(Vec3::ZERO, Quat::IDENTITY)
    }
}
