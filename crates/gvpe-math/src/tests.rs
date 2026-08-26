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

    assert_eq!(a.dot(b), 0.0);
    assert_eq!(a.dot(a), 1.0);

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
    assert_eq!(q.length_squared(), 1.0);
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
