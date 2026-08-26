//! 接触流形数据结构。
//!
//! 依据 `GVPE-DOC-06`（06_collision_design.md）§7.1：
//! 精筛输出以 [`ContactManifold`] 形式馈入 `gvpe-constraint` 的 `ContactConstraint` 行
//! —— **绝不**作为图 `Constraint` 节点（依据 `02_physics_ontology.md` §9 绑定规则）。
//!
//! ## v0.7 实现范围
//!
//! - [`ContactManifold`] / [`ContactPoint`] 数据结构与 design doc §7.1 一致；
//! - 集成测试验证 GJK + EPA 输出的 manifold 字段语义（`normal` 单位长度、
//!   `penetration >= 0`、point 落在物体表面）。
//! - 多接触点（multi-point manifold）由 EPA + clipping 生成；本 crate v0.7
//!   输出**单点** manifold，多点 manifold 留 v0.8（见 crate 根 `KNOWN_GAPS`）。

use gvpe_core::BodyHandle;
use gvpe_math::Vec3;

/// 单接触点。
///
/// 字段语义（per `06_collision_design.md` §7.1）：
/// - `position`：世界空间接触点位置。
/// - `normal`：世界空间接触法向（从 A 指向 B；**单位长度**）。
/// - `penetration`：穿透深度（**非负**；正值 = 物体重叠量）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContactPoint {
    /// 接触点世界坐标。
    pub position: Vec3,
    /// 接触法向（`a → b` 方向；单位长度）。
    pub normal: Vec3,
    /// 穿透深度（`>= 0`）。
    pub penetration: f32,
}

/// 接触流形。
///
/// 描述一对 body 之间的接触点集合。`points` 使用 [`SmallVec`] 风格的小内联
/// buffer（v0.7 简化为 `Vec`，因 `smallvec` 暂未加入 workspace deps；
/// v0.8+ 改回 SmallVec 以匹配 design doc §7.1 原文）。
#[derive(Clone, Debug, PartialEq)]
pub struct ContactManifold {
    /// 参与接触的 body A。
    pub body_a: BodyHandle,
    /// 参与接触的 body B。
    pub body_b: BodyHandle,
    /// 接触点列表（v0.7 限 1 个；v0.8 扩展至 ≤ 4）。
    pub points: Vec<ContactPoint>,
}

impl ContactManifold {
    /// 构造单点接触流形。
    ///
    /// v0.7 限单点；v0.8 改用 `SmallVec<[ContactPoint; 4]>` 后可暴露
    /// `with_capacity` / `push` 系列 API。
    #[inline]
    #[must_use]
    pub fn single(body_a: BodyHandle, body_b: BodyHandle, point: ContactPoint) -> Self {
        Self {
            body_a,
            body_b,
            points: vec![point],
        }
    }
}
