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
    Lod0Full = 0,
    Lod1Reduced = 1,
    Lod2Approximation = 2,
    Lod3CachedBehavior = 3,
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
                1.0 / 6.0, 0.0, 0.0, 0.0, 1.0 / 6.0, 0.0, 0.0, 0.0, 1.0 / 6.0,
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
}

impl Default for PhysicsProfile {
    fn default() -> Self {
        Self::default_solid()
    }
}
