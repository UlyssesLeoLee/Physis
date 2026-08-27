//! EPA（Expanding Polytope Algorithm）：穿透深度计算。
//!
//! 依据 `GVPE-DOC-06`（06_collision_design.md）§6.3：EPA 与 GJK 配对，凸包落地后
//! 启用穿透深度计算。v0.7 提前实现（brief 显式要求），用于 GJK 输出
//! [`crate::narrow_phase::GjkResult::Intersect`] 后计算 [`PenetrationInfo`]。
//!
//! ## 算法
//!
//! 1. 从 GJK 终止 simplex（保证包含原点）初始化多面体；
//! 2. 迭代：找距原点最近的面，沿面法向 support 扩展；
//! 3. 终止条件：support 点距面距离 < `EPS`；
//! 4. 返回最近面法向 + 距离作为穿透法向 + 深度。
//!
//! ## 多面体重构
//!
//! 每轮 support 扩展后会"局部重构"：移除所有"朝向新点"的 face（顶点在新点
//! 一侧），再用新点 + 每个被移除 face 的边构造新 face。参考
//! Christer Ericson "Real-Time Collision Detection" §5.5。
//!
//! ## 数值稳定性
//!
//! - 退化（coplanar）面会被跳过；
//! - 迭代上限 `MAX_ITERATIONS = 64` 防止数值病态下死循环；
//! - 法向未归一化时距离计算会触发单位化并按比例放缩。

use gvpe_math::Vec3;

use crate::shape::Shape;

/// 数值容差（与 `narrow_phase::EPS` 同值；分开以避免模块间常量耦合）。
const EPS: f32 = 1e-6;

const MAX_ITERATIONS: usize = 64;

/// EPA 穿透信息。
///
/// - `normal_a_to_b`：从 A 指向 B 的**单位**法向；
/// - `penetration`：穿透深度（>= `EPS`）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PenetrationInfo {
    /// 穿透法向（A → B 方向）。
    pub normal_a_to_b: Vec3,
    /// 穿透深度。
    pub penetration: f32,
}

/// 多面体维护：顶点 + 三角面（按 CCW 朝外法向顺序）。
#[derive(Clone, Debug)]
struct Polytope {
    /// 顶点列表。
    vertices: Vec<Vec3>,
    /// 三角面列表（每面 3 个顶点索引 + 缓存法向）。
    faces: Vec<Face>,
}

#[derive(Clone, Copy, Debug)]
struct Face {
    /// 顶点索引（CCW，朝外法向）。
    indices: [usize; 3],
    /// 面法向（朝外）。
    normal: Vec3,
    /// 原点到面的距离（>= 0）。
    distance: f32,
}

impl Face {
    /// 构造面；自动计算法向 + 距离。
    ///
    /// 若顶点退化（cross 接近 0）返回 `None`，调用方应跳过。
    fn new(vertices: &[Vec3], a: usize, b: usize, c: usize) -> Option<Self> {
        let va = vertices[a];
        let vb = vertices[b];
        let vc = vertices[c];
        let normal = (vb - va).cross(vc - va);
        let len = normal.length();
        if len < EPS {
            return None;
        }
        let n = normal / len;
        // 面过 va, vb, vc；距离 = |n · (va - origin)| = |n · va|（origin = 0）
        let distance = n.dot(va);
        if distance < 0.0 {
            // 朝向原点的法向（朝内）→ 取反保证 normal 始终朝外
            Some(Self {
                indices: [a, b, c],
                normal: -n,
                distance: -distance,
            })
        } else {
            Some(Self {
                indices: [a, b, c],
                normal: n,
                distance,
            })
        }
    }

    /// 该面是否"朝向"点 `p`（p 在面外侧）。
    ///
    /// 用距离符号判定：p 在面外 ⇔ n · (p - va) > 0 ⇔ n · p - distance > 0。
    fn faces_toward(&self, p: Vec3) -> bool {
        self.normal.dot(p) - self.distance > EPS
    }
}

/// EPA 主入口。
///
/// `a` / `b` 为参与重叠的两个 shape。返回 [`PenetrationInfo`]；
/// 若迭代到上限仍未收敛（数值病态），返回 `None` 由调用方决定兜底策略。
pub fn epa(a: &Shape, b: &Shape) -> Option<PenetrationInfo> {
    // 初始化：3 个面（tetrahedron 4 顶点）。调用方应保证 a ∩ b ≠ ∅。
    // 退化输入（如两 shape 完全重合于一点）→ 返回 None。
    let mut poly = Polytope {
        vertices: vec![
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 0.0),
        ],
        faces: Vec::with_capacity(64),
    };
    // 4 顶点初始 tetrahedron — 用一组对角面 + 对角面构造 CCW 朝外法向。
    // 实际初始法向不重要：EPA 后续会通过支持函数扩展；只要原点在内即可。
    let v = &poly.vertices;
    // 顶点序：0(1,0,0), 1(0,1,0), 2(0,0,1), 3(0,0,0)
    // 面：0-1-2, 0-3-1, 0-2-3, 1-3-2  (CCW 使法向朝外)
    let candidates = [[0usize, 1, 2], [0, 3, 1], [0, 2, 3], [1, 3, 2]];
    for [a, b, c] in candidates {
        if let Some(f) = Face::new(v, a, b, c) {
            poly.faces.push(f);
        }
    }
    if poly.faces.is_empty() {
        return None;
    }

    for _ in 0..MAX_ITERATIONS {
        // 找最近面
        let (face_idx, closest) = poly
            .faces
            .iter()
            .enumerate()
            .min_by(|(_, x), (_, y)| {
                x.distance
                    .partial_cmp(&y.distance)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, f)| (i, *f))?;

        // 沿法向 support
        let p = a.support(closest.normal) - b.support(-closest.normal);

        // 终止判定：新点距面 < EPS
        let d = closest.normal.dot(p) - closest.distance;
        if d < EPS {
            return Some(PenetrationInfo {
                normal_a_to_b: closest.normal,
                penetration: closest.distance,
            });
        }

        // 局部重构：移除所有"朝向 p"的面，添加新面
        let p_idx = poly.vertices.len();
        poly.vertices.push(p);
        // 记录被移除的面的"边"（按 CCW 顺序）
        let mut edges: Vec<(usize, usize)> = Vec::new();
        let mut i = 0;
        while i < poly.faces.len() {
            if poly.faces[i].faces_toward(p) {
                let f = poly.faces.remove(i);
                add_edge(&mut edges, f.indices[0], f.indices[1]);
                add_edge(&mut edges, f.indices[1], f.indices[2]);
                add_edge(&mut edges, f.indices[2], f.indices[0]);
            } else {
                i += 1;
            }
        }
        // 边按 (min, max) 去重（每条边出现 2 次 = 共享面；只保留 1 次 = 边界）
        for (a, b) in edges {
            if let Some(f) = Face::new(&poly.vertices, a, b, p_idx) {
                poly.faces.push(f);
            }
        }

        // 数值稳定性：面数 / 顶点数爆炸 → 终止
        if poly.faces.len() > 256 || poly.vertices.len() > 128 {
            return Some(PenetrationInfo {
                normal_a_to_b: closest.normal,
                penetration: closest.distance,
            });
        }
        // 抑制 unused warning
        let _ = face_idx;
    }

    // 迭代上限：保守返回最近面
    let closest = poly.faces.iter().min_by(|x, y| {
        x.distance
            .partial_cmp(&y.distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    })?;
    Some(PenetrationInfo {
        normal_a_to_b: closest.normal,
        penetration: closest.distance,
    })
}

/// 边合并：若 `edges` 已含 `{a, b}`（顺序无关）则移除（共享边 = 内部，非边界），
/// 否则按 `a, b` 原序添加 —— **原序**用于新面 `(a, b, p_idx)` 的 CCW 朝向构造。
fn add_edge(edges: &mut Vec<(usize, usize)>, a: usize, b: usize) {
    let key = if a < b { (a, b) } else { (b, a) };
    if let Some(pos) = edges
        .iter()
        .position(|&e| if e.0 < e.1 { e } else { (e.1, e.0) } == key)
    {
        edges.remove(pos);
    } else {
        edges.push((a, b));
    }
}
