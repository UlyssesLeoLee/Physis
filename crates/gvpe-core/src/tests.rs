//! `gvpe-core` 单元测试。

use crate::{
    BodyHandle, BodySpec, BodySpecBuilder, ConstraintHandle, CoreError, InitialTransform,
    IslandHandle, PhysicsLodTag, PhysicsProfile, RuntimeDescriptor, ShapeDesc, SolverTypeId,
};
use gvpe_math::Vec3;

// ============================================================================
// 句柄测试
// ============================================================================

#[test]
fn body_handle_size() {
    use std::mem::{align_of, size_of};
    assert_eq!(size_of::<BodyHandle>(), 8);
    assert_eq!(align_of::<BodyHandle>(), 4);
}

#[test]
fn body_handle_invalid_detection() {
    let h = BodyHandle::INVALID;
    assert!(h.is_invalid());

    let h2 = BodyHandle::new(1, 1);
    assert!(!h2.is_invalid());
}

#[test]
fn constraint_handle_size() {
    use std::mem::size_of;
    assert_eq!(size_of::<ConstraintHandle>(), 8);
}

#[test]
fn island_handle_size() {
    use std::mem::{align_of, size_of};
    assert_eq!(size_of::<IslandHandle>(), 4);
    assert_eq!(align_of::<IslandHandle>(), 4);
}

// ============================================================================
// PhysicsProfile 测试
// ============================================================================

#[test]
fn physics_profile_size() {
    use std::mem::{align_of, size_of};
    // 字段：3 个 f32 + 9 个 f32 + 4 个 f32 + 2 个 f32 + u8 (solver) + u16 (iter) + u8 (collision) + u8 (lod) + u8 (padding) = 12 + 36 + 16 + 8 + 1 + 2 + 1 + 1 + 1 = 78 字节（带 padding）
    // 但 align 4，可能有 padding：
    //   mass(4) + density(4) + inertia(36) + friction(4) + restitution(4) + damping_linear(4) + damping_angular(4) + stiffness(4) + compliance(4) + viscosity(4) + solver_type(1) + [pad 1] + solver_iterations(2) + collision_profile(1) + approximation_level(1) + padding(1) = 80 字节
    let size = size_of::<PhysicsProfile>();
    assert!(
        (78..=84).contains(&size),
        "PhysicsProfile size = {size} (expected 78-84)"
    );
    assert_eq!(align_of::<PhysicsProfile>(), 4);
}

#[test]
fn physics_profile_pod() {
    use bytemuck::Pod;
    fn assert_pod<T: Pod>() {}
    assert_pod::<PhysicsProfile>();
}

#[test]
fn physics_profile_default_solid() {
    let p = PhysicsProfile::default_solid();
    assert!((p.mass - 1.0).abs() < 1e-6);
    assert_eq!(p.solver_type, SolverTypeId::SequentialImpulse);
    assert_eq!(p.approximation_level, PhysicsLodTag::Lod0Full);
}

#[test]
fn physics_profile_default_static() {
    let p = PhysicsProfile::default_static();
    assert!(p.mass.abs() < f32::EPSILON);
}

// ============================================================================
// RuntimeDescriptor 测试
// ============================================================================

#[test]
fn runtime_descriptor_empty() {
    let d = RuntimeDescriptor::empty();
    assert_eq!(d.body_count(), 0);
    assert!((d.gravity.y - (-9.81)).abs() < 1e-4);
}

#[test]
fn runtime_descriptor_add_body() {
    let mut d = RuntimeDescriptor::empty();
    let spec = BodySpec {
        shape: ShapeDesc::Sphere { radius: 0.5 },
        initial_transform: InitialTransform {
            translation: Vec3::new(0.0, 10.0, 0.0),
            rotation_yaw_pitch_roll: [0.0, 0.0, 0.0],
        },
        profile: PhysicsProfile::default_solid(),
        is_static: false,
    };
    d.add_body(spec);
    assert_eq!(d.body_count(), 1);
}

// ============================================================================
// 错误类型测试
// ============================================================================

#[test]
fn core_error_display() {
    let e = CoreError::DescriptorEmpty;
    assert_eq!(format!("{e}"), "RuntimeDescriptor 为空");

    let e = CoreError::ProfileInconsistent {
        field: "mass",
        value: -1.0,
    };
    assert!(format!("{e}").contains("mass"));
}

// ============================================================================
// PhysicsProfile::validate / is_static / inverse_inertia
// ============================================================================

fn make_solid_profile() -> PhysicsProfile {
    PhysicsProfile::default_solid()
}

#[test]
fn physics_profile_validate_all_legal() {
    let p = make_solid_profile();
    assert!(p.validate().is_ok());
}

#[test]
fn physics_profile_validate_mass_negative_fails() {
    let mut p = make_solid_profile();
    p.mass = -1.0;
    let err = p.validate().unwrap_err();
    assert!(matches!(
        err,
        CoreError::ProfileInconsistent { field: "mass", .. }
    ));
}

#[test]
fn physics_profile_validate_friction_out_of_range_fails() {
    let mut p = make_solid_profile();
    p.friction = 2.0;
    let err = p.validate().unwrap_err();
    assert!(matches!(
        err,
        CoreError::ProfileInconsistent {
            field: "friction",
            ..
        }
    ));
}

#[test]
fn physics_profile_validate_restitution_out_of_range_fails() {
    let mut p = make_solid_profile();
    p.restitution = 1.5;
    let err = p.validate().unwrap_err();
    assert!(matches!(
        err,
        CoreError::ProfileInconsistent {
            field: "restitution",
            ..
        }
    ));
}

#[test]
fn physics_profile_validate_solver_iterations_zero_fails() {
    let mut p = make_solid_profile();
    p.solver_iterations = 0;
    let err = p.validate().unwrap_err();
    assert!(matches!(
        err,
        CoreError::ProfileInconsistent {
            field: "solver_iterations",
            ..
        }
    ));
}

#[test]
fn physics_profile_validate_density_zero_with_mass_fails() {
    let mut p = make_solid_profile();
    p.density = 0.0;
    let err = p.validate().unwrap_err();
    assert!(matches!(
        err,
        CoreError::ProfileInconsistent {
            field: "density",
            ..
        }
    ));
}

#[test]
fn physics_profile_validate_static_mass_zero_ok() {
    let p = PhysicsProfile::default_static();
    assert!(p.validate().is_ok());
}

#[test]
fn physics_profile_is_static_via_mass() {
    let mut p = make_solid_profile();
    assert!(!p.is_static());
    p.mass = 0.0;
    assert!(p.is_static());
    p.mass = 1.0;
    assert!(!p.is_static());
}

#[test]
fn physics_profile_inertia_diagonal_extracts_correct_components() {
    let p = make_solid_profile();
    let d = p.inertia_diagonal();
    assert!((d[0] - p.inertia[0]).abs() < f32::EPSILON);
    assert!((d[1] - p.inertia[4]).abs() < f32::EPSILON);
    assert!((d[2] - p.inertia[8]).abs() < f32::EPSILON);
}

#[test]
fn physics_profile_inverse_inertia_static_returns_zero() {
    let p = PhysicsProfile::default_static();
    let inv = p.inverse_inertia();
    assert!(inv[0].abs() < f32::EPSILON);
    assert!(inv[1].abs() < f32::EPSILON);
    assert!(inv[2].abs() < f32::EPSILON);
}

#[test]
fn physics_profile_inverse_inertia_dynamic_returns_reciprocals() {
    let p = make_solid_profile();
    let inv = p.inverse_inertia();
    let diag = p.inertia_diagonal();
    for i in 0..3 {
        assert!((inv[i] - 1.0 / diag[i]).abs() < 1e-6);
    }
}

// ============================================================================
// BodySpecBuilder
// ============================================================================

fn dummy_transform() -> InitialTransform {
    InitialTransform {
        translation: Vec3::new(0.0, 10.0, 0.0),
        rotation_yaw_pitch_roll: [0.0, 0.0, 0.0],
    }
}

#[test]
fn body_spec_builder_chained_call_succeeds() {
    let spec = BodySpec::builder()
        .shape(ShapeDesc::Sphere { radius: 0.5 })
        .transform(dummy_transform())
        .profile(PhysicsProfile::default_solid())
        .static_(false)
        .build()
        .expect("builder should succeed with all fields");
    assert!((spec.profile.mass - 1.0).abs() < f32::EPSILON);
    assert!(!spec.is_static);
    assert!(matches!(spec.shape, ShapeDesc::Sphere { radius } if (radius - 0.5).abs() < 1e-6));
}

#[test]
fn body_spec_builder_missing_shape_fails() {
    let err = BodySpec::builder()
        .transform(dummy_transform())
        .profile(PhysicsProfile::default_solid())
        .build()
        .unwrap_err();
    assert!(matches!(
        err,
        CoreError::BodySpecMissingField { field: "shape" }
    ));
}

#[test]
fn body_spec_builder_missing_transform_fails() {
    let err = BodySpec::builder()
        .shape(ShapeDesc::Sphere { radius: 0.5 })
        .profile(PhysicsProfile::default_solid())
        .build()
        .unwrap_err();
    assert!(matches!(
        err,
        CoreError::BodySpecMissingField {
            field: "initial_transform"
        }
    ));
}

#[test]
fn body_spec_builder_missing_profile_fails() {
    let err = BodySpec::builder()
        .shape(ShapeDesc::Sphere { radius: 0.5 })
        .transform(dummy_transform())
        .build()
        .unwrap_err();
    assert!(matches!(
        err,
        CoreError::BodySpecMissingField { field: "profile" }
    ));
}

#[test]
fn body_spec_builder_static_zero_mass_mismatch_fails() {
    // mass == 0 但 is_static == false → ProfileInconsistent
    let err = BodySpec::builder()
        .shape(ShapeDesc::Sphere { radius: 0.5 })
        .transform(dummy_transform())
        .profile(PhysicsProfile::default_static())
        .static_(false)
        .build()
        .unwrap_err();
    assert!(matches!(
        err,
        CoreError::ProfileInconsistent {
            field: "is_static",
            ..
        }
    ));
}

#[test]
fn body_spec_builder_alias_is_static() {
    let spec = BodySpec::builder()
        .shape(ShapeDesc::Sphere { radius: 0.5 })
        .transform(dummy_transform())
        .profile(PhysicsProfile::default_solid())
        .is_static(false) // alias for static_
        .build()
        .expect("alias method should work");
    assert!(!spec.is_static);
}

#[test]
fn body_spec_builder_direct_construction() {
    // 也支持 BodySpecBuilder::new() 直接构造（非 BodySpec::builder() 别名路径）
    let b: BodySpecBuilder = BodySpecBuilder::new();
    let spec = b
        .shape(ShapeDesc::Sphere { radius: 1.0 })
        .transform(dummy_transform())
        .profile(PhysicsProfile::default_solid())
        .build()
        .expect("direct new() should work");
    assert!(!spec.is_static);
}

// ============================================================================
// RuntimeDescriptor::validate / body / body_mut / counts / with_capacity
// ============================================================================

fn make_dynamic_body(mass: f32) -> BodySpec {
    BodySpec {
        shape: ShapeDesc::Sphere { radius: 0.5 },
        initial_transform: dummy_transform(),
        profile: PhysicsProfile {
            mass,
            ..PhysicsProfile::default_solid()
        },
        is_static: mass == 0.0,
    }
}

#[test]
fn runtime_descriptor_validate_empty_fails() {
    let d = RuntimeDescriptor::empty();
    let err = d.validate().unwrap_err();
    assert!(matches!(err, CoreError::DescriptorEmpty));
}

#[test]
fn runtime_descriptor_validate_one_valid_body_succeeds() {
    let mut d = RuntimeDescriptor::empty();
    d.add_body(make_dynamic_body(1.0));
    assert!(d.validate().is_ok());
}

#[test]
fn runtime_descriptor_validate_invalid_profile_fails() {
    let mut d = RuntimeDescriptor::empty();
    d.add_body(BodySpec {
        shape: ShapeDesc::Sphere { radius: 0.5 },
        initial_transform: dummy_transform(),
        profile: PhysicsProfile {
            friction: 5.0, // 超出 [0, 1]
            ..PhysicsProfile::default_solid()
        },
        is_static: false,
    });
    let err = d.validate().unwrap_err();
    assert!(matches!(
        err,
        CoreError::ProfileInconsistent {
            field: "friction",
            ..
        }
    ));
}

#[test]
fn runtime_descriptor_body_oob_returns_none() {
    let d = RuntimeDescriptor::empty();
    assert!(d.body(0).is_none());
    assert!(d.body(999).is_none());
}

#[test]
fn runtime_descriptor_body_mut_oob_returns_none() {
    let mut d = RuntimeDescriptor::empty();
    assert!(d.body_mut(0).is_none());
    let mut d = RuntimeDescriptor::with_capacity(1);
    d.add_body(make_dynamic_body(1.0));
    assert!(d.body_mut(0).is_some());
    assert!(d.body_mut(1).is_none());
}

#[test]
fn runtime_descriptor_static_and_dynamic_counts() {
    let mut d = RuntimeDescriptor::with_capacity(3);
    d.add_body(make_dynamic_body(1.0));
    d.add_body(make_dynamic_body(2.0));
    d.add_body(make_dynamic_body(0.0)); // static
    assert_eq!(d.static_body_count(), 1);
    assert_eq!(d.dynamic_body_count(), 2);
    assert_eq!(d.body_count(), 3);
}

#[test]
fn runtime_descriptor_with_capacity_preserves_capacity() {
    let mut d = RuntimeDescriptor::with_capacity(5);
    for i in 0..5 {
        d.add_body(make_dynamic_body((i + 1) as f32));
    }
    // 推 5 个后不应发生 realloc
    assert!(
        d.bodies.capacity() >= 5,
        "capacity should be >= 5, got {}",
        d.bodies.capacity()
    );
    assert_eq!(d.body_count(), 5);
}

// ============================================================================
// from_raw
// ============================================================================

#[test]
fn body_handle_from_raw_equivalent_to_new() {
    let a = BodyHandle::new(7, 13);
    let b = BodyHandle::from_raw(7, 13);
    assert_eq!(a, b);
    assert_eq!(a.index, 7);
    assert_eq!(a.generation, 13);
}

#[test]
fn constraint_handle_from_raw_equivalent_to_new() {
    let a = ConstraintHandle::new(11, 22);
    let b = ConstraintHandle::from_raw(11, 22);
    assert_eq!(a, b);
}

#[test]
fn island_handle_from_raw_equivalent_to_new() {
    let a = IslandHandle::new(42);
    let b = IslandHandle::from_raw(42);
    assert_eq!(a, b);
}

// ============================================================================
// CoreError 新变体 Display
// ============================================================================

#[test]
fn core_error_new_variants_display_not_empty() {
    let e1 = CoreError::BodyIndexOutOfBounds { index: 3, len: 2 };
    let s1 = format!("{e1}");
    assert!(!s1.is_empty());
    assert!(s1.contains('3'));
    assert!(s1.contains('2'));

    let e2 = CoreError::DuplicateBodyIndex { index: 7 };
    let s2 = format!("{e2}");
    assert!(!s2.is_empty());
    assert!(s2.contains('7'));

    let e3 = CoreError::BodySpecMissingField { field: "shape" };
    let s3 = format!("{e3}");
    assert!(!s3.is_empty());
    assert!(s3.contains("shape"));
}
