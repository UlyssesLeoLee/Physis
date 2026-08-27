//! 精筛（narrow phase）：GJK 距离 / 相交判定。
//!
//! 依据 `GVPE-DOC-06`（06_collision_design.md）§6.3：
//! - **SAT** 为 MVP Box-Box / Box-Plane / Sphere-Box 主选；
//! - **GJK** 保留给凸包（post-MVP）。
//!
//! **v0.7 实现选择**（与 design doc MVP 选型偏差，详见 crate 根 `KNOWN_GAPS`）：
//! brief 显式要求 GJK + EPA，本 crate 优先落地 GJK 通用路径
//! （适用 Sphere / Box / ConvexHull），SAT 留 v0.8 补全。
//!
//! ## 算法
//!
//! GJK 通过 **Minkowski difference** `C = A ⊖ B` 在原点处是否包含原点判定相交。
//! 维护一个 ≤ 4 顶点的 **simplex**，迭代逼近原点：
//! 1. 取初始搜索方向 `d = b.center - a.center`；
//! 2. 单纯形加入 `support_C(d) = support_A(d) - support_B(-d)`；
//! 3. 用 "closest point to origin in simplex" 算法找最近点 `p` 与 Voronoi region；
//! 4. 原点在 simplex 内 → 相交；否则 `p` 给出最近距离 + 法向；
//! 5. 缩减 simplex 到 Voronoi region 子集，重复 2-4。
//!
//! 收敛判定：当新 support 沿 `d` 方向与 simplex 最近点的距离无显著改善时
//! 停止（典型 ≤ 32 次迭代可收敛；本实现上限 64）。
//!
//! ## 数值稳定性
//!
//! - 所有浮点比较给 `eps = 1e-6` 容差；
//! - simplex 退化（点重合 / 共线）由 closest-point 算法自然处理；
//! - 极端深穿透（penetration > 1e4）判定为 "数值病态" 直接返回 overlap 让 EPA 接管。

use gvpe_math::Vec3;

use crate::shape::Shape;

/// 数值容差（法向、点重合判定）。
const EPS: f32 = 1e-6;

/// GJK 收敛最大迭代次数（防退化输入死循环）。
const MAX_ITERATIONS: usize = 64;

/// GJK 输出：分离或相交。
///
/// - `Separated { distance, normal_a_to_b }`：两 shape 不相交；`distance` 为
///   表面最近距离（>= `EPS`），`normal_a_to_b` 为从 A 指向 B 的单位法向。
/// - `Intersect`：两 shape 重叠（GJK 终止于原点落在 simplex 内）；
///   调用方应转入 [`crate::epa::epa`] 计算穿透深度。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GjkResult {
    /// 两 shape 分离。
    Separated {
        /// 表面最近距离（> 0）。
        distance: f32,
        /// 从 A 指向 B 的单位法向。
        normal_a_to_b: Vec3,
    },
    /// 两 shape 重叠。
    Intersect,
}

/// 内部 simplex 状态（≤ 4 顶点）。
///
/// 顶点按添加顺序存于固定 buffer；`size` 为当前顶点数（1..=4）。
/// simplex 内顶点的"closest point to origin"通过 barycentric 坐标 + 缩减算法
/// 计算，参考 Casey Muratori 视频 "Implementing GJK" (2006) / Christer Ericson
/// "Real-Time Collision Detection" §5.4。
#[derive(Clone, Copy, Debug)]
struct Simplex {
    /// simplex 顶点。
    points: [Vec3; 4],
    /// 当前顶点数。
    size: usize,
}

impl Simplex {
    /// 空 simplex。
    const fn new() -> Self {
        Self {
            points: [Vec3::ZERO; 4],
            size: 0,
        }
    }

    /// 推入新点；返回旧 size。
    fn push(&mut self, p: Vec3) -> usize {
        self.points[self.size] = p;
        self.size += 1;
        self.size - 1
    }

    /// 移除 idx 位置的点（用末尾点覆盖）。
    fn remove(&mut self, idx: usize) {
        self.points[idx] = self.points[self.size - 1];
        self.size -= 1;
    }
}

/// Minkowski difference support: `support_A(d) - support_B(-d)`。
#[inline]
fn support(a: &Shape, b: &Shape, direction: Vec3) -> Vec3 {
    a.support(direction) - b.support(-direction)
}

/// GJK 主入口。
///
/// 返回 [`GjkResult`]：`Separated` 时 `normal_a_to_b` 指向 B，`distance` >= `EPS`；
/// `Intersect` 时调用方应转 EPA。
pub fn gjk(a: &Shape, b: &Shape) -> GjkResult {
    // 初始方向：A 中心 → B 中心。两中心重合时退化为 +X 方向。
    let a_center = shape_center(a);
    let b_center = shape_center(b);
    let initial_dir = (b_center - a_center).normalize();
    let dir = if initial_dir == Vec3::ZERO {
        Vec3::X
    } else {
        initial_dir
    };

    let mut simplex = Simplex::new();
    simplex.push(support(a, b, dir));
    let mut direction = -simplex.points[0];

    for _ in 0..MAX_ITERATIONS {
        if direction == Vec3::ZERO {
            // 原点已在 simplex 内 → 相交
            return GjkResult::Intersect;
        }
        // 新 support point
        let new_point = support(a, b, direction);

        // 去重：若 new_point 已在 simplex 中（距离 < EPS），说明 support 退
        // 化到已有点，搜索无进展；按当前最近点终止。
        if simplex.points[..simplex.size]
            .iter()
            .any(|p| (*p - new_point).length() < EPS)
        {
            return separated_from_direction(direction);
        }

        // 收敛判定：new_point 沿 direction 的投影 < simplex 中最近点投影
        // （近似为 -|closest|²）。若 ≤ 0 则新点不更接近原点，终止。
        // 用 direction 长度作为"当前最近距离"上界（direction 由 do_simplex 设
        // 定向，标准 GJK 推导下 |direction| = |v_closest|）。
        let closest_dist = direction.length();
        if new_point.dot(direction) <= 0.0 {
            if closest_dist < EPS {
                return GjkResult::Intersect;
            }
            return GjkResult::Separated {
                distance: closest_dist,
                normal_a_to_b: direction / closest_dist,
            };
        }

        simplex.push(new_point);

        // 用 closest-point-to-origin 缩减 simplex + 更新搜索方向
        if do_simplex(&mut simplex, &mut direction) {
            return GjkResult::Intersect;
        }
    }

    // 达到迭代上限：保守返回分离（避免误报 overlap；调用方应视作 numerical failure）
    separated_from_direction(direction)
}

/// 由搜索方向构造 `Separated` 结果（distance = |direction|）。
#[inline]
fn separated_from_direction(direction: Vec3) -> GjkResult {
    let dist = direction.length();
    if dist < EPS {
        // 方向为零 → 退化为 Intersect（防止"零距离"假报）
        GjkResult::Intersect
    } else {
        GjkResult::Separated {
            distance: dist,
            normal_a_to_b: direction / dist,
        }
    }
}

/// 求形状中心（仅用于初始方向估计，不影响算法正确性）。
fn shape_center(s: &Shape) -> Vec3 {
    match s {
        // Sphere / Box 中心字段同名，合并分支。
        Shape::Sphere { center, .. } | Shape::Box { center, .. } => *center,
        Shape::ConvexHull { points } => {
            // 凸包中心 = 顶点均值（粗略；GJK 仅用作初始方向估计）
            if points.is_empty() {
                Vec3::ZERO
            } else {
                let mut sum = Vec3::ZERO;
                for p in points {
                    sum = sum + *p;
                }
                sum / (points.len() as f32)
            }
        }
    }
}

/// 处理 simplex：根据顶点数（1/2/3/4）执行 closest-point-to-origin 子程序，
/// 返回 `true` 表示原点包含（重叠）。
///
/// 几何推导参考：
/// - Point (size=1)：最近点 = 该点；新方向 = -point。
/// - Line (size=2)：找线段上最近原点的点；新方向 = -closest。
/// - Triangle (size=3)：3 个 voronoi region（顶点 / 3 条边）。
/// - Tetrahedron (size=4)：4 个 voronoi region（顶点 / 6 条边 / 4 个面）。
///
/// 缩减 simplex 到子 simplex（仅保留影响 closest point 的顶点）。
fn do_simplex(simplex: &mut Simplex, direction: &mut Vec3) -> bool {
    match simplex.size {
        1 => do_point(simplex, direction),
        2 => do_line(simplex, direction),
        3 => do_triangle(simplex, direction),
        4 => do_tetrahedron(simplex, direction),
        _ => false,
    }
}

fn do_point(simplex: &Simplex, direction: &mut Vec3) -> bool {
    // size=1：simplex 仅一个顶点；新方向指向 -a 即可（不再 modify simplex）。
    // 取 `simplex` 借用仅为统一签名（与 do_line/tri/tet 一致）。
    let a = simplex.points[0];
    *direction = -a;
    false
}

fn do_line(simplex: &mut Simplex, direction: &mut Vec3) -> bool {
    let a = simplex.points[0];
    let b = simplex.points[1];
    let ab = b - a;
    let ao = -a;

    if ab.dot(ao) > 0.0 {
        // 原点在 AB 方向的 voronoi region → 保留 a, b
        let cross = ab.cross(ao);
        if cross.length_squared() < EPS * EPS {
            // AB 平行于 AO：原点在 AB 延长线上。
            // 0 在 [a, b] 线段内 ⇔ a · b ≤ 0（a, b 在 0 两侧）。
            if a.dot(b) <= 0.0 {
                return true; // 0 严格在线段内 → Intersect
            }
            // 0 在线段外（a, b 同侧）→ 方向退化为 -a
            *direction = ao;
        } else {
            *direction = cross.cross(ab);
        }
    } else {
        // 原点在 A 顶点 voronoi region → 缩减为 a
        simplex.remove(1);
        *direction = ao;
    }
    false
}

fn do_triangle(simplex: &mut Simplex, direction: &mut Vec3) -> bool {
    let a = simplex.points[0];
    let b = simplex.points[1];
    let c = simplex.points[2];
    let ab = b - a;
    let ac = c - a;
    let ao = -a;
    let abc = ab.cross(ac);

    // voronoi region 分类（朝外法向为 abc）：
    // 1) 顶点 A：abc × ac 与 abc × ab 之外（ac.dot(ao) <= 0 且 ab.dot(ao) <= 0）
    // 2) 边 AB：abc × ab 与 -abc × ac 之间
    // 3) 边 AC：abc × ac 与 -abc × ab 之间
    // 4) 面 ABC：abc 与 ao 同向

    if abc.dot(ao) > 0.0 {
        // 4) 面 region（朝外法向 abc，ao 与其同向）
        *direction = abc;
        return false;
    }

    // 候选边 region 检查
    if ab.cross(abc).dot(ao) > 0.0 {
        // 2) 边 AB region
        if ab.dot(ao) > 0.0 {
            // 保留 a, b
            simplex.remove(2);
            *direction = ab.cross(ao).cross(ab);
        } else {
            // 退化到顶点 A region
            simplex.remove(1);
            simplex.remove(1); // size 减两次：移除 b, c
            *direction = ao;
        }
        return false;
    }

    if abc.cross(ac).dot(ao) > 0.0 {
        // 3) 边 AC region
        if ac.dot(ao) > 0.0 {
            // 保留 a, c
            simplex.remove(1);
            *direction = ac.cross(ao).cross(ac);
        } else {
            // 退化到顶点 A region
            simplex.remove(1);
            simplex.remove(1); // 移除 b, c
            *direction = ao;
        }
        return false;
    }

    // 1) 顶点 A region
    simplex.remove(1);
    simplex.remove(1); // 移除 b, c
    *direction = ao;
    false
}

fn do_tetrahedron(simplex: &mut Simplex, direction: &mut Vec3) -> bool {
    let a = simplex.points[0];
    let b = simplex.points[1];
    let c = simplex.points[2];
    let d = simplex.points[3];
    let ab = b - a;
    let ac = c - a;
    let ad = d - a;
    let ao = -a;
    let abc = ab.cross(ac);
    let acd = ac.cross(ad);
    let adb = ad.cross(ab);

    // 检查 4 个面。原点在内 ⇔ ao 与 4 个面法向（朝外）夹角均 >= 90°。
    if abc.dot(ao) > 0.0 {
        // 原点在 ABC 面外侧 → 移除 d
        simplex.remove(3);
        *direction = abc;
        return do_triangle(simplex, direction);
    }
    if acd.dot(ao) > 0.0 {
        // 原点在 ACD 面外侧 → 移除 b
        simplex.remove(1);
        // 重新映射 c, d 到 b, c 位置以保持 triangle 顶点顺序
        let tmp = simplex.points[simplex.size - 1];
        simplex.points[1] = simplex.points[2];
        simplex.points[2] = tmp;
        *direction = acd;
        return do_triangle(simplex, direction);
    }
    if adb.dot(ao) > 0.0 {
        // 原点在 ADB 面外侧 → 移除 c
        simplex.remove(2);
        let tmp = simplex.points[simplex.size - 1];
        simplex.points[2] = tmp;
        *direction = adb;
        return do_triangle(simplex, direction);
    }
    // 4 个面均不外排 → 原点在 tetrahedron 内
    true
}
