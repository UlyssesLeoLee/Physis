//! `Shape` trait + `ShapeType` enum + `ShapeHandle`。
//!
//! 详见 `GVPE-DOC-21` §2（形状抽象设计）。
//!
//! ## 设计取舍
//!
//! - **`Shape` trait 仅暴露局部坐标 AABB**：世界坐标 AABB 需要 `Transform`（gvpe-core / gvpe-collision 关注），
//!   本 crate 故意不依赖；调用方拿到 `local_aabb` 后自行施加 transform。
//! - **`ShapeHandle = Arc<dyn Shape>`**：克隆廉价（原子 refcount bump），多 body 共享同一形状资产。
//! - **`ShapeType` enum 用于 FFI / 派发表**：vtable 分派开销可接受（MVP 不在热路径上对 trait object 做高频分派；
//!   真正的 hot path 由后续 `gvpe-collision` 直接特化）。

use std::fmt::Debug;
use std::sync::Arc;

use gvpe_math::Aabb;

/// 形状类型标签。
///
/// 用于：
/// - FFI 边界（`#[repr(u8)]` 后可直接 `transmute`）
/// - 派发表（`gvpe-collision::narrow_phase` 按 `(a, b)` 形状组合选择算法）
/// - `Debug` 输出与日志
///
/// **与 `gvpe-core::ShapeDesc` 关系**：`ShapeDesc` 是描述符层（per-body 拷贝值，存于场景），
/// `ShapeType` 是资产层（per-shape 元数据）。两者不强制一一对应——一个 `ShapeType::Sphere`
/// 资产可被多个 `BodySpec` 引用。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ShapeType {
    /// 球。
    Sphere = 0,
    /// 盒。
    Box3 = 1,
    /// 胶囊。
    Capsule = 2,
    /// 无限平面（半空间）。
    Plane = 3,
    /// 凸包。
    ConvexHull = 4,
}

/// 形状抽象。
///
/// 实现者必须可 `Send + Sync`（多线程 broad phase 共享）+ `Debug`（日志与 assert）。
pub trait Shape: Debug + Send + Sync {
    /// 类型 tag。
    fn shape_type(&self) -> ShapeType;

    /// **局部坐标** AABB（不施加任何 transform）。
    ///
    /// 用途：broad phase 缓存与快速剔除。
    /// **世界坐标 AABB** 应由 gvpe-collision / gvpe-core 在拿到 `Transform` 后计算。
    ///
    /// 注：实现应**缓存**计算结果（构造时一次性算好），不要每次调用都重新计算——
    /// AABB 会被 broad phase 频繁读取（每帧每 body 至少一次）。
    fn local_aabb(&self) -> Aabb;
}

/// 共享形状句柄。
///
/// 内部为 `Arc<dyn Shape>`，克隆廉价（原子 refcount bump），允许多 body 共享同一形状资产。
/// 典型用法：
///
/// ```ignore
/// let sphere = ShapeHandle::new(Sphere { radius: 0.5 });
/// let sphere_clone = sphere.clone();  // 资产共享，不是数据拷贝
/// assert!(Arc::ptr_eq(&sphere.0, &sphere_clone.0));
/// ```
pub struct ShapeHandle(pub(crate) Arc<dyn Shape>);

impl ShapeHandle {
    /// 从值构造（包装为 `Arc<dyn Shape>`）。
    #[inline]
    pub fn new<S: Shape + 'static>(shape: S) -> Self {
        Self(Arc::new(shape))
    }

    /// 从已存在的 `Arc<S>` 构造（用于把已共享的资产包成 `ShapeHandle`）。
    ///
    /// 典型场景：assimp / glTF 加载器一次性解析顶点后 `Arc::new(ConvexHull { ... })`，
    /// 多个 `ShapeHandle` 共享此资产。
    #[inline]
    pub fn from_arc<S: Shape + 'static>(arc: Arc<S>) -> Self {
        Self(arc)
    }

    /// 取引用为 trait object。
    #[inline]
    pub fn as_shape(&self) -> &dyn Shape {
        &*self.0
    }

    /// 内部 `Arc` 强引用计数。
    ///
    /// 主要用于测试 / 调试（断言 "多 body 共享同一形状资产"）。
    /// 不要用于业务逻辑：强引用计数受 weak ref / 显式 drop 影响，不可作为语义保证。
    #[inline]
    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.0)
    }
}

impl Clone for ShapeHandle {
    #[inline]
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl Debug for ShapeHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShapeHandle")
            .field("shape_type", &self.0.shape_type())
            .field("strong_count", &Arc::strong_count(&self.0))
            .finish()
    }
}
