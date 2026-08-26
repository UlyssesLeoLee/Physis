//! `PhysicsProfile`：POD 数据结构。
//!
//! 依据 `GVPE-DOC-17` §1.2 与 `GVPE-DOC-58` §2.2。
//! 80 字节，`#[repr(C)]` 保证布局稳定。

use bytemuck::{Pod, Zeroable};

/// 求解器类型（u8 枚举）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SolverTypeId {
    /// Sequential Impulse（MVP）。
    SequentialImpulse = 0,
    /// XPBD（Gen 2，reserved / 未实现）。
    Xpbd = 1,
}

/// Physics LOD（u8 枚举）。
///
/// MVP 仅使用 `Lod0Full`；其他 LOD 槽位保留（`R-FR-007`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PhysicsLodTag {
    /// LOD 0：全精度物理（每帧完整求解）。
    Lod0Full = 0,
    /// LOD 1：精简物理（降频 / 简化求解器）。
    Lod1Reduced = 1,
    /// LOD 2：近似物理（外推 / 拟合）。
    Lod2Approximation = 2,
    /// LOD 3：缓存行为（外推前一帧结果）。
    Lod3CachedBehavior = 3,
    /// LOD 4：静态（不求解，仅广播 transform）。
    Lod4Static = 4,
}

/// `PhysicsProfile`：POD 物理属性（80 字节）。
///
/// 字段顺序固定（`#[repr(C)]` 锁定），按访问频率从高到低排列。
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct PhysicsProfile {
    /// 质量（kg）。
    pub mass: f32,
    /// 密度（kg/m³）。
    pub density: f32,
    /// 惯性张量（3x3，行优先）。
    pub inertia: [f32; 9],
    /// 摩擦系数。
    pub friction: f32,
    /// 弹性恢复系数。
    pub restitution: f32,
    /// 线速度阻尼。
    pub damping_linear: f32,
    /// 角速度阻尼。
    pub damping_angular: f32,
    /// 刚度。
    pub stiffness: f32,
    /// 柔度（XPBD 兼容字段，MVP 求解器不读）。
    pub compliance: f32,
    /// 黏度。
    pub viscosity: f32,
    /// 求解器类型（u8）。
    pub solver_type: SolverTypeId,
    /// 求解迭代次数。
    pub solver_iterations: u16,
    /// 碰撞 profile ID（u8）。
    pub collision_profile: u8,
    /// 物理 LOD 等级（u8）。
    pub approximation_level: PhysicsLodTag,
    /// 显式 padding（避免编译器 reorder）。
    #[allow(clippy::pub_underscore_fields)]
    pub _padding: [u8; 1],
}

// SAFETY: 所有字段为 `Pod`（f32 / u16 / u8），`_padding` 显式，无隐式 padding。
unsafe impl Pod for PhysicsProfile {}
unsafe impl Zeroable for PhysicsProfile {}

impl PhysicsProfile {
    /// 默认 profile（MVP 典型值）。
    pub fn default_solid() -> Self {
        Self {
            mass: 1.0,
            density: 1000.0,
            inertia: [
                1.0 / 6.0,
                0.0,
                0.0,
                0.0,
                1.0 / 6.0,
                0.0,
                0.0,
                0.0,
                1.0 / 6.0,
            ],
            friction: 0.5,
            restitution: 0.1,
            damping_linear: 0.01,
            damping_angular: 0.01,
            stiffness: 0.0,
            compliance: 0.0,
            viscosity: 0.0,
            solver_type: SolverTypeId::SequentialImpulse,
            solver_iterations: 10,
            collision_profile: 0,
            approximation_level: PhysicsLodTag::Lod0Full,
            _padding: [0],
        }
    }

    /// 静态 body 的 profile（mass = 0）。
    pub fn default_static() -> Self {
        let mut p = Self::default_solid();
        p.mass = 0.0;
        p
    }

    /// 单 `mass` 参数构造 profile，其余字段取 [`Self::default_solid`] 默认值。
    ///
    /// - `mass > 0` → 返回 `default_solid` 并覆盖 `mass`（典型动态 body 用法）
    /// - `mass <= 0` → 走 [`Self::default_static`]（即 `mass == 0` 的 static body）
    ///
    /// **设计选择**：用 `mass` 符号覆盖 `dynamic` / `static` 切换，不分裂成两份
    /// `from_mass_dynamic` / `from_mass_static`（保持 API 简洁；与 [`Self::is_static`]
    /// 的 `mass == 0` 判定一致）。负值会被静默映射到 `default_static` 而**不**报
    /// `CoreError::ProfileInconsistent`——本函数不返 `Result`；调用方若需严格区分
    /// 应在传入前检查或调用 [`Self::validate`]。
    pub fn from_mass(mass: f32) -> Self {
        if mass > 0.0 {
            let mut p = Self::default_solid();
            p.mass = mass;
            p
        } else {
            Self::default_static()
        }
    }

    /// 验证 profile 不变式。
    ///
    /// 全部不变量见 `GVPE-DOC-17` §1.2。失败时返回首个违反的字段（便于上层定位）。
    ///
    /// ## 检查项
    ///
    /// - `mass >= 0`（`mass == 0` 表示 static body —— 调用方需自行保证 `is_static == true`）
    /// - 若 `mass > 0`，则 `density > 0` 且惯性对角线 > 0
    /// - `friction ∈ [0, 1]`、`restitution ∈ [0, 1]`
    /// - `damping_linear / damping_angular ∈ [0, 1]`
    /// - `compliance >= 0`、`viscosity >= 0`
    /// - `solver_iterations >= 1`
    /// - 所有浮点字段非 `NaN`
    ///
    /// 注：是否为 static 由 [`crate::descriptor::BodySpec::is_static`] 决定，
    /// `PhysicsProfile` 本身不携带该信息（与 `BodySpec` 解耦）。
    pub fn validate(&self) -> Result<(), crate::error::CoreError> {
        use crate::error::CoreError;

        // 1. NaN 检测（所有浮点字段）。
        Self::check_not_nan(self.mass, "mass")?;
        Self::check_not_nan(self.density, "density")?;
        Self::check_not_nan(self.friction, "friction")?;
        Self::check_not_nan(self.restitution, "restitution")?;
        Self::check_not_nan(self.damping_linear, "damping_linear")?;
        Self::check_not_nan(self.damping_angular, "damping_angular")?;
        Self::check_not_nan(self.compliance, "compliance")?;
        Self::check_not_nan(self.viscosity, "viscosity")?;

        // 2. mass >= 0（mass == 0 合法 → static body）。
        if self.mass < 0.0 {
            return Err(CoreError::ProfileInconsistent {
                field: "mass",
                value: self.mass,
            });
        }

        // 3. 若 mass > 0，density 必须 > 0；惯性对角线必须 > 0。
        if self.mass > 0.0 {
            if self.density <= 0.0 {
                return Err(CoreError::ProfileInconsistent {
                    field: "density",
                    value: self.density,
                });
            }
            let diag = self.inertia_diagonal();
            for (i, &d) in diag.iter().enumerate() {
                if d <= 0.0 {
                    return Err(CoreError::ProfileInconsistent {
                        field: match i {
                            0 => "inertia[0]",
                            1 => "inertia[4]",
                            _ => "inertia[8]",
                        },
                        value: d,
                    });
                }
            }
        }

        // 4. 范围检查。
        Self::check_unit_range(self.friction, "friction")?;
        Self::check_unit_range(self.restitution, "restitution")?;
        Self::check_unit_range(self.damping_linear, "damping_linear")?;
        Self::check_unit_range(self.damping_angular, "damping_angular")?;
        if self.compliance < 0.0 {
            return Err(CoreError::ProfileInconsistent {
                field: "compliance",
                value: self.compliance,
            });
        }
        if self.viscosity < 0.0 {
            return Err(CoreError::ProfileInconsistent {
                field: "viscosity",
                value: self.viscosity,
            });
        }

        // 5. 求解器迭代。
        if self.solver_iterations < 1 {
            return Err(CoreError::ProfileInconsistent {
                field: "solver_iterations",
                // u16 → f32（仅用于 Display）
                value: f32::from(self.solver_iterations),
            });
        }

        Ok(())
    }

    /// NaN 检测辅助：若 `v` 是 `NaN` 返回 `ProfileInconsistent`。
    #[inline]
    fn check_not_nan(v: f32, field: &'static str) -> Result<(), crate::error::CoreError> {
        use crate::error::CoreError;
        if v.is_nan() {
            Err(CoreError::ProfileInconsistent { field, value: v })
        } else {
            Ok(())
        }
    }

    /// 范围检查辅助：`v ∈ [0, 1]`。
    #[inline]
    fn check_unit_range(v: f32, field: &'static str) -> Result<(), crate::error::CoreError> {
        use crate::error::CoreError;
        if (0.0..=1.0).contains(&v) {
            Ok(())
        } else {
            Err(CoreError::ProfileInconsistent { field, value: v })
        }
    }

    /// 是否为 static body（`mass == 0.0`）。
    ///
    /// 注：`BodySpec` 自身有 `is_static` 字段；本方法仅基于 `mass` 判断，
    /// 两者**应保持一致**（`BodySpecBuilder::build` 会做交叉检查）。
    #[inline]
    pub fn is_static(&self) -> bool {
        self.mass == 0.0
    }

    /// 提取惯性张量对角线 `[Ixx, Iyy, Izz]`。
    ///
    /// `inertia` 字段为 3×3 行优先矩阵（`#[repr(C)]` 锁定布局），
    /// 对角元素索引为 `0 / 4 / 8`。
    #[inline]
    pub const fn inertia_diagonal(&self) -> [f32; 3] {
        [self.inertia[0], self.inertia[4], self.inertia[8]]
    }

    /// 惯性张量对角线倒数 `[1/Ixx, 1/Iyy, 1/Izz]`。
    ///
    /// **static body（`mass == 0`）返回 `[0, 0, 0]`** 而不是 `[+inf, +inf, +inf]`，
    /// 以避免后续积分步骤出现 `NaN`（详见 `GVPE-DOC-17` §1.2 末尾备注）。
    /// 调用方无需再做 `is_finite` 判断。
    #[inline]
    pub fn inverse_inertia(&self) -> [f32; 3] {
        if self.is_static() {
            return [0.0, 0.0, 0.0];
        }
        let d = self.inertia_diagonal();
        [
            if d[0] == 0.0 { 0.0 } else { 1.0 / d[0] },
            if d[1] == 0.0 { 0.0 } else { 1.0 / d[1] },
            if d[2] == 0.0 { 0.0 } else { 1.0 / d[2] },
        ]
    }
}

impl Default for PhysicsProfile {
    fn default() -> Self {
        Self::default_solid()
    }
}
