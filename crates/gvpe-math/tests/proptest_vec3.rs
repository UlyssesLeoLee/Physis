//! `gvpe-math` v0.7/v0.8 加固测试 —— property-based。
//!
//! 加固目标：
//! - `Vec3` 代数恒等式（交换律 / 分配律 / 零元 / 逆元）
//! - `Vec3` 归一化 / 距离 / 投影
//! - `Quat` 单位元 / Hamilton 积结合律 / slerp 端点 / look_rotation 性质
//! - `Aabb` 合并 / 交集 / 包含单调性
//! - `Transform` lerp 端点
//!
//! 加固 commit 基线：v0.7 (58f0a31) + v0.8 (63b9921)。
//! 修订者：Mavis 接手 agent per DEC-008 (2026-08-27 08:00 JST 指令)。

// 测试模块 lint 集中允许：见 `crates/gvpe-core/tests/integration_profile_runtime.rs` 同段说明
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_parens)]
#![allow(clippy::float_cmp)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::collection_is_never_read)]
#![allow(clippy::double_parens)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::match_wildcard_for_single_variants)]

use gvpe_math::{Aabb, Mat3, Quat, Transform, Vec3};
use proptest::prelude::*;

/// 浮点容差。
const EPS: f32 = 1e-5;

/// 有限 f32 策略：排除 NaN / Inf / 超大值。
fn finite_f32() -> impl Strategy<Value = f32> {
    (-1.0e3_f32..=1.0e3_f32).prop_filter("finite", |v| v.is_finite())
}

/// 有限 Vec3 策略。
fn finite_vec3() -> impl Strategy<Value = Vec3> {
    (finite_f32(), finite_f32(), finite_f32()).prop_map(|(x, y, z)| Vec3::new(x, y, z))
}

/// 非零 Vec3 策略。
fn nonzero_vec3() -> impl Strategy<Value = Vec3> {
    finite_vec3().prop_filter("nonzero", |v| v.length_squared() > EPS * EPS)
}

/// 归一化 Vec3 策略。
fn unit_vec3() -> impl Strategy<Value = Vec3> {
    nonzero_vec3().prop_map(|v| v.normalize())
}

/// 浮点近似相等。
fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() <= EPS || ((a - b).abs() / (a.abs() + b.abs() + 1.0)) <= EPS
}

/// Vec3 近似相等。
fn vec3_approx_eq(a: Vec3, b: Vec3) -> bool {
    approx_eq(a.x, b.x) && approx_eq(a.y, b.y) && approx_eq(a.z, b.z)
}

proptest! {
    // ===== Vec3 代数 =====

    fn vec3_add_commutative(v in finite_vec3(), w in finite_vec3()) {
        prop_assert_eq!(v + w, w + v);
    }

    fn vec3_add_associative(v in finite_vec3(), w in finite_vec3(), u in finite_vec3()) {
        prop_assert_eq!((v + w) + u, v + (w + u));
    }

    fn vec3_add_inverse(v in finite_vec3()) {
        let s = v + (-v);
        prop_assert!(s.length_squared() < EPS);
    }

    fn vec3_add_identity(v in finite_vec3()) {
        prop_assert_eq!(v + Vec3::ZERO, v);
    }

    fn vec3_scalar_distributive(v in finite_vec3(), w in finite_vec3(), a in finite_f32()) {
        let lhs = (v + w) * a;
        let rhs = v * a + w * a;
        prop_assert!(vec3_approx_eq(lhs, rhs));
    }

    fn vec3_splat_length_squared(v in finite_f32()) {
        let s = Vec3::splat(v);
        prop_assert!(approx_eq(s.length_squared(), 3.0 * v * v));
    }

    fn vec3_min_max_clamp(v in finite_vec3(), lo in finite_f32(), hi in finite_f32()) {
        prop_assume!(lo <= hi);
        let lo_v = Vec3::splat(lo);
        let hi_v = Vec3::splat(hi);
        let clamped = Vec3::min(Vec3::max(v, lo_v), hi_v);
        prop_assert!(clamped.x >= lo - EPS && clamped.x <= hi + EPS);
        prop_assert!(clamped.y >= lo - EPS && clamped.y <= hi + EPS);
        prop_assert!(clamped.z >= lo - EPS && clamped.z <= hi + EPS);
    }

    fn vec3_abs_involution(v in finite_vec3()) {
        prop_assert_eq!(v.abs(), (-v).abs());
    }

    fn vec3_min_max_component_order(v in finite_vec3()) {
        prop_assert!(v.min_component() <= v.max_component() + EPS);
    }

    fn vec3_dot_self_is_length_squared(v in finite_vec3()) {
        prop_assert!(approx_eq(v.dot(v), v.length_squared()));
    }

    fn vec3_dot_commutative(v in finite_vec3(), w in finite_vec3()) {
        prop_assert!(approx_eq(v.dot(w), w.dot(v)));
    }

    fn vec3_cross_anticommutative(v in finite_vec3(), w in finite_vec3()) {
        let lhs = v.cross(w);
        let rhs = w.cross(v);
        prop_assert!(vec3_approx_eq(lhs, -rhs));
    }

    fn vec3_cross_self_is_zero(v in finite_vec3()) {
        let c = v.cross(v);
        prop_assert!(c.length_squared() < EPS);
    }

    // ===== Vec3 距离 / 投影 =====

    fn vec3_distance_squared_relationship(a in finite_vec3(), b in finite_vec3()) {
        let d = a.distance(b);
        let ds = a.distance_squared(b);
        prop_assert!(approx_eq(d * d, ds));
    }

    fn vec3_distance_self_zero(a in finite_vec3()) {
        prop_assert!(a.distance(a) < EPS);
    }

    fn vec3_project_onto_axis_aligned(v in finite_vec3(), axis in nonzero_vec3()) {
        let proj = v.project_onto(axis);
        let cross = proj.cross(axis);
        prop_assert!(cross.length_squared() < EPS * EPS * axis.length_squared());
    }

    fn vec3_normalize_is_unit(v in nonzero_vec3()) {
        let n = v.normalize();
        prop_assert!(approx_eq(n.length(), 1.0));
    }

    // ===== Quat =====

    fn quat_identity_left(v in any::<[f32; 4]>()) {
        let q = Quat::new(v[0], v[1], v[2], v[3]);
        let r = Quat::IDENTITY.mul(q);
        prop_assert_eq!(r.x, q.x);
        prop_assert_eq!(r.y, q.y);
        prop_assert_eq!(r.z, q.z);
        prop_assert_eq!(r.w, q.w);
    }

    fn quat_identity_right(v in any::<[f32; 4]>()) {
        let q = Quat::new(v[0], v[1], v[2], v[3]);
        let r = q.mul(Quat::IDENTITY);
        prop_assert_eq!(r.x, q.x);
        prop_assert_eq!(r.y, q.y);
        prop_assert_eq!(r.z, q.z);
        prop_assert_eq!(r.w, q.w);
    }

    fn quat_mul_associative(p in any::<[f32; 4]>(), q in any::<[f32; 4]>(), r in any::<[f32; 4]>()) {
        let p = Quat::new(p[0], p[1], p[2], p[3]);
        let q = Quat::new(q[0], q[1], q[2], q[3]);
        let r = Quat::new(r[0], r[1], r[2], r[3]);
        let lhs = p.mul(q).mul(r);
        let rhs = p.mul(q.mul(r));
        prop_assert!(approx_eq(lhs.x, rhs.x));
        prop_assert!(approx_eq(lhs.y, rhs.y));
        prop_assert!(approx_eq(lhs.z, rhs.z));
        prop_assert!(approx_eq(lhs.w, rhs.w));
    }

    fn quat_slerp_t_zero(ax in finite_f32(), ay in finite_f32(), az in finite_f32(), aw in finite_f32()) {
        let a = Quat::new(ax, ay, az, aw);
        let r = Quat::slerp(a, a, 0.0);
        prop_assert!(approx_eq(r.x, a.x));
        prop_assert!(approx_eq(r.y, a.y));
        prop_assert!(approx_eq(r.z, a.z));
        prop_assert!(approx_eq(r.w, a.w));
    }

    fn quat_from_rotation_between_same(u in unit_vec3()) {
        let q = Quat::from_rotation_between(u, u);
        prop_assert!(approx_eq(q.x, 0.0));
        prop_assert!(approx_eq(q.y, 0.0));
        prop_assert!(approx_eq(q.z, 0.0));
        prop_assert!(approx_eq(q.w, 1.0));
    }

    fn quat_from_rotation_between_correctness(u in unit_vec3(), v in unit_vec3()) {
        let dot = u.dot(v);
        prop_assume!(dot > -0.999 && dot < 0.999);
        let q = Quat::from_rotation_between(u, v);
        let rotated = q.rotate_vec3(u);
        prop_assert!(vec3_approx_eq(rotated, v));
    }

    fn quat_look_rotation_aligns_forward(f in unit_vec3(), up in unit_vec3()) {
        prop_assume!(up.cross(f).length_squared() > EPS * EPS);
        prop_assume!(f.y.abs() < 0.999);
        let q = Quat::look_rotation(f, up);
        let rotated = q.rotate_vec3(f);
        let neg_z = Vec3::new(0.0, 0.0, -1.0);
        prop_assert!(vec3_approx_eq(rotated, neg_z));
    }

    // ===== Mat3 =====

    fn mat3_from_diagonal_scales_components(d in finite_vec3(), v in finite_vec3()) {
        let m = Mat3::from_diagonal(d);
        let expected = Vec3::new(d.x * v.x, d.y * v.y, d.z * v.z);
        let r = m.mul_vec3(v);
        prop_assert!(vec3_approx_eq(r, expected));
    }

    fn mat3_from_quat_matches_quat_rotate(q in any::<[f32; 4]>(), v in finite_vec3()) {
        let q = Quat::new(q[0], q[1], q[2], q[3]).normalize();
        let m = Mat3::from_quat(q);
        let a = m.mul_vec3(v);
        let b = q.rotate_vec3(v);
        prop_assert!(vec3_approx_eq(a, b));
    }

    fn quat_to_mat3_matches_from_quat(q in any::<[f32; 4]>()) {
        let q = Quat::new(q[0], q[1], q[2], q[3]).normalize();
        let a = q.to_mat3();
        let b = Mat3::from_quat(q);
        prop_assert!(vec3_approx_eq(a.m[0], b.m[0]));
        prop_assert!(vec3_approx_eq(a.m[1], b.m[1]));
        prop_assert!(vec3_approx_eq(a.m[2], b.m[2]));
    }

    fn mat3_from_basis_preserves_basis_vectors(x in unit_vec3(), y in unit_vec3(), z in unit_vec3()) {
        let m = Mat3::from_basis(x, y, z);
        let r = m.mul_vec3(Vec3::new(1.0, 0.0, 0.0));
        prop_assert!(vec3_approx_eq(r, x));
    }

    // ===== Aabb =====

    fn aabb_from_points_order_invariant(pts in proptest::collection::vec(finite_vec3(), 1..16)) {
        let a = Aabb::from_points(&pts);
        let b = Aabb::from_points(&pts.iter().rev().copied().collect::<Vec<_>>().as_slice());
        prop_assert_eq!(a, b);
    }

    fn aabb_overlaps_self(min in finite_vec3(), max in finite_vec3()) {
        prop_assume!(min.x <= max.x && min.y <= max.y && min.z <= max.z);
        let a = Aabb::new(min, max);
        prop_assert!(a.overlaps(a));
    }

    fn aabb_contains_implies_in_range(min in finite_vec3(), max in finite_vec3(), p in finite_vec3()) {
        prop_assume!(min.x <= max.x && min.y <= max.y && min.z <= max.z);
        let a = Aabb::new(min, max);
        if a.contains(p) {
            prop_assert!(p.x >= min.x - EPS && p.x <= max.x + EPS);
            prop_assert!(p.y >= min.y - EPS && p.y <= max.y + EPS);
            prop_assert!(p.z >= min.z - EPS && p.z <= max.z + EPS);
        }
    }

    fn aabb_intersection_self(min in finite_vec3(), max in finite_vec3()) {
        prop_assume!(min.x <= max.x && min.y <= max.y && min.z <= max.z);
        let a = Aabb::new(min, max);
        prop_assert_eq!(a.intersection(a), Some(a));
    }

    fn aabb_intersection_iff_overlaps(amin in finite_vec3(), amax in finite_vec3(), bmin in finite_vec3(), bmax in finite_vec3()) {
        prop_assume!(amin.x <= amax.x && amin.y <= amax.y && amin.z <= amax.z);
        prop_assume!(bmin.x <= bmax.x && bmin.y <= bmax.y && bmin.z <= bmax.z);
        let a = Aabb::new(amin, amax);
        let b = Aabb::new(bmin, bmax);
        let overlaps = a.overlaps(b);
        let intersection = a.intersection(b);
        match intersection {
            Some(_) => prop_assert!(overlaps),
            None => prop_assert!(!overlaps),
        }
    }

    // ===== Transform =====

    fn transform_identity_is_noop(v in finite_vec3()) {
        let r = Transform::IDENTITY.transform_vec3(v);
        prop_assert!(vec3_approx_eq(r, v));
    }

    fn transform_inverse_at_translation_is_zero(t in finite_vec3(), q in any::<[f32; 4]>()) {
        let q = Quat::new(q[0], q[1], q[2], q[3]).normalize();
        let xf = Transform::new(t, q);
        let inv = xf.inverse();
        let r = inv.transform_vec3(t);
        prop_assert!(r.length_squared() < EPS);
    }

    fn transform_lerp_endpoints(at in finite_f32(), bt in finite_f32(), aq in any::<[f32; 4]>(), bq in any::<[f32; 4]>()) {
        let at = Vec3::new(at, 0.0, 0.0);
        let bt = Vec3::new(bt, 0.0, 0.0);
        let aq = Quat::new(aq[0], aq[1], aq[2], aq[3]).normalize();
        let bq = Quat::new(bq[0], bq[1], bq[2], bq[3]).normalize();
        let a = Transform::new(at, aq);
        let b = Transform::new(bt, bq);
        let r0 = Transform::lerp(a, b, 0.0);
        prop_assert!(vec3_approx_eq(r0.translation, a.translation));
        prop_assert!(approx_eq(r0.rotation.x, a.rotation.x));
        prop_assert!(approx_eq(r0.rotation.y, a.rotation.y));
        prop_assert!(approx_eq(r0.rotation.z, a.rotation.z));
        prop_assert!(approx_eq(r0.rotation.w, a.rotation.w));
        let r1 = Transform::lerp(a, b, 1.0);
        prop_assert!(vec3_approx_eq(r1.translation, b.translation));
    }
}
