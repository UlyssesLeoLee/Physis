//! `gvpe-collision` 集成测试。
//!
//! ## 测试矩阵（v0.7）
//!
//! | # | 名称 | 类别 | 覆盖 |
//! |---|---|---|---|
//! | 1 | `sap_empty` | broad | 空输入 → 空输出 |
//! | 2 | `sap_single_body` | broad | 单 AABB → 空输出 |
//! | 3 | `sap_two_overlap` | broad | 2 AABB 重叠 → 1 对 |
//! | 4 | `sap_two_separate` | broad | 2 AABB 分离 → 空输出 |
//! | 5 | `sap_many_bodies_mixed` | broad | 多 AABB，部分重叠 |
//! | 6 | `sap_no_overlap_among_many` | broad | 多 AABB 全分离 |
//! | 7 | `gjk_sphere_sphere_separated_known_distance` | narrow | 球-球距离 = 5.0 |
//! | 8 | `gjk_sphere_sphere_overlap` | narrow | 球-球重叠 → Intersect |
//! | 9 | `gjk_box_box_separated` | narrow | OBB-OBB 分离 + 距离 |
//! | 10 | `gjk_box_box_overlap` | narrow | OBB-OBB 重叠 → Intersect |
//! | 11 | `gjk_sphere_box_mixed` | narrow | 球-盒距离 / 重叠 |
//! | 12 | `gjk_convex_hull_distance` | narrow | 凸包-凸包距离 |
//! | 13 | `epa_sphere_sphere_penetration` | epa | 球-球穿透深度 |
//! | 14 | `epa_box_box_penetration` | epa | 盒-盒穿透深度 |
//! | 15 | `epa_sphere_box_penetration` | epa | 球-盒穿透深度 |
//! | 16 | `contact_manifold_single_structure` | manifold | 数据结构字段语义 |
//! | 17 | `contact_manifold_normal_unit_length` | manifold | normal 长度 = 1 |
//! | 18 | `integration_broad_to_narrow_to_manifold` | 端到端 | broad → narrow → EPA → manifold |

use gvpe_core::BodyHandle;
use gvpe_math::{Aabb, Quat, Vec3};

use crate::{
    broad_phase, epa, gjk, ContactManifold, ContactPoint, GjkResult, PenetrationInfo, Shape,
};

// ============================================================================
// helpers
// ============================================================================

/// 容差（浮点比较）。
const EPS: f32 = 1e-3;

/// 球 shape 构造。
fn sphere(c: Vec3, r: f32) -> Shape {
    Shape::Sphere {
        center: c,
        radius: r,
    }
}

/// OBB shape 构造（轴对齐简化为无旋转）。
fn aabb_box(c: Vec3, half: Vec3) -> Shape {
    Shape::Box {
        center: c,
        half_extents: half,
        rotation: Quat::IDENTITY,
    }
}

/// 凸包 shape 构造（4 顶点四面体）。
fn tetra(center: Vec3, size: f32) -> Shape {
    Shape::ConvexHull {
        points: vec![
            center + Vec3::new(size, 0.0, 0.0),
            center + Vec3::new(0.0, size, 0.0),
            center + Vec3::new(0.0, 0.0, size),
            center + Vec3::new(-size, -size, -size),
        ],
    }
}

/// 单点 contact point 断言（保留供未来多点 manifold 测试使用）。
#[allow(dead_code)]
fn assert_contact_point_close(actual: &ContactPoint, expected: &ContactPoint) {
    let pos_diff = (actual.position - expected.position).length();
    let n_diff = (actual.normal - expected.normal).length();
    assert!(
        pos_diff < EPS,
        "position mismatch: actual={:?} expected={:?} (diff={})",
        actual.position,
        expected.position,
        pos_diff
    );
    assert!(
        n_diff < EPS,
        "normal mismatch: actual={:?} expected={:?} (diff={})",
        actual.normal,
        expected.normal,
        n_diff
    );
    assert!(
        (actual.penetration - expected.penetration).abs() < EPS,
        "penetration mismatch: actual={} expected={}",
        actual.penetration,
        expected.penetration
    );
}

// ============================================================================
// Broad phase (SAP) — 6 tests
// ============================================================================

#[test]
fn sap_empty() {
    let pairs = broad_phase(&[]);
    assert!(pairs.is_empty());
}

#[test]
fn sap_single_body() {
    let bodies = [Aabb::from_point(Vec3::ZERO)];
    let pairs = broad_phase(&bodies);
    assert!(pairs.is_empty());
}

#[test]
fn sap_two_overlap() {
    let bodies = [
        Aabb::new(Vec3::ZERO, Vec3::splat(2.0)),
        Aabb::new(Vec3::new(1.0, 0.0, 0.0), Vec3::splat(3.0)),
    ];
    let mut pairs = broad_phase(&bodies);
    pairs.sort_unstable();
    assert_eq!(pairs, vec![(0, 1)]);
}

#[test]
fn sap_two_separate() {
    let bodies = [
        Aabb::new(Vec3::ZERO, Vec3::splat(1.0)),
        Aabb::new(Vec3::splat(10.0), Vec3::splat(11.0)),
    ];
    let pairs = broad_phase(&bodies);
    assert!(pairs.is_empty());
}

#[test]
fn sap_many_bodies_mixed() {
    // 5 物体：0↔1 重叠；2↔3 重叠；4 独立
    let bodies = [
        Aabb::new(Vec3::ZERO, Vec3::splat(2.0)), // 0
        Aabb::new(Vec3::new(1.0, 0.0, 0.0), Vec3::new(3.0, 2.0, 2.0)), // 1
        Aabb::new(Vec3::splat(5.0), Vec3::splat(7.0)), // 2
        Aabb::new(Vec3::new(6.0, 5.0, 5.0), Vec3::new(8.0, 7.0, 7.0)), // 3
        Aabb::new(Vec3::splat(20.0), Vec3::splat(22.0)), // 4
    ];
    let mut pairs = broad_phase(&bodies);
    pairs.sort_unstable();
    assert_eq!(pairs, vec![(0, 1), (2, 3)]);
}

#[test]
fn sap_no_overlap_among_many() {
    let bodies: Vec<Aabb> = (0..10)
        .map(|i| {
            let center = Vec3::splat((i as f32) * 10.0);
            Aabb::new(center, center + Vec3::splat(1.0))
        })
        .collect();
    let pairs = broad_phase(&bodies);
    assert!(pairs.is_empty());
}

// ============================================================================
// Narrow phase (GJK) — 6 tests
// ============================================================================

#[test]
fn gjk_sphere_sphere_separated_known_distance() {
    // 球 1 at (0,0,0) r=1；球 2 at (5,0,0) r=1 → 距离 3
    let a = sphere(Vec3::ZERO, 1.0);
    let b = sphere(Vec3::new(5.0, 0.0, 0.0), 1.0);
    match gjk(&a, &b) {
        GjkResult::Separated {
            distance,
            normal_a_to_b,
        } => {
            assert!((distance - 3.0).abs() < EPS, "distance = {}", distance);
            // normal A → B 应为 +X
            assert!((normal_a_to_b.x - 1.0).abs() < EPS);
            assert!(normal_a_to_b.y.abs() < EPS);
            assert!(normal_a_to_b.z.abs() < EPS);
        }
        GjkResult::Intersect => panic!("expected separated, got intersect"),
    }
}

#[test]
fn gjk_sphere_sphere_overlap() {
    // 中心距 0.5 < r_a + r_b = 2 → 真正 overlap（穿透 1.5）
    let a = sphere(Vec3::ZERO, 1.0);
    let b = sphere(Vec3::new(0.5, 0.0, 0.0), 1.0);
    assert_eq!(gjk(&a, &b), GjkResult::Intersect);
}

#[test]
fn gjk_box_box_separated() {
    // 2 个 1x1x1 box 中心距 3 → 分离 1
    let a = aabb_box(Vec3::ZERO, Vec3::splat(0.5));
    let b = aabb_box(Vec3::new(3.0, 0.0, 0.0), Vec3::splat(0.5));
    match gjk(&a, &b) {
        GjkResult::Separated { distance, .. } => {
            assert!((distance - 2.0).abs() < EPS, "distance = {}", distance);
        }
        GjkResult::Intersect => panic!("expected separated"),
    }
}

#[test]
fn gjk_box_box_overlap() {
    // 中心距 0.5 < 边长 2 → 真正 overlap（穿透 1.5）
    let a = aabb_box(Vec3::ZERO, Vec3::splat(1.0));
    let b = aabb_box(Vec3::new(0.5, 0.0, 0.0), Vec3::splat(1.0));
    assert_eq!(gjk(&a, &b), GjkResult::Intersect);
}

#[test]
fn gjk_sphere_box_mixed() {
    // 球 (0,0,0) r=1 vs 1x1x1 box at (3,0,0) → 距离 1.5
    let a = sphere(Vec3::ZERO, 1.0);
    let b = aabb_box(Vec3::new(3.0, 0.0, 0.0), Vec3::splat(0.5));
    match gjk(&a, &b) {
        GjkResult::Separated {
            distance,
            normal_a_to_b,
        } => {
            assert!((distance - 1.5).abs() < EPS, "distance = {}", distance);
            assert!((normal_a_to_b.x - 1.0).abs() < EPS);
        }
        GjkResult::Intersect => panic!("expected separated"),
    }
}

#[test]
fn gjk_convex_hull_distance() {
    // 2 个 tetra，中心距 2，size=0.5
    // 最近顶点对：(0.5,0,0) vs (1.5,0,0) → 距离 1.0
    let a = tetra(Vec3::ZERO, 0.5);
    let b = tetra(Vec3::splat(2.0), 0.5);
    match gjk(&a, &b) {
        GjkResult::Separated { distance, .. } => {
            // 凸包 A 顶点：(0.5,0,0),(0,0.5,0),(0,0,0.5),(-0.5,-0.5,-0.5)
            // 凸包 B 顶点：(2.5,2,2),(2,2.5,2),(2,2,2.5),(1.5,1.5,1.5)
            // argmin |a_p - b_q| = (0.5,0,0) - (1.5,1.5,1.5) = (-1, -1.5, -1.5)
            //                          |.| = sqrt(1+2.25+2.25) = sqrt(5.5) ≈ 2.345
            assert!(
                (distance - 2.345).abs() < 0.1,
                "distance should be ~2.345, got {}",
                distance
            );
        }
        GjkResult::Intersect => panic!("expected separated"),
    }
}

// ============================================================================
// EPA — 3 tests
// ============================================================================

#[test]
fn epa_sphere_sphere_penetration() {
    // 2 球中心距 1，r=1 each → 穿透 1
    let a = sphere(Vec3::ZERO, 1.0);
    let b = sphere(Vec3::new(1.0, 0.0, 0.0), 1.0);
    let info = epa(&a, &b).expect("EPA should succeed for non-degenerate overlap");
    assert!(
        (info.penetration - 1.0).abs() < 0.1,
        "penetration should be ~1, got {}",
        info.penetration
    );
    assert!(info.penetration > 0.0);
    let n_len = info.normal_a_to_b.length();
    assert!(
        (n_len - 1.0).abs() < 0.01,
        "normal should be unit, got {}",
        n_len
    );
}

#[test]
fn epa_box_box_penetration() {
    // 2 个 2x2x2 box 中心距 1 → 沿 x 方向穿透 1
    let a = aabb_box(Vec3::ZERO, Vec3::splat(1.0));
    let b = aabb_box(Vec3::new(1.0, 0.0, 0.0), Vec3::splat(1.0));
    let info = epa(&a, &b).expect("EPA should succeed");
    assert!(
        info.penetration > 0.5,
        "penetration should be ~1, got {}",
        info.penetration
    );
    let n_len = info.normal_a_to_b.length();
    assert!((n_len - 1.0).abs() < 0.01);
}

#[test]
fn epa_sphere_box_penetration() {
    // 球 r=2 at (0,0,0) vs 1x1x1 box at (1,0,0) → 球心到 box 表面最近点 0.5；穿透 2 - 0.5 = 1.5
    let a = sphere(Vec3::ZERO, 2.0);
    let b = aabb_box(Vec3::new(1.0, 0.0, 0.0), Vec3::splat(0.5));
    let info = epa(&a, &b).expect("EPA should succeed");
    assert!(
        info.penetration > 1.0,
        "penetration should be ~1.5, got {}",
        info.penetration
    );
    assert!(info.penetration < 2.0);
}

// ============================================================================
// Manifold — 2 tests
// ============================================================================

#[test]
fn contact_manifold_single_structure() {
    let a = BodyHandle::new(0, 0);
    let b = BodyHandle::new(1, 0);
    let point = ContactPoint {
        position: Vec3::new(0.0, 0.0, 0.5),
        normal: Vec3::X,
        penetration: 0.5,
    };
    let m = ContactManifold::single(a, b, point);
    assert_eq!(m.body_a, a);
    assert_eq!(m.body_b, b);
    assert_eq!(m.points.len(), 1);
    assert_eq!(m.points[0], point);
}

#[test]
fn contact_manifold_normal_unit_length() {
    // 数据契约：normal 必须是单位长度（per `06_collision_design.md §7.1`）
    let m = ContactManifold::single(
        BodyHandle::new(0, 0),
        BodyHandle::new(1, 0),
        ContactPoint {
            position: Vec3::ZERO,
            normal: Vec3::new(3.0, 4.0, 0.0).normalize(), // 5, 3-4-5
            penetration: 1.0,
        },
    );
    let n_len = m.points[0].normal.length();
    assert!((n_len - 1.0).abs() < 1e-5, "normal length = {}", n_len);
    assert!(m.points[0].penetration > 0.0);
}

// ============================================================================
// 端到端集成测试
// ============================================================================

#[test]
fn integration_broad_to_narrow_to_manifold() {
    // 场景：3 物体
    // - 0 vs 1: AABB 重叠 + sphere 真正 overlap（broad + narrow 都报）
    // - 0 vs 2 / 1 vs 2: 完全分离
    let bodies = [
        Aabb::new(Vec3::ZERO, Vec3::splat(2.0)), // 0: (0,0,0)-(2,2,2)
        Aabb::new(Vec3::splat(1.0), Vec3::splat(3.0)), // 1: (1,1,1)-(3,3,3) 与 0 重叠
        Aabb::new(Vec3::splat(10.0), Vec3::splat(11.0)), // 2: 独立
    ];
    let pairs = broad_phase(&bodies);
    assert_eq!(pairs.len(), 1, "broad phase 应只报 1 对");
    assert_eq!(pairs[0], (0, 1));

    // 用 AABB 中心 + 大半径 sphere（保证 narrow phase 也报 overlap）
    let shapes = [
        sphere(bodies[0].center(), 1.5), // (1,1,1) r=1.5
        sphere(bodies[1].center(), 1.5), // (2,2,2) r=1.5
        sphere(bodies[2].center(), 0.5), // (10.5,10.5,10.5) r=0.5
    ];

    let mut manifolds: Vec<ContactManifold> = Vec::new();
    for (i, j) in pairs {
        let h_i = BodyHandle::new(i, 0);
        let h_j = BodyHandle::new(j, 0);
        match gjk(&shapes[i as usize], &shapes[j as usize]) {
            GjkResult::Separated { distance, .. } => {
                assert!(distance > 0.0);
            }
            GjkResult::Intersect => {
                let info: PenetrationInfo = epa(&shapes[i as usize], &shapes[j as usize])
                    .expect("EPA must succeed for GJK-intersect");
                let point = ContactPoint {
                    position: shapes[i as usize].support(info.normal_a_to_b),
                    normal: info.normal_a_to_b,
                    penetration: info.penetration,
                };
                manifolds.push(ContactManifold::single(h_i, h_j, point));
            }
        }
    }
    assert_eq!(manifolds.len(), 1, "应生成 1 个 manifold（0↔1）");
    let m = &manifolds[0];
    assert_eq!(m.body_a, BodyHandle::new(0, 0));
    assert_eq!(m.body_b, BodyHandle::new(1, 0));
    // 验证 manifold 数据契约
    let p = &m.points[0];
    assert!(p.penetration > 0.0, "penetration 必须正：{}", p.penetration);
    assert!(
        (p.normal.length() - 1.0).abs() < 0.01,
        "normal 应单位长度：len={}",
        p.normal.length()
    );
}
