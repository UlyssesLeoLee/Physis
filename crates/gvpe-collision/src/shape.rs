//! 形状定义 + GJK support 函数。
//!
//! 依据 `GVPE-DOC-06`（06_collision_design.md）§6.2 MVP 选型：
//! **Sphere**、**Box** 为 MVP 必选；**Plane** 因非闭合（无穿透）GJK 不适用，
//! 落入 post-MVP（详见 crate 根 `KNOWN_GAPS`）；**Convex Hull** 在本 crate 同步落地为
//! "point cloud → 隐式凸包"的最小实现，便于 GJK 路径串联。
//!
//! ## 与 `gvpe-core` 的边界
//!
//! 本 crate 不依赖 `gvpe-core::ShapeDesc` —— `gvpe-core` 的 `ShapeDesc` 仅含
//! Sphere / Box / Plane 三种且无变换信息；本 crate 需要 OBB 的旋转与凸包点云，
//! 维护独立的 [`Shape`] 枚举。v0.8+ 在 `gvpe-shape` 落地后做适配器桥接（见
//! crate 根 TODO）。

use gvpe_math::{Quat, Vec3};

/// 形状类别。
///
/// `gvpe-collision` 自有形状 —— 跟 `gvpe-core::ShapeDesc`（仅含 radius / half_extents）
/// 不同，本枚举含**世界空间位置 + 朝向**，可直接喂 GJK support 函数。
#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    /// 球。
    Sphere {
        /// 球心（世界坐标）。
        center: Vec3,
        /// 半径。
        radius: f32,
    },
    /// 定向盒（OBB）。
    Box {
        /// 中心（世界坐标）。
        center: Vec3,
        /// 半尺寸（局部坐标，未旋转）。
        half_extents: Vec3,
        /// 旋转（局部 → 世界）。
        rotation: Quat,
    },
    /// 凸包（点云隐式凸包）。
    ///
    /// **MVP 约束**：调用方需保证点云已**凸且非退化**（至少 4 个不共面点）；
    /// GJK 不会做凸包重建，只取 `argmax point · direction`。
    /// 输入非凸点云时返回**外接凸包**的支撑点（行为定义，但非设计目标）。
    ConvexHull {
        /// 凸包顶点（世界坐标）。
        points: Vec<Vec3>,
    },
}

impl Shape {
    /// GJK 支撑函数：返回 shape 在世界空间 `direction` 方向上的最远点。
    ///
    /// **约定**：`direction` 为零向量时行为未定义（返回实现任选一点），
    /// 调用方须保证 `direction` 非零（GJK / EPA 内部均满足此约束）。
    #[inline]
    #[must_use]
    pub fn support(&self, direction: Vec3) -> Vec3 {
        match self {
            Self::Sphere { center, radius } => *center + direction.normalize() * (*radius),
            Self::Box {
                center,
                half_extents,
                rotation,
            } => {
                // 局部 space 取 sign(d_local) * half_extents；再经旋转 + 平移到 world。
                //
                // **不能用 `f32::signum`** —— 它对 `0.0` 返 `1.0`（IEEE 754 / Rust 语义），
                // 会让 y/z 分量 0 输入时偏移 +0.5 * half_extents，污染 GJK 最近点。
                let d_local = rotation.conjugate().rotate_vec3(direction);
                let local = Vec3::new(
                    sgn(d_local.x) * half_extents.x,
                    sgn(d_local.y) * half_extents.y,
                    sgn(d_local.z) * half_extents.z,
                );
                rotation.rotate_vec3(local) + *center
            }
            Self::ConvexHull { points } => {
                // 点云 argmax dot(point, direction)。空点云返原点（防御性；GJK 会因 simplex
                // 退化终止，调用方需保证至少 1 点）。
                let mut best = Vec3::ZERO;
                let mut best_dot = f32::NEG_INFINITY;
                for &p in points {
                    let d = p.dot(direction);
                    if d > best_dot {
                        best_dot = d;
                        best = p;
                    }
                }
                best
            }
        }
    }
}

/// 严格三分量符号：`> 0 → 1.0`，`< 0 → -1.0`，`== 0 → 0.0`。
///
/// 区别于 [`f32::signum`]（对 `0.0` 返 `1.0`）；GJK Box support 需要
/// "0 输入返 0 输出" 语义。
#[inline]
#[must_use]
const fn sgn(v: f32) -> f32 {
    if v > 0.0 {
        1.0
    } else if v < 0.0 {
        -1.0
    } else {
        0.0
    }
}
