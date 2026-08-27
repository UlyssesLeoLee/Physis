//! `gvpe-shape` 单元测试 + 集成测试。
//!
//! 测试覆盖：
//! - 5 种形状的局部 AABB 正确性
//! - `ShapeHandle` 的 Arc 共享语义（refcount + 资产共享）
//! - 凸包点数校验
//! - `ShapeType` tag 正确性

use std::sync::Arc;

use gvpe_math::Vec3;

use crate::{
    Box3, Capsule, ConvexError, ConvexHull, Plane, Shape, ShapeHandle, ShapeType, Sphere,
};

// ============================================================================
// 局部 AABB 正确性
// ============================================================================

#[test]
fn sphere_local_aabb_symmetric() {
    let s = Sphere { radius: 2.5 };
    let aabb = s.local_aabb();
    assert_eq!(aabb.min, Vec3::new(-2.5, -2.5, -2.5));
    assert_eq!(aabb.max, Vec3::new(2.5, 2.5, 2.5));
    // 球心在原点
    let c = aabb.center();
    assert_eq!(c, Vec3::ZERO);
}

#[test]
fn box3_local_aabb_matches_half_extents() {
    let b = Box3::new([1.0, 2.0, 3.0]);
    let aabb = b.local_aabb();
    assert_eq!(aabb.min, Vec3::new(-1.0, -2.0, -3.0));
    assert_eq!(aabb.max, Vec3::new(1.0, 2.0, 3.0));
    // cube 构造器
    let c = Box3::cube(0.5);
    #[allow(clippy::float_cmp)] // 0.5 在 f32 精确表示(2^-1),逐位比较无误差
    {
        assert_eq!(c.half_extents, [0.5, 0.5, 0.5]);
        assert_eq!(c.local_aabb().min, Vec3::splat(-0.5));
    }
}

#[test]
fn capsule_local_aabb_extends_along_y() {
    // radius=0.5, half_height=1.0：圆柱段 [-1, +1]，半球各延伸 0.5 → 总范围 [-1.5, +1.5]
    let c = Capsule::new(0.5, 1.0);
    let aabb = c.local_aabb();
    assert_eq!(aabb.min, Vec3::new(-0.5, -1.5, -0.5));
    assert_eq!(aabb.max, Vec3::new(0.5, 1.5, 0.5));
    // total_length = 2*(half_height + radius) = 2*(1.0 + 0.5) = 3.0
    assert!((c.total_length() - 3.0).abs() < 1e-6);
}

#[test]
fn plane_local_aabb_is_conservative() {
    let p = Plane::new(Vec3::new(0.0, 1.0, 0.0), 0.0);
    let aabb = p.local_aabb();
    // 平面 AABB 应是保守的"无穷大"立方体——大于任意游戏场景
    let h = 1.0e6;
    assert!(aabb.min.x < -1.0e5 && aabb.max.x > 1.0e5);
    assert!(aabb.min.y < -1.0e5 && aabb.max.y > 1.0e5);
    assert!((aabb.min.x - (-h)).abs() < 1.0);
    assert!((aabb.max.x - h).abs() < 1.0);
}

#[test]
fn convex_hull_local_aabb_from_points() {
    // 立方体 8 顶点：(±1, ±1, ±1)
    let pts: Vec<Vec3> = vec![
        Vec3::new(-1.0, -1.0, -1.0),
        Vec3::new(1.0, -1.0, -1.0),
        Vec3::new(-1.0, 1.0, -1.0),
        Vec3::new(1.0, 1.0, -1.0),
        Vec3::new(-1.0, -1.0, 1.0),
        Vec3::new(1.0, -1.0, 1.0),
        Vec3::new(-1.0, 1.0, 1.0),
        Vec3::new(1.0, 1.0, 1.0),
    ];
    let hull = ConvexHull::new(Arc::from(pts.into_boxed_slice())).expect("8 points OK");
    let aabb = hull.local_aabb();
    assert_eq!(aabb.min, Vec3::new(-1.0, -1.0, -1.0));
    assert_eq!(aabb.max, Vec3::new(1.0, 1.0, 1.0));
    assert_eq!(hull.num_points(), 8);
}

// ============================================================================
// 凸包构造校验
// ============================================================================

#[test]
fn convex_hull_rejects_too_few_points() {
    // 0 / 1 / 2 / 3 个点都应被拒收
    for n in 0..4 {
        let pts: Vec<Vec3> = (0..n).map(|i| Vec3::new(i as f32, 0.0, 0.0)).collect();
        let arc: Arc<[Vec3]> = Arc::from(pts.into_boxed_slice());
        let res = ConvexHull::new(arc);
        assert_eq!(res.err(), Some(ConvexError::TooFewPoints(n)), "n = {n}");
    }

    // 4 个非共面点应通过（仅检查"不报错"——MVP 不做完整凸性校验）
    let pts: Arc<[Vec3]> = Arc::new([
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
    ]);
    assert!(ConvexHull::new(pts).is_ok());
}

// ============================================================================
// ShapeHandle Arc 共享语义
// ============================================================================

#[test]
fn shape_handle_clone_shares_underlying_arc() {
    let s = Sphere { radius: 1.0 };
    let h1 = ShapeHandle::new(s);
    let h2 = h1.clone();
    // strong_count：h1 + h2 = 2
    assert_eq!(h1.strong_count(), 2);
    assert_eq!(h2.strong_count(), 2);
    // as_shape 返回的引用指向同一份 Sphere
    let r1 = h1.as_shape();
    let r2 = h2.as_shape();
    assert_eq!(r1.shape_type(), ShapeType::Sphere);
    assert_eq!(r2.shape_type(), ShapeType::Sphere);
    assert_eq!(r1.local_aabb(), r2.local_aabb());
}

#[test]
fn shape_handle_from_arc_shares_existing_asset() {
    // 模拟加载器场景：assimp 解析一次 → Arc<ConvexHull> → 包成 ShapeHandle
    let pts: Arc<[Vec3]> = Arc::new([
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
    ]);
    let hull = Arc::new(ConvexHull::new(pts).expect("4 points OK"));
    let h1 = ShapeHandle::from_arc(Arc::clone(&hull));
    let h2 = ShapeHandle::from_arc(Arc::clone(&hull));
    // 三个 Arc 引用同一份 ConvexHull：hull(1) + h1(1) + h2(1) = 3 个强引用
    assert_eq!(Arc::strong_count(&hull), 3);
    assert_eq!(h1.strong_count(), 3);
    assert_eq!(h2.strong_count(), 3);
}

#[test]
fn shape_handle_debug_includes_type_and_count() {
    let h = ShapeHandle::new(Box3::cube(0.5));
    let dbg = format!("{h:?}");
    assert!(dbg.contains("ShapeHandle"));
    assert!(dbg.contains("Box3"));
    assert!(dbg.contains("strong_count"));
}

// ============================================================================
// ShapeType tag 一致性
// ============================================================================

#[test]
fn shape_type_tags_match_implementations() {
    let sphere = Sphere { radius: 1.0 };
    assert_eq!(sphere.shape_type(), ShapeType::Sphere);

    let bx = Box3::cube(1.0);
    assert_eq!(bx.shape_type(), ShapeType::Box3);

    let cap = Capsule::new(0.5, 1.0);
    assert_eq!(cap.shape_type(), ShapeType::Capsule);

    let pl = Plane::new(Vec3::Y, 0.0);
    assert_eq!(pl.shape_type(), ShapeType::Plane);

    let pts: Arc<[Vec3]> = Arc::new([
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
    ]);
    let hull = ConvexHull::new(pts).expect("4 points OK");
    assert_eq!(hull.shape_type(), ShapeType::ConvexHull);
}

// ============================================================================
// AABB 形状对 broad phase 兼容性（粗粒度 sanity）
// ============================================================================

#[test]
fn all_shapes_produce_finite_aabb() {
    // 所有形状的 local_aabb 必须有限（非 NaN / 非 INF），否则 broad phase 会出错
    let shapes: Vec<Box<dyn Shape>> = vec![
        Box::new(Sphere { radius: 1.0 }),
        Box::new(Box3::cube(1.0)),
        Box::new(Capsule::new(0.5, 1.0)),
        Box::new(Plane::new(Vec3::Y, 0.0)),
    ];
    for s in &shapes {
        let aabb = s.local_aabb();
        assert!(aabb.min.x.is_finite() && aabb.max.x.is_finite());
        assert!(aabb.min.y.is_finite() && aabb.max.y.is_finite());
        assert!(aabb.min.z.is_finite() && aabb.max.z.is_finite());
    }

    // ConvexHull 单独测
    let pts: Arc<[Vec3]> = Arc::new([
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
    ]);
    let hull = ConvexHull::new(pts).expect("4 points OK");
    let aabb = hull.local_aabb();
    assert!(aabb.min.x.is_finite() && aabb.max.x.is_finite());
}

#[test]
fn all_aabbs_contain_origin() {
    // 除 Plane（保守立方体）外，所有形状局部 AABB 都应包含原点（中心在原点）
    let sphere = Sphere { radius: 1.0 };
    assert!(sphere.local_aabb().contains(Vec3::ZERO));

    let bx = Box3::cube(1.0);
    assert!(bx.local_aabb().contains(Vec3::ZERO));

    let cap = Capsule::new(0.5, 1.0);
    assert!(cap.local_aabb().contains(Vec3::ZERO));

    let pl = Plane::new(Vec3::Y, 0.0);
    assert!(pl.local_aabb().contains(Vec3::ZERO)); // 保守立方体含原点

    let pts: Arc<[Vec3]> = Arc::new([
        Vec3::new(-1.0, -1.0, -1.0),
        Vec3::new(1.0, -1.0, -1.0),
        Vec3::new(-1.0, 1.0, -1.0),
        Vec3::new(1.0, 1.0, -1.0),
        Vec3::new(-1.0, -1.0, 1.0),
        Vec3::new(1.0, -1.0, 1.0),
        Vec3::new(-1.0, 1.0, 1.0),
        Vec3::new(1.0, 1.0, 1.0),
    ]);
    let hull = ConvexHull::new(pts).expect("8 points OK");
    assert!(hull.local_aabb().contains(Vec3::ZERO));
}
