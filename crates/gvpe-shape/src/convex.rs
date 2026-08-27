//! [`ConvexHull`]：凸包。
//!
//! ## 资产式分配
//!
//! `points` 字段为 `Arc<[Vec3]>`——多 body 可共享同一组顶点（参考 20 号文書 §6.1
//! "Arc 后端的资产不在热路径上 clone" 分配策略）。
//! 典型场景：assimp / glTF 加载器解析一次后 `Arc::new(...)`，所有引用此 mesh 的 body
//! 共享顶点资产。
//!
//! ## 构造时凸性校验
//!
//! [`ConvexHull::new`] 调用 `validate_convex` 校验输入点集是否真的"凸"——MVP 实现：
//! 点数 < 4（3 = 三角形，2 = 线段，1 = 点，0 = 退化）一律拒收，返回 [`ConvexError::TooFewPoints`]。
//! **真正的凸性校验**（半空间交集、QHull 算法等）超出 MVP 范围——见 `GVPE-DOC-21` §已知缺口。
//!
//! ## 与 GJK 关系
//!
//! `ConvexHull` 是 GJK 的主要消费方（见 20 号文書 §20.2）；本 crate 不实现 GJK，
//! 仅暴露资产（`points`）让 gvpe-collision 自行遍历。

use std::sync::Arc;

use gvpe_math::{Aabb, Vec3};
use thiserror::Error;

use crate::shape::{Shape, ShapeType};

/// 凸包构造错误。
#[derive(Debug, Error, PartialEq)]
pub enum ConvexError {
    /// 点数 < 4（凸包最少需要 4 个非共面点）。
    #[error("ConvexHull 点数不足: {0} < 4")]
    TooFewPoints(usize),
}

/// 凸包。
///
/// 持有 `Arc<[Vec3]>` 共享资产（参考 20 号文書 §6.1）。
#[derive(Clone, Debug, PartialEq)]
pub struct ConvexHull {
    /// 顶点集合（共享 / 引用计数）。
    pub points: Arc<[Vec3]>,
}

impl ConvexHull {
    /// 从 `Arc<[Vec3]>` 构造。
    ///
    /// 校验点数 ≥ 4（凸包最小条件）；MVP 不做完整凸性校验。
    /// 失败返回 [`ConvexError::TooFewPoints`]。
    pub fn new(points: Arc<[Vec3]>) -> Result<Self, ConvexError> {
        if points.len() < 4 {
            return Err(ConvexError::TooFewPoints(points.len()));
        }
        Ok(Self { points })
    }

    /// 顶点数。
    #[inline]
    #[must_use]
    pub fn num_points(&self) -> usize {
        self.points.len()
    }
}

impl Shape for ConvexHull {
    #[inline]
    fn shape_type(&self) -> ShapeType {
        ShapeType::ConvexHull
    }

    #[inline]
    fn local_aabb(&self) -> Aabb {
        // 调用 `Aabb::from_points` —— 已知 len >= 4 >= 1，所以必定 Some。
        Aabb::from_points(&self.points).expect("ConvexHull::new 已保证 points.len() >= 4 >= 1")
    }
}
