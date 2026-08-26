//! `gvpe-core` 单元测试。

use crate::{
    BodyHandle, BodySpec, ConstraintHandle, CoreError, InitialTransform, IslandHandle,
    PhysicsLodTag, PhysicsProfile, RuntimeDescriptor, ShapeDesc, SolverTypeId,
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
    assert!(p.mass.abs() < 1e-6, "static profile mass should be 0.0");
}

// ============================================================================
// RuntimeDescriptor 测试
// ============================================================================

#[test]
fn runtime_descriptor_empty() {
    let d = RuntimeDescriptor::empty();
    assert_eq!(d.body_count(), 0);
    assert!((d.gravity.y - (-9.81)).abs() < 1e-6);
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
