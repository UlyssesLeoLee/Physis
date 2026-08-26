//! `BodySpec` 与 `RuntimeDescriptor`。
//!
//! 依据 `GVPE-DOC-17` §1.3。

use gvpe_math::Vec3;

use crate::profile::PhysicsProfile;

/// 形状描述（MVP 仅 Sphere / Box3 / Plane）。
///
/// 详细设计在 `gvpe-shape` crate（`GVPE-DOC-04` §4.1 / `GVPE-DOC-06` §6.1）。
/// 此处为占位 enum，待 `gvpe-shape` 实现后替换为完整版本。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShapeDesc {
    /// 球体。
    Sphere {
        /// 球半径。
        radius: f32,
    },
    /// 盒。
    Box3 {
        /// 半尺寸 (x, y, z)。
        half_extents: [f32; 3],
    },
    /// 平面。
    Plane {
        /// 平面法线（归一化）。
        normal: [f32; 3],
        /// 平面到原点的有符号偏移。
        offset: f32,
    },
}

/// 初始变换（位置 + 旋转）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InitialTransform {
    /// 初始平移。
    pub translation: Vec3,
    /// 初始旋转（简化：yaw / pitch / roll，弧度）。
    pub rotation_yaw_pitch_roll: [f32; 3],
}

/// Body 规格（场景加载时使用）。
#[derive(Clone, Debug)]
pub struct BodySpec {
    /// 形状描述。
    pub shape: ShapeDesc,
    /// 初始变换。
    pub initial_transform: InitialTransform,
    /// 物理 profile。
    pub profile: PhysicsProfile,
    /// 是否为静态 body（mass = 0）。
    pub is_static: bool,
}

/// Runtime 描述符：场景 + 全局参数。
///
/// 详见 `GVPE-DOC-17` §1.3。
#[derive(Clone, Debug)]
pub struct RuntimeDescriptor {
    /// Body 列表。
    pub bodies: Vec<BodySpec>,
    /// 重力（m/s²）。
    pub gravity: Vec3,
    /// 确定性模式（骨架，见 `GVPE-DOC-05` §5.3 + DEC-006）。
    pub determinism_mode: DeterminismMode,
    /// 线程池大小（None = 主机线程池）。
    pub thread_pool_size: Option<u32>,
}

/// 确定性模式。
///
/// MVP 实际行为均为 `BestEffort`（架构区分已就位，详见 DEC-006）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeterminismMode {
    /// 性能优先，可能非确定性（libm 差异、SIMD 求和顺序）。
    BestEffort,
    /// 严格确定性（feature = "deterministic" 开启）。
    Strict,
}

impl Default for DeterminismMode {
    fn default() -> Self {
        Self::BestEffort
    }
}

impl RuntimeDescriptor {
    /// 构造空 Runtime。
    pub fn empty() -> Self {
        Self {
            bodies: Vec::new(),
            gravity: Vec3::new(0.0, -9.81, 0.0),
            determinism_mode: DeterminismMode::BestEffort,
            thread_pool_size: None,
        }
    }

    /// 添加 body。
    pub fn add_body(&mut self, spec: BodySpec) {
        self.bodies.push(spec);
    }

    /// body 数量。
    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }
}
