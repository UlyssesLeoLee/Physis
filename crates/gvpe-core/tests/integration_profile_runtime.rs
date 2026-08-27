//! `gvpe-core` v0.8 加固测试 —— cross-crate integration + property-based + 边界。
//!
//! 加固目标：
//! - PhysicsProfile 不变式（NaN / 范围 / mass-density-inertia 一致性）
//! - BodySpecBuilder 必填校验 + is_static/mass 交叉检查 + mass() 覆盖语义
//! - RuntimeDescriptor validate 跨 body 校验 + dynamic/static 计数
//! - 联合 gvpe-math::Vec3/Quat：构造可运行的 body 场景
//!
//! 加固 commit 基线：v0.8 (63b9921) — PhysicsProfile::from_mass + BodySpecBuilder::mass。
//! 修订者：Mavis 接手 agent per DEC-008 (2026-08-27 08:00 JST 指令)。

use gvpe_core::{
    BodyHandle, BodySpec, BodySpecBuilder, CoreError, InitialTransform, PhysicsLodTag,
    PhysicsProfile, RuntimeDescriptor, ShapeDesc,
};
use gvpe_math::{Quat, Vec3};
use proptest::prelude::*;

/// 浮点容差。
const EPS: f32 = 1e-5;

/// 合理 mass（> 0 且有限）。
fn safe_mass() -> impl Strategy<Value = f32> {
    (EPS..1.0e3_f32).prop_filter("positive finite", |v| v.is_finite() && *v > 0.0)
}

/// 合理 density（> 0）。
fn safe_density() -> impl Strategy<Value = f32> {
    (EPS..1.0e3_f32).prop_filter("positive finite", |v| v.is_finite() && *v > 0.0)
}

/// 合理 unit range（[0, 1]）。
fn unit_range() -> impl Strategy<Value = f32> {
    (0.0_f32..=1.0_f32).prop_filter("finite", |v| v.is_finite())
}

/// 构造有效 PhysicsProfile 的策略。
fn valid_profile() -> impl Strategy<Value = PhysicsProfile> {
    (
        safe_mass(),
        safe_density(),
        unit_range(),
        unit_range(),
        unit_range(),
        unit_range(),
        1u16..100,
    )
        .prop_map(|(mass, density, friction, restitution, dl, da, iter)| {
            let mut p = PhysicsProfile::default_solid();
            p.mass = mass;
            p.density = density;
            p.friction = friction;
            p.restitution = restitution;
            p.damping_linear = dl;
            p.damping_angular = da;
            p.solver_iterations = iter;
            p
        })
}

/// 构造有效 BodySpec 的策略。
fn valid_body_spec() -> impl Strategy<Value = BodySpec> {
    (unit_range(), unit_range(), unit_range(), unit_range(), valid_profile())
        .prop_map(|(radius, x, y, z, profile)| {
            BodySpec::builder()
                .shape(ShapeDesc::Sphere { radius })
                .transform(InitialTransform {
                    translation: Vec3::new(x, y, z),
                    rotation_yaw_pitch_roll: [0.0; 3],
                })
                .profile(profile)
                .is_static(false)
                .build()
                .expect("valid profile 应能 build")
        })
}

proptest! {
    // ===== PhysicsProfile 不变式 =====

    fn profile_from_mass_positive(mass in safe_mass()) {
        let p = PhysicsProfile::from_mass(mass);
        prop_assert_eq!(p.mass, mass);
        prop_assert!(p.validate().is_ok(), "m>0 必 validate 通过");
        prop_assert!(!p.is_static());
    }

    fn profile_from_mass_non_positive(mass in -1.0e3_f32..=0.0_f32) {
        prop_assume!(mass.is_finite());
        let p = PhysicsProfile::from_mass(mass);
        prop_assert_eq!(p.mass, 0.0, "m≤0 必被映射到 static");
        prop_assert!(p.validate().is_ok());
        prop_assert!(p.is_static());
    }

    fn profile_is_static_iff_mass_zero(mass in 0.0_f32..1.0e3_f32) {
        let mut p = PhysicsProfile::default_solid();
        p.mass = mass;
        prop_assert_eq!(p.is_static(), mass == 0.0);
    }

    fn profile_inverse_inertia_dynamic(
        diag in (EPS..1.0e3_f32).prop_filter("finite", |v| v.is_finite()),
    ) {
        let mut p = PhysicsProfile::default_solid();
        p.inertia = [diag, 0.0, 0.0, 0.0, diag, 0.0, 0.0, 0.0, diag];
        let inv = p.inverse_inertia();
        prop_assert!((inv[0] - 1.0 / diag).abs() < EPS);
        prop_assert!((inv[1] - 1.0 / diag).abs() < EPS);
        prop_assert!((inv[2] - 1.0 / diag).abs() < EPS);
    }

    fn profile_inertia_diagonal_indices(ix in 0.0_f32..1.0e3_f32, iy in 0.0_f32..1.0e3_f32, iz in 0.0_f32..1.0e3_f32) {
        let mut p = PhysicsProfile::default_solid();
        p.inertia = [ix, 0.0, 0.0, 0.0, iy, 0.0, 0.0, 0.0, iz];
        let d = p.inertia_diagonal();
        prop_assert_eq!(d[0], ix);
        prop_assert_eq!(d[1], iy);
        prop_assert_eq!(d[2], iz);
    }

    fn profile_valid_strategy_passes(p in valid_profile()) {
        prop_assert!(p.validate().is_ok());
    }

    // ===== BodySpecBuilder =====

    fn builder_happy_path(spec in valid_body_spec()) {
        prop_assert!(spec.profile.validate().is_ok());
        prop_assert!(!spec.is_static || spec.profile.is_static(), "is_static 交叉一致");
    }

    fn builder_mass_override(mass in safe_mass()) {
        let spec = BodySpec::builder()
            .shape(ShapeDesc::Sphere { radius: 0.5 })
            .transform(InitialTransform {
                translation: Vec3::ZERO,
                rotation_yaw_pitch_roll: [0.0; 3],
            })
            .profile(PhysicsProfile::default_solid())
            .mass(mass)
            .build()
            .unwrap();
        prop_assert_eq!(spec.profile.mass, mass, "mass() 必覆盖");
    }

    // ===== RuntimeDescriptor 跨 body 校验 =====

    fn runtime_mixed_static_dynamic(n_static in 0..4usize, n_dynamic in 1..4usize) {
        let mut rt = RuntimeDescriptor::with_capacity(n_static + n_dynamic);
        for i in 0..n_static {
            let spec = BodySpec::builder()
                .shape(ShapeDesc::Plane { normal: [0.0, 1.0, 0.0], offset: 0.0 })
                .transform(InitialTransform {
                    translation: Vec3::new(0.0, i as f32, 0.0),
                    rotation_yaw_pitch_roll: [0.0; 3],
                })
                .profile(PhysicsProfile::default_static())
                .is_static(true)
                .build()
                .unwrap();
            rt.add_body(spec);
        }
        for i in 0..n_dynamic {
            let spec = BodySpec::builder()
                .shape(ShapeDesc::Sphere { radius: 0.5 })
                .transform(InitialTransform {
                    translation: Vec3::new(i as f32, 10.0, 0.0),
                    rotation_yaw_pitch_roll: [0.0; 3],
                })
                .profile(PhysicsProfile::default_solid())
                .is_static(false)
                .build()
                .unwrap();
            rt.add_body(spec);
        }
        prop_assert!(rt.validate().is_ok());
        prop_assert_eq!(rt.body_count(), n_static + n_dynamic);
        prop_assert_eq!(rt.static_body_count(), n_static);
        prop_assert_eq!(rt.dynamic_body_count(), n_dynamic);
    }

    fn runtime_counts_sum(n_static in 0..6usize, n_dynamic in 0..6usize) {
        let total = n_static + n_dynamic;
        prop_assume!(total > 0);
        let mut rt = RuntimeDescriptor::with_capacity(total);
        for _ in 0..n_static {
            let s = BodySpec::builder()
                .shape(ShapeDesc::Plane { normal: [0.0, 1.0, 0.0], offset: 0.0 })
                .transform(InitialTransform {
                    translation: Vec3::ZERO,
                    rotation_yaw_pitch_roll: [0.0; 3],
                })
                .profile(PhysicsProfile::default_static())
                .is_static(true)
                .build()
                .unwrap();
            rt.add_body(s);
        }
        for _ in 0..n_dynamic {
            let s = BodySpec::builder()
                .shape(ShapeDesc::Sphere { radius: 0.5 })
                .transform(InitialTransform {
                    translation: Vec3::ZERO,
                    rotation_yaw_pitch_roll: [0.0; 3],
                })
                .profile(PhysicsProfile::default_solid())
                .build()
                .unwrap();
            rt.add_body(s);
        }
        prop_assert_eq!(rt.static_body_count() + rt.dynamic_body_count(), rt.body_count());
    }

    // ===== cross-crate：Vec3/Quat 与 BodySpec 联合 =====

    fn initial_transform_vec3_round_trip(
        x in unit_range(), y in unit_range(), z in unit_range(),
        yaw in -1.0_f32..1.0_f32,
        pitch in -1.0_f32..1.0_f32,
        roll in -1.0_f32..1.0_f32,
    ) {
        let v = Vec3::new(x, y, z);
        let t = InitialTransform {
            translation: v,
            rotation_yaw_pitch_roll: [yaw, pitch, roll],
        };
        let _q = Quat::from_euler_ypr(t.rotation_yaw_pitch_roll[0], t.rotation_yaw_pitch_roll[1], t.rotation_yaw_pitch_roll[2]);
        let spec = BodySpec::builder()
            .shape(ShapeDesc::Sphere { radius: 0.5 })
            .transform(t)
            .profile(PhysicsProfile::default_solid())
            .build()
            .unwrap();
        prop_assert!((spec.initial_transform.translation.x - v.x).abs() < EPS);
        prop_assert!((spec.initial_transform.translation.y - v.y).abs() < EPS);
        prop_assert!((spec.initial_transform.translation.z - v.z).abs() < EPS);
    }

    fn shape_desc_box3_round_trip(
        hx in 0.01_f32..10.0_f32,
        hy in 0.01_f32..10.0_f32,
        hz in 0.01_f32..10.0_f32,
    ) {
        let s = ShapeDesc::Box3 { half_extents: [hx, hy, hz] };
        let spec = BodySpec::builder()
            .shape(s)
            .transform(InitialTransform {
                translation: Vec3::ZERO,
                rotation_yaw_pitch_roll: [0.0; 3],
            })
            .profile(PhysicsProfile::default_solid())
            .build()
            .unwrap();
        match spec.shape {
            ShapeDesc::Box3 { half_extents } => {
                prop_assert!((half_extents[0] - hx).abs() < EPS);
                prop_assert!((half_extents[1] - hy).abs() < EPS);
                prop_assert!((half_extents[2] - hz).abs() < EPS);
            }
            _ => prop_assert!(false, "ShapeDesc 必为 Box3"),
        }
    }
}

// ===== 静态单测（proptest! 不支持无参 fn / 需边界对照） =====

#[test]
fn profile_default_solid_validates() {
    assert!(PhysicsProfile::default_solid().validate().is_ok());
}

#[test]
fn profile_default_static_validates() {
    assert!(PhysicsProfile::default_static().validate().is_ok());
}

#[test]
fn profile_inverse_inertia_static_returns_zero() {
    let p = PhysicsProfile::default_static();
    let inv = p.inverse_inertia();
    assert_eq!(inv, [0.0, 0.0, 0.0]);
}

#[test]
fn profile_validate_rejects_negative_mass() {
    let mut p = PhysicsProfile::default_solid();
    p.mass = -1.0;
    let err = p.validate().unwrap_err();
    assert!(matches!(err, CoreError::ProfileInconsistent { field: "mass", .. }));
}

#[test]
fn profile_validate_rejects_invalid_density() {
    let mut p = PhysicsProfile::default_solid();
    p.mass = 1.0;
    p.density = 0.0;
    let err = p.validate().unwrap_err();
    assert!(matches!(err, CoreError::ProfileInconsistent { field: "density", .. }));
}

#[test]
fn profile_validate_rejects_friction_out_of_range() {
    let mut p = PhysicsProfile::default_solid();
    p.friction = 1.5;
    let err = p.validate().unwrap_err();
    assert!(matches!(err, CoreError::ProfileInconsistent { field: "friction", .. }));
}

#[test]
fn profile_validate_rejects_zero_iterations() {
    let mut p = PhysicsProfile::default_solid();
    p.solver_iterations = 0;
    let err = p.validate().unwrap_err();
    assert!(matches!(err, CoreError::ProfileInconsistent { field: "solver_iterations", .. }));
}

#[test]
fn profile_validate_rejects_nan_mass() {
    let mut p = PhysicsProfile::default_solid();
    p.mass = f32::NAN;
    let err = p.validate().unwrap_err();
    assert!(matches!(err, CoreError::ProfileInconsistent { field: "mass", .. }));
}

#[test]
fn builder_missing_shape() {
    let r = BodySpec::builder()
        .transform(InitialTransform {
            translation: Vec3::ZERO,
            rotation_yaw_pitch_roll: [0.0; 3],
        })
        .profile(PhysicsProfile::default_solid())
        .build();
    let err = r.unwrap_err();
    assert!(matches!(err, CoreError::BodySpecMissingField { field: "shape" }));
}

#[test]
fn builder_missing_transform() {
    let r = BodySpec::builder()
        .shape(ShapeDesc::Sphere { radius: 0.5 })
        .profile(PhysicsProfile::default_solid())
        .build();
    let err = r.unwrap_err();
    assert!(matches!(err, CoreError::BodySpecMissingField { field: "initial_transform" }));
}

#[test]
fn builder_missing_profile() {
    let r = BodySpec::builder()
        .shape(ShapeDesc::Sphere { radius: 0.5 })
        .transform(InitialTransform {
            translation: Vec3::ZERO,
            rotation_yaw_pitch_roll: [0.0; 3],
        })
        .build();
    let err = r.unwrap_err();
    assert!(matches!(err, CoreError::BodySpecMissingField { field: "profile" }));
}

#[test]
fn builder_is_static_mismatch() {
    let r = BodySpec::builder()
        .shape(ShapeDesc::Sphere { radius: 0.5 })
        .transform(InitialTransform {
            translation: Vec3::ZERO,
            rotation_yaw_pitch_roll: [0.0; 3],
        })
        .profile(PhysicsProfile::default_solid())
        .is_static(true)
        .build();
    let err = r.unwrap_err();
    assert!(matches!(err, CoreError::ProfileInconsistent { field: "is_static", .. }));
}

#[test]
fn builder_is_static_mismatch_reverse() {
    let r = BodySpec::builder()
        .shape(ShapeDesc::Sphere { radius: 0.5 })
        .transform(InitialTransform {
            translation: Vec3::ZERO,
            rotation_yaw_pitch_roll: [0.0; 3],
        })
        .profile(PhysicsProfile::default_static())
        .is_static(false)
        .build();
    let err = r.unwrap_err();
    assert!(matches!(err, CoreError::ProfileInconsistent { field: "is_static", .. }));
}

#[test]
fn builder_static_with_mass_zero_valid() {
    let spec = BodySpec::builder()
        .shape(ShapeDesc::Plane { normal: [0.0, 1.0, 0.0], offset: 0.0 })
        .transform(InitialTransform {
            translation: Vec3::ZERO,
            rotation_yaw_pitch_roll: [0.0; 3],
        })
        .profile(PhysicsProfile::default_static())
        .is_static(true)
        .build()
        .unwrap();
    assert!(spec.is_static);
    assert!(spec.profile.is_static());
}

#[test]
fn runtime_empty_rejected() {
    let r = RuntimeDescriptor::empty().validate();
    let err = r.unwrap_err();
    assert!(matches!(err, CoreError::DescriptorEmpty), "expected DescriptorEmpty, got {:?}", err);
}

#[test]
fn runtime_body_index_out_of_bounds() {
    let rt = RuntimeDescriptor::empty();
    assert!(rt.body(0).is_none());
    assert!(rt.body(usize::MAX).is_none());
}

#[test]
fn runtime_with_capacity_initial() {
    let rt = RuntimeDescriptor::with_capacity(16);
    assert_eq!(rt.body_count(), 0);
    assert_eq!(rt.static_body_count(), 0);
    assert_eq!(rt.dynamic_body_count(), 0);
}

#[test]
fn body_handle_debug_copy_eq() {
    let h = BodyHandle { index: 7, generation: 3 };
    let h2 = h; // Copy
    assert_eq!(h, h2);
    let _dbg = format!("{:?}", h);
}

#[test]
fn lod_tag_values_distinct() {
    use PhysicsLodTag::*;
    let vals = [Lod0Full, Lod1Reduced, Lod2Approximation, Lod3CachedBehavior, Lod4Static];
    for (i, a) in vals.iter().enumerate() {
        for b in &vals[i + 1..] {
            assert_ne!(a, b);
        }
    }
}
