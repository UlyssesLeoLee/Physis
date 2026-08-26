//! `gvpe-math` 单元测试 + 布局验证。

use crate::{Aabb, Mat3, Quat, Transform, Vec3};

// ============================================================================
// Vec3 测试
// ============================================================================

#[test]
fn vec3_size_and_alignment() {
    use std::mem::{align_of, size_of};
    assert_eq!(size_of::<Vec3>(), 12);
    assert_eq!(align_of::<Vec3>(), 4);
}

#[test]
fn vec3_arithmetic() {
    let a = Vec3::new(1.0, 2.0, 3.0);
    let b = Vec3::new(4.0, 5.0, 6.0);

    assert_eq!(a + b, Vec3::new(5.0, 7.0, 9.0));
    assert_eq!(b - a, Vec3::new(3.0, 3.0, 3.0));
    assert_eq!(a * 2.0, Vec3::new(2.0, 4.0, 6.0));
    assert_eq!(-a, Vec3::new(-1.0, -2.0, -3.0));
}

#[test]
fn vec3_dot_and_cross() {
    let a = Vec3::new(1.0, 0.0, 0.0);
    let b = Vec3::new(0.0, 1.0, 0.0);

    assert!(a.dot(b).abs() < 1e-6);
    assert!((a.dot(a) - 1.0).abs() < 1e-6);

    let c = a.cross(b);
    assert_eq!(c, Vec3::new(0.0, 0.0, 1.0));
}

#[test]
fn vec3_normalize() {
    let v = Vec3::new(3.0, 0.0, 4.0);
    let n = v.normalize();
    assert!((n.length() - 1.0).abs() < 1e-6);

    // 零向量归一化返回零
    let zero = Vec3::ZERO.normalize();
    assert_eq!(zero, Vec3::ZERO);
}

// ============================================================================
// Quat 测试
// ============================================================================

#[test]
fn quat_size_and_alignment() {
    use std::mem::{align_of, size_of};
    assert_eq!(size_of::<Quat>(), 16);
    assert_eq!(align_of::<Quat>(), 16, "Quat 应 16 字节对齐以支持 SIMD");
}

#[test]
fn quat_identity_is_unit() {
    let q = Quat::IDENTITY;
    assert!((q.length_squared() - 1.0).abs() < 1e-6);
}

#[test]
fn quat_conjugate() {
    let q = Quat::new(0.1, 0.2, 0.3, 0.4).normalize();
    let c = q.conjugate();
    assert!((c.x - (-q.x)).abs() < 1e-6);
    assert!((c.y - (-q.y)).abs() < 1e-6);
    assert!((c.z - (-q.z)).abs() < 1e-6);
    assert!((c.w - q.w).abs() < 1e-6);
}

#[test]
fn quat_rotate_180_x_axis() {
    // 围绕 X 轴 180 度
    let axis = Vec3::X;
    let q = Quat::from_axis_angle(axis, std::f32::consts::PI);
    let v = Vec3::new(0.0, 1.0, 0.0);
    let r = q.rotate_vec3(v);
    assert!((r.x - 0.0).abs() < 1e-5);
    assert!((r.y - (-1.0)).abs() < 1e-5);
    assert!((r.z - 0.0).abs() < 1e-5);
}

// ============================================================================
// Mat3 测试
// ============================================================================

#[test]
fn mat3_size_and_alignment() {
    use std::mem::{align_of, size_of};
    assert_eq!(size_of::<Mat3>(), 36);
    assert_eq!(align_of::<Mat3>(), 4);
}

#[test]
fn mat3_identity_mul() {
    let m = Mat3::IDENTITY;
    let v = Vec3::new(1.0, 2.0, 3.0);
    assert_eq!(m.mul_vec3(v), v);
}

#[test]
fn mat3_inverse_works() {
    // 简单测试矩阵
    let m = Mat3::new(2.0, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 4.0);
    let inv = m.inverse().unwrap();
    // 逆矩阵 × 原矩阵 = 单位
    let prod = inv.mul_mat3(m);
    assert!((prod.m[0].x - 1.0).abs() < 1e-5);
    assert!((prod.m[1].y - 1.0).abs() < 1e-5);
    assert!((prod.m[2].z - 1.0).abs() < 1e-5);
    assert!(prod.m[0].y.abs() < 1e-5);
    assert!(prod.m[1].x.abs() < 1e-5);
}

#[test]
fn mat3_singular_returns_none() {
    // 奇异矩阵（所有元素相同）
    let m = Mat3::new(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0);
    assert!(m.inverse().is_none());
}

// ============================================================================
// Transform 测试
// ============================================================================

#[test]
fn transform_size_and_alignment() {
    use std::mem::{align_of, size_of};
    // 布局：Vec3(12) + padding(4, 凑齐 16 字节对齐) + Quat(16) = 32 字节。
    // `rotation` 需要 16 字节对齐以支持 SIMD 优化，故必须插入 4 字节 padding。
    assert_eq!(size_of::<Transform>(), 32);
    assert_eq!(
        align_of::<Transform>(),
        16,
        "Transform 应 16 字节对齐以支持 SIMD"
    );
}

#[test]
fn transform_identity() {
    let t = Transform::IDENTITY;
    let v = Vec3::new(1.0, 2.0, 3.0);
    assert_eq!(t.transform_vec3(v), v);
}

#[test]
fn transform_inverse() {
    let t = Transform::new(Vec3::new(10.0, 20.0, 30.0), Quat::IDENTITY);
    let inv = t.inverse();
    let v = Vec3::new(1.0, 2.0, 3.0);
    // 应用 t 然后 inv 应回到原值
    let result = inv.transform_vec3(t.transform_vec3(v));
    assert!((result.x - v.x).abs() < 1e-5);
    assert!((result.y - v.y).abs() < 1e-5);
    assert!((result.z - v.z).abs() < 1e-5);
}

// ============================================================================
// Aabb 测试
// ============================================================================

#[test]
fn aabb_size_and_alignment() {
    use std::mem::{align_of, size_of};
    assert_eq!(size_of::<Aabb>(), 24);
    assert_eq!(align_of::<Aabb>(), 4);
}

#[test]
fn aabb_overlaps() {
    let a = Aabb::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
    let b = Aabb::new(Vec3::new(0.5, 0.5, 0.5), Vec3::new(1.5, 1.5, 1.5));
    let c = Aabb::new(Vec3::new(2.0, 2.0, 2.0), Vec3::new(3.0, 3.0, 3.0));

    assert!(a.overlaps(b));
    assert!(b.overlaps(a));
    assert!(!a.overlaps(c));
    assert!(!c.overlaps(a));
}

#[test]
fn aabb_contains() {
    let a = Aabb::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
    assert!(a.contains(Vec3::new(0.5, 0.5, 0.5)));
    assert!(a.contains(Vec3::new(0.0, 0.0, 0.0))); // 边界
    assert!(!a.contains(Vec3::new(2.0, 0.5, 0.5)));
}

#[test]
fn aabb_expand_and_merge() {
    let mut a = Aabb::new(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0));
    a = a.expand_to_include(Vec3::new(2.0, 0.5, 0.5));
    assert!(a.contains(Vec3::new(2.0, 0.5, 0.5)));

    let b = Aabb::new(Vec3::new(0.5, 0.5, 0.5), Vec3::new(1.5, 1.5, 1.5));
    let merged = a.merged(b);
    assert!(merged.contains(Vec3::new(2.0, 0.5, 0.5)));
    assert!(merged.contains(Vec3::new(1.5, 1.5, 1.5)));
}

// ============================================================================
// v0.7 新增 API 测试
// ============================================================================

// ---- Quat::from_euler_ypr ----

#[test]
fn quat_from_euler_ypr_zero() {
    let q = Quat::from_euler_ypr(0.0, 0.0, 0.0);
    assert!((q.x - 0.0).abs() < 1e-6);
    assert!((q.y - 0.0).abs() < 1e-6);
    assert!((q.z - 0.0).abs() < 1e-6);
    assert!((q.w - 1.0).abs() < 1e-6);
}

#[test]
fn quat_from_euler_ypr_pure_yaw() {
    // yaw = π/2 绕 Z 轴，pitch = roll = 0
    let q = Quat::from_euler_ypr(std::f32::consts::FRAC_PI_2, 0.0, 0.0);
    // X 轴应被旋转到 Y 轴
    let rotated = q.rotate_vec3(Vec3::X);
    assert!((rotated.x - 0.0).abs() < 1e-5);
    assert!((rotated.y - 1.0).abs() < 1e-5);
    assert!((rotated.z - 0.0).abs() < 1e-5);
}

#[test]
fn quat_from_euler_ypr_compose() {
    // 小角度避免 gimbal lock
    let ypr = [0.3_f32, 0.2, 0.1];
    let q = Quat::from_euler_ypr(ypr[0], ypr[1], ypr[2]);
    let back = q.to_euler_ypr();
    assert!((back[0] - ypr[0]).abs() < 1e-5);
    assert!((back[1] - ypr[1]).abs() < 1e-5);
    assert!((back[2] - ypr[2]).abs() < 1e-5);
}

// ---- Quat::to_euler_ypr ----

#[test]
fn quat_to_euler_ypr_identity() {
    let ypr = Quat::IDENTITY.to_euler_ypr();
    assert!(ypr[0].abs() < 1e-6);
    assert!(ypr[1].abs() < 1e-6);
    assert!(ypr[2].abs() < 1e-6);
}

#[test]
fn quat_to_euler_ypr_gimbal_lock() {
    // pitch = π/2：构造绕 Y 轴 90° 旋转（只设 pitch）
    let q = Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2);
    let ypr = q.to_euler_ypr();
    assert!(!ypr[0].is_nan(), "yaw 不应为 NaN");
    assert!(!ypr[1].is_nan(), "pitch 不应为 NaN");
    assert!(!ypr[2].is_nan(), "roll 不应为 NaN");
    // pitch 应接近 +π/2（f32 精度下可能走 asin 分支，asin(0.99999994) ≈ 1.5704，
    // 差 ~3.4e-4 是 f32 ULP 误差的本质，不是实现 bug；放宽到 1e-3）
    assert!(
        (ypr[1] - std::f32::consts::FRAC_PI_2).abs() < 1e-3,
        "pitch 偏离 π/2 过大：ypr[1] = {}",
        ypr[1]
    );
    // pitch = -π/2 也应不爆 NaN
    let q_neg = Quat::from_axis_angle(Vec3::Y, -std::f32::consts::FRAC_PI_2);
    let ypr_neg = q_neg.to_euler_ypr();
    assert!(
        (ypr_neg[1] - (-std::f32::consts::FRAC_PI_2)).abs() < 1e-3,
        "pitch 偏离 -π/2 过大：ypr_neg[1] = {}",
        ypr_neg[1]
    );
}

// ---- Vec3::splat ----

#[test]
fn vec3_splat() {
    let v = Vec3::splat(2.5);
    assert_eq!(v, Vec3::new(2.5, 2.5, 2.5));
    let z = Vec3::splat(0.0);
    assert_eq!(z, Vec3::ZERO);
    let neg = Vec3::splat(-1.0);
    assert_eq!(neg, Vec3::new(-1.0, -1.0, -1.0));
}

#[test]
fn vec3_splat_arithmetic() {
    // 验证 splat 出来的向量与显式构造语义一致
    let s = Vec3::splat(3.0);
    assert_eq!(s + s, Vec3::new(6.0, 6.0, 6.0));
    assert_eq!(s * 2.0, Vec3::splat(6.0));
    assert!((s.length_squared() - 27.0).abs() < 1e-6); // 3*3 + 3*3 + 3*3 = 27
}

// ---- Mat3::from_diagonal ----

#[test]
fn mat3_from_diagonal() {
    let m = Mat3::from_diagonal(Vec3::new(2.0, 3.0, 4.0));
    // 对角矩阵 × 向量 = 逐分量缩放
    let r = m.mul_vec3(Vec3::new(1.0, 1.0, 1.0));
    assert_eq!(r, Vec3::new(2.0, 3.0, 4.0));
    // 非对角元素为 0
    assert!(m.m[0].y.abs() < 1e-6);
    assert!(m.m[1].x.abs() < 1e-6);
    assert!(m.m[2].x.abs() < 1e-6);
    // 验证对角项位置
    assert!((m.m[0].x - 2.0).abs() < 1e-6);
    assert!((m.m[1].y - 3.0).abs() < 1e-6);
    assert!((m.m[2].z - 4.0).abs() < 1e-6);
}

// ---- Mat3::from_basis ----

#[test]
fn mat3_from_basis() {
    // 三轴列向量 = 坐标轴
    let m = Mat3::from_basis(Vec3::X, Vec3::Y, Vec3::Z);
    // 应为 IDENTITY（行优先存储：row0 = (1,0,0)）
    assert_eq!(m, Mat3::IDENTITY);
}

#[test]
fn mat3_from_basis_columns() {
    // 验证列向量语义：m * X = x, m * Y = y, m * Z = z
    let x = Vec3::new(1.0, 0.0, 0.0);
    let y = Vec3::new(0.0, 1.0, 0.0);
    let z = Vec3::new(0.0, 0.0, 1.0);
    let m = Mat3::from_basis(x, y, z);
    assert_eq!(m.mul_vec3(Vec3::X), x);
    assert_eq!(m.mul_vec3(Vec3::Y), y);
    assert_eq!(m.mul_vec3(Vec3::Z), z);
}

// ---- Quat::to_mat3 ----

#[test]
fn quat_to_mat3_matches_from_quat() {
    // 三个不同四元数：IDENTITY、纯 yaw、复合小角度
    let qs = [
        Quat::IDENTITY,
        Quat::from_axis_angle(Vec3::Z, std::f32::consts::FRAC_PI_2),
        Quat::from_euler_ypr(0.3, 0.2, 0.1),
    ];
    for &q in &qs {
        let a = q.to_mat3();
        let b = Mat3::from_quat(q);
        assert_eq!(a, b, "to_mat3 必须 = from_quat");
        // 再验证语义：m * v == q.rotate_vec3(v)
        let v = Vec3::new(1.0, 2.0, 3.0);
        let m_v = a.mul_vec3(v);
        let q_v = q.rotate_vec3(v);
        assert!((m_v.x - q_v.x).abs() < 1e-5);
        assert!((m_v.y - q_v.y).abs() < 1e-5);
        assert!((m_v.z - q_v.z).abs() < 1e-5);
    }
}

// ---- Vec3::distance / distance_squared ----

#[test]
fn vec3_distance_zero() {
    let p = Vec3::new(1.0, 2.0, 3.0);
    assert!(p.distance(p).abs() < 1e-6);
    assert!(p.distance_squared(p).abs() < 1e-6);
}

#[test]
fn vec3_distance_axis_aligned() {
    // (1,2,3) → (4,6,3)：差 = (3, 4, 0)，距离 = 5
    let a = Vec3::new(1.0, 2.0, 3.0);
    let b = Vec3::new(4.0, 6.0, 3.0);
    assert!((a.distance(b) - 5.0).abs() < 1e-6);
    assert!((a.distance_squared(b) - 25.0).abs() < 1e-6);
}

#[test]
fn vec3_distance_squared_no_sqrt() {
    // 验证 distance_squared 等于差向量的 length_squared
    let a = Vec3::new(1.0, 2.0, 3.0);
    let b = Vec3::new(4.0, 6.0, 8.0);
    let ds = a.distance_squared(b);
    let expected = (b - a).length_squared();
    assert!((ds - expected).abs() < 1e-6);
    // 3² + 4² + 5² = 9 + 16 + 25 = 50
    assert!((ds - 50.0).abs() < 1e-6);
}

// ---- Aabb::from_points ----

#[test]
fn aabb_from_points_empty() {
    let pts: [Vec3; 0] = [];
    assert!(Aabb::from_points(&pts).is_none());
}

#[test]
fn aabb_from_points_single() {
    let pts = [Vec3::new(1.0, 2.0, 3.0)];
    let a = Aabb::from_points(&pts).unwrap();
    assert_eq!(a.min, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(a.max, Vec3::new(1.0, 2.0, 3.0));
    // 零体积 AABB 与 from_point 一致
    assert_eq!(a, Aabb::from_point(pts[0]));
}

#[test]
fn aabb_from_points_two_corners() {
    // 两个对角点
    let pts = [Vec3::new(-1.0, -2.0, -3.0), Vec3::new(4.0, 5.0, 6.0)];
    let a = Aabb::from_points(&pts).unwrap();
    assert_eq!(a.min, Vec3::new(-1.0, -2.0, -3.0));
    assert_eq!(a.max, Vec3::new(4.0, 5.0, 6.0));
}

#[test]
fn aabb_from_points_three_points() {
    // 三个点，验证 min/max 正确（顺序无关）
    let pts = [
        Vec3::new(1.0, 5.0, -2.0),
        Vec3::new(-3.0, 2.0, 4.0),
        Vec3::new(0.0, 0.0, 0.0),
    ];
    let a = Aabb::from_points(&pts).unwrap();
    assert_eq!(a.min, Vec3::new(-3.0, 0.0, -2.0));
    assert_eq!(a.max, Vec3::new(1.0, 5.0, 4.0));
    // 三个点都应被包含
    for &p in &pts {
        assert!(a.contains(p), "AABB 必须包含点 {p:?}");
    }
}

// ============================================================================
// Pod 派生验证（bytemuck）
// ============================================================================

#[test]
fn all_types_are_pod() {
    use bytemuck::Pod;
    fn assert_pod<T: Pod>() {}
    assert_pod::<Vec3>();
    assert_pod::<Quat>();
    assert_pod::<Mat3>();
    assert_pod::<Aabb>();
    assert_pod::<Transform>();
}

// ===== v0.8 API surface 扩充测试 =====

#[test]
fn vec3_min_component_wise() {
    let a = Vec3::new(1.0, 5.0, -3.0);
    let b = Vec3::new(2.0, -1.0, 4.0);
    let m = Vec3::min(a, b);
    assert_eq!(m, Vec3::new(1.0, -1.0, -3.0));
}

#[test]
fn vec3_max_component_wise() {
    let a = Vec3::new(1.0, 5.0, -3.0);
    let b = Vec3::new(2.0, -1.0, 4.0);
    let m = Vec3::max(a, b);
    assert_eq!(m, Vec3::new(2.0, 5.0, 4.0));
}

#[test]
fn vec3_abs_all_positive() {
    let v = Vec3::new(-1.0, 2.0, -3.0);
    assert_eq!(v.abs(), Vec3::new(1.0, 2.0, 3.0));
}

#[test]
fn vec3_min_component_picks_min() {
    let v = Vec3::new(1.0, -5.0, 3.0);
    assert!((v.min_component() - (-5.0)).abs() < 1e-6);
}

#[test]
fn vec3_max_component_picks_max() {
    let v = Vec3::new(1.0, -5.0, 3.0);
    assert!((v.max_component() - 3.0).abs() < 1e-6);
}

#[test]
fn vec3_neg_flips_sign() {
    let v = Vec3::new(1.0, -2.0, 3.0);
    assert_eq!(-v, Vec3::new(-1.0, 2.0, -3.0));
}

#[test]
fn quat_from_rotation_between_same_vector() {
    let a = Vec3::new(1.0, 0.0, 0.0);
    let q = Quat::from_rotation_between(a, a);
    // 同向量 → 单位四元数 (w ≈ 1, xyz ≈ 0)
    let v = q.rotate_vec3(a);
    assert!((v - a).length() < 1e-5);
}

#[test]
fn quat_from_rotation_between_perpendicular_90deg() {
    let from = Vec3::new(1.0, 0.0, 0.0);
    let to = Vec3::new(0.0, 1.0, 0.0);
    let q = Quat::from_rotation_between(from, to);
    let rotated = q.rotate_vec3(from);
    assert!((rotated - to).length() < 1e-5);
}

#[test]
fn quat_look_rotation_aligned_with_forward() {
    // look_rotation(forward=+Z, up=+Y) → 旋转 +Z 到 +Z，identity
    let q = Quat::look_rotation(Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 1.0, 0.0));
    let fwd = q.rotate_vec3(Vec3::new(0.0, 0.0, 1.0));
    assert!((fwd - Vec3::new(0.0, 0.0, 1.0)).length() < 1e-5);
}

#[test]
fn aabb_intersection_overlap_returns_smaller_box() {
    let a = Aabb::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(2.0, 2.0, 2.0));
    let b = Aabb::new(Vec3::new(1.0, 1.0, 1.0), Vec3::new(3.0, 3.0, 3.0));
    let inter = a.intersection(b).expect("overlap exists");
    assert_eq!(inter.min, Vec3::new(1.0, 1.0, 1.0));
    assert_eq!(inter.max, Vec3::new(2.0, 2.0, 2.0));
}

#[test]
fn aabb_intersection_disjoint_returns_none() {
    let a = Aabb::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
    let b = Aabb::new(Vec3::new(2.0, 2.0, 2.0), Vec3::new(3.0, 3.0, 3.0));
    assert!(a.intersection(b).is_none());
}

#[test]
fn transform_lerp_midpoint() {
    let a = Transform::new(Vec3::ZERO, Quat::IDENTITY);
    let b = Transform::new(
        Vec3::new(2.0, 4.0, 6.0),
        Quat::from_axis_angle(Vec3::Z, std::f32::consts::PI),
    );
    let mid = Transform::lerp(a, b, 0.5);
    // translation lerp at t=0.5 = (1, 2, 3)
    assert!((mid.translation.x - 1.0).abs() < 1e-5);
    assert!((mid.translation.y - 2.0).abs() < 1e-5);
    assert!((mid.translation.z - 3.0).abs() < 1e-5);
}
