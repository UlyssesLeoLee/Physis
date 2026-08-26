//! `BodySpec` 与 `RuntimeDescriptor`。
//!
//! 依据 `GVPE-DOC-17` §1.3。

use gvpe_math::Vec3;

use crate::error::CoreError;
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

impl BodySpec {
    /// 创建 builder（链式构造 `BodySpec`，必填字段在 `build()` 时校验）。
    ///
    /// 默认值：
    /// - `shape`: **None**（必填）
    /// - `initial_transform`: `Vec3::ZERO` + 零旋转
    /// - `profile`: [`PhysicsProfile::default_solid`]
    /// - `is_static`: `false`
    ///
    /// 链式示例：
    /// ```ignore
    /// let spec = BodySpec::builder()
    ///     .shape(ShapeDesc::Sphere { radius: 0.5 })
    ///     .transform(InitialTransform { translation: Vec3::new(0.0, 10.0, 0.0), rotation_yaw_pitch_roll: [0.0, 0.0, 0.0] })
    ///     .profile(PhysicsProfile::default_solid())
    ///     .build()?;
    /// ```
    #[inline]
    pub const fn builder() -> BodySpecBuilder {
        BodySpecBuilder::new()
    }
}

/// [`BodySpec`] 的链式构造器。
///
/// 所有字段为 `Option<…>`，未设置则 `build()` 失败（必填字段 `shape` /
/// `initial_transform` / `profile`）或使用默认值（`is_static`）。
///
/// `build()` 期间会调用 [`PhysicsProfile::validate`] 校验 profile 不变式，
/// 并交叉检查 `is_static` 与 `mass` 一致性。
#[derive(Clone, Debug)]
pub struct BodySpecBuilder {
    shape: Option<ShapeDesc>,
    initial_transform: Option<InitialTransform>,
    profile: Option<PhysicsProfile>,
    is_static: Option<bool>,
}

impl BodySpecBuilder {
    /// 新建 builder（全部字段未填）。
    #[inline]
    pub const fn new() -> Self {
        Self {
            shape: None,
            initial_transform: None,
            profile: None,
            is_static: None,
        }
    }

    /// 设置形状（**必填**）。
    #[inline]
    #[must_use]
    pub const fn shape(mut self, shape: ShapeDesc) -> Self {
        self.shape = Some(shape);
        self
    }

    /// 设置初始变换（**必填**）。
    #[inline]
    #[must_use]
    pub const fn transform(mut self, transform: InitialTransform) -> Self {
        self.initial_transform = Some(transform);
        self
    }

    /// 设置物理 profile（**必填**）。
    #[inline]
    #[must_use]
    pub const fn profile(mut self, profile: PhysicsProfile) -> Self {
        self.profile = Some(profile);
        self
    }

    /// 设置 `is_static`（默认 `false`）。
    ///
    /// Rust 关键字 `static` 被保留，因此保留尾部下划线。
    /// [`is_static`](Self::is_static) 是无下划线别名，便于阅读。
    #[inline]
    #[must_use]
    pub const fn static_(mut self, is_static: bool) -> Self {
        self.is_static = Some(is_static);
        self
    }

    /// 同 [`static_`](Self::static_)，仅为 API 友好别名。
    #[inline]
    #[must_use]
    pub const fn is_static(self, is_static: bool) -> Self {
        self.static_(is_static)
    }

    /// 构造 `BodySpec`，校验必填 + profile 不变式 + static 交叉检查。
    pub fn build(self) -> Result<BodySpec, CoreError> {
        let shape = self
            .shape
            .ok_or(CoreError::BodySpecMissingField { field: "shape" })?;
        let initial_transform = self
            .initial_transform
            .ok_or(CoreError::BodySpecMissingField {
                field: "initial_transform",
            })?;
        let profile = self
            .profile
            .ok_or(CoreError::BodySpecMissingField { field: "profile" })?;
        let is_static = self.is_static.unwrap_or(false);

        // profile 自身不变式。
        profile.validate()?;

        // 交叉：is_static 与 mass 一致。
        if profile.is_static() != is_static {
            return Err(CoreError::ProfileInconsistent {
                field: "is_static",
                value: if is_static { 1.0 } else { 0.0 },
            });
        }

        Ok(BodySpec {
            shape,
            initial_transform,
            profile,
            is_static,
        })
    }
}

impl Default for BodySpecBuilder {
    fn default() -> Self {
        Self::new()
    }
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DeterminismMode {
    /// 性能优先，可能非确定性（libm 差异、SIMD 求和顺序）。
    #[default]
    BestEffort,
    /// 严格确定性（feature = "deterministic" 开启）。
    Strict,
}

impl RuntimeDescriptor {
    /// 构造空 Runtime。
    pub const fn empty() -> Self {
        Self {
            bodies: Vec::new(),
            gravity: Vec3::new(0.0, -9.81, 0.0),
            determinism_mode: DeterminismMode::BestEffort,
            thread_pool_size: None,
        }
    }

    /// 预分配容量的空 Runtime（避免后续 `push` 触发多次 realloc）。
    ///
    /// `cap` 仅是 `bodies` Vec 的 `capacity` 提示，不影响语义。
    #[inline]
    #[must_use]
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            bodies: Vec::with_capacity(cap),
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
    #[inline]
    #[must_use]
    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }

    /// 索引访问 body（越界返 `None`）。
    #[inline]
    #[must_use]
    pub fn body(&self, idx: usize) -> Option<&BodySpec> {
        self.bodies.get(idx)
    }

    /// 索引访问 body（可变，越界返 `None`）。
    #[inline]
    #[must_use]
    pub fn body_mut(&mut self, idx: usize) -> Option<&mut BodySpec> {
        self.bodies.get_mut(idx)
    }

    /// 静态 body 数（`is_static == true`）。
    #[inline]
    #[must_use]
    pub fn static_body_count(&self) -> usize {
        self.bodies.iter().filter(|b| b.is_static).count()
    }

    /// 动态 body 数（`is_static == false`）。
    #[inline]
    #[must_use]
    pub fn dynamic_body_count(&self) -> usize {
        self.bodies.iter().filter(|b| !b.is_static).count()
    }

    /// 校验 `RuntimeDescriptor` 不变式。
    ///
    /// 检查项：
    /// - 至少 1 个 body（否则 [`CoreError::DescriptorEmpty`]）
    /// - 每个 body 的 [`PhysicsProfile::validate`] 通过
    /// - 每个 body 的 `is_static` 与 `profile.is_static()` 一致
    ///
    /// **MVP 限制**：`BodySpec` 当前无显式 `id` 字段，
    /// [`CoreError::DuplicateBodyIndex`] 变体已预留但**不**在本检查中触发；
    /// 引入 `id` 字段后扩展（详见 `GVPE-DOC-17` §1.3 待办）。
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.bodies.is_empty() {
            return Err(CoreError::DescriptorEmpty);
        }
        for b in &self.bodies {
            b.profile.validate()?;
            if b.profile.is_static() != b.is_static {
                return Err(CoreError::ProfileInconsistent {
                    field: "is_static",
                    value: if b.is_static { 1.0 } else { 0.0 },
                });
            }
        }
        Ok(())
    }
}
