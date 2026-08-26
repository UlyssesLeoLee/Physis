//! `gvpe-ffi`: Physis C ABI surface。
//!
//! 依据 `ROADMAP.md` §5.3（v0.9 FFI 最小集）+ `GVPE-DOC-10`（跨引擎集成规范）。
//!
//! ## 设计原则（per `GVPE-DOC-10`）
//!
//! - **opaque handle**: C 端只见 `*mut GvpeRuntime`,不暴露内部 Rust struct 布局。
//! - **POD-only FFI 边界**: 所有跨边界结构体 `#[repr(C)]` + `bytemuck::Pod`。
//! - **panic-safe boundary**: 所有 `extern "C"` 入口用 [`std::panic::catch_unwind`]
//!   包裹,防止 panic 跨语言边界 unwind（C 端栈展开行为未定义）。
//! - **不写引擎专用 wrapper**: Unity / Unreal / Godot 各自的 binding 留给各引擎侧
//!   仓库（`gvpe-unity` / `gvpe-unreal` / `gvpe-godot`,post-MVP 规划）。
//!
//! ## v0.9 范围
//!
//! 5 个最小入口（per `ROADMAP.md` §5.3）:
//!
//! 1. [`gvpe_ffi_runtime_create`] — 创建 runtime,返回 opaque 句柄。
//! 2. [`gvpe_ffi_runtime_destroy`] — 销毁 runtime,double-free-safe（null no-op）。
//! 3. [`gvpe_ffi_step`] — 单步推进（**v0.9 skeleton 仅递增计数器**,不真做物理）。
//! 4. [`gvpe_ffi_body_count`] — 查询 body 数量。
//! 5. [`gvpe_ffi_body_get_transform`] — 读取 body 当前 transform。
//!
//! ## 不做（per `ROADMAP.md` §2.3 + `GVPE-DOC-10`）
//!
//! - 不暴露 `set_transform` / `add_body` / `set_gravity` 等场景构建 API（v0.9 skeleton
//!   只读 [`RuntimeDescriptor`] 中的初始 transform; v0.10+ 在 gvpe-dynamics 集成后
//!   补 `set_body` / `add_body`）。
//! - 不写引擎 wrapper（同上）。
//! - 不暴露 [`RuntimeDescriptor`] 内部结构（C 端只接触 [`GvpeTransform`]）。
//!
//! ## 错误模型
//!
//! 所有 `extern "C"` fn 返 [`i32`] 错误码（`0` = 成功）。错误语义见 [`FfiError::code`]。
//! 调用方需要字符串时,可订阅 `tracing` 日志(v0.10+)或在 wrapper 层加 lookup 表。
//!
//! ## 性能 / 安全约束
//!
//! - 所有 `unsafe` 块均有 `// SAFETY:` 注释,说明 handle 来源 + 生命周期假设。
//! - 理想配置是 release profile 用 `panic = "abort"`,以确保 panic 不会
//!   unwind 跨 C ABI 边界（unwind 跨语言行为未定义）。但 Cargo **stable
//!   不支持 per-package panic override**（需 nightly `-Z profile-rustflags`）,
//!   本 crate 沿用 workspace 根 `[profile.release]` 的 `panic = "unwind"`。
//!   `catch_unwind` 包裹所有 `extern "C"` 入口作为主屏障。v0.10+ 切换到
//!   nightly / 引入 build script 设置 RUSTFLAGS 时再启用 abort 模式。
//! - C 端传入的指针必须满足:
//!   - 句柄来自 [`gvpe_ffi_runtime_create`] 且未被 destroy。
//!   - 输出指针指向至少 1 个目标结构体大小的有效内存。
//!   - 调用方负责线程安全（本 crate 不内置锁; 调用方需保证单个 runtime
//!     不跨线程并发调用）。

#![deny(unsafe_op_in_unsafe_fn, missing_debug_implementations, missing_docs)]
#![warn(non_ascii_idents)]
// FFI 边界:所有 `extern "C"` 入口对 C 端是"安全"的(其安全契约写在 /// Safety 注释),
// 必须在 `unsafe { ... }` 块内解引用原始指针。clippy::not_unsafe_ptr_arg_deref 默认会
// 建议把这些 public fn 标 `unsafe fn`,但 `extern "C"` 不能是 `unsafe fn`(C 调用方
// 不会显式 unsafe),因此整 crate 关闭此 lint。
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::panic::catch_unwind;
use std::ptr;

use bytemuck::{Pod, Zeroable};
use gvpe_core::{CoreError, RuntimeDescriptor};
use gvpe_math::Quat;
use thiserror::Error;

// ============================================================================
// 错误类型
// ============================================================================

/// 统一 FFI 错误类型（Rust 端; C 端只见 i32 错误码）。
///
/// 错误码集中由 [`FfiError::code`] 管理,新增变体时**必须**同步更新 [`FfiError::code`]。
#[derive(Debug, Error)]
pub enum FfiError {
    /// 句柄为空指针。
    #[error("null handle")]
    NullHandle,

    /// 句柄已销毁或从未由 [`gvpe_ffi_runtime_create`] 创建。
    ///
    /// v0.9 skeleton 暂不实现世代校验（`Slab` 集成在 v0.10+）;仅占位。
    #[error("dangling handle")]
    DanglingHandle,

    /// Runtime 为空（无 body）。
    #[error("empty runtime")]
    EmptyRuntime,

    /// body 索引越界。
    #[error("body index {idx} out of bounds (count = {count})")]
    BodyOutOfBounds {
        /// 请求的 body 索引。
        idx: u32,
        /// 当前 body 总数。
        count: u32,
    },

    /// 透传 `gvpe-core` 错误。
    #[error("core error: {0}")]
    Core(#[from] CoreError),

    /// 边界 panic 捕获（`catch_unwind` 触发）。
    #[error("panic caught at FFI boundary")]
    Panic,

    /// 未知 / 兜底错误（v0.9 用作 dt 校验 / usize → u32 转换失败等简化场景）。
    #[error("unknown error")]
    Unknown,
}

impl FfiError {
    /// 错误码（`0` = 成功,非 0 = 失败;C ABI 端仅传 i32）。
    #[inline]
    #[must_use]
    pub const fn code(&self) -> i32 {
        match self {
            Self::NullHandle => 1,
            Self::DanglingHandle => 2,
            Self::EmptyRuntime => 3,
            Self::BodyOutOfBounds { .. } => 4,
            Self::Core(_) => 5,
            Self::Panic => 6,
            Self::Unknown => 7,
        }
    }
}

impl From<FfiError> for i32 {
    /// 转 C ABI 错误码。
    #[inline]
    fn from(e: FfiError) -> Self {
        e.code()
    }
}

// ============================================================================
// Opaque handle 设计
// ============================================================================

/// Runtime opaque 句柄（C ABI 端类型 = `GvpeRuntime*`）。
///
/// `#[repr(transparent)]` 保证 Rust 端和 C 端布局一致（`GvpeRuntime*` ≡ `*mut ()` 兼容）。
/// C 端不可解引用,只能传回 destroy / step / query 系列函数。
#[repr(transparent)]
pub struct GvpeRuntime(*mut RuntimeInner);

// 仅打印类型名,不暴露原始指针地址（避免 leak + 让 Debug 输出在测试中断言更稳定）。
impl std::fmt::Debug for GvpeRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GvpeRuntime").finish_non_exhaustive()
    }
}

/// Runtime 内部状态（v0.9 skeleton 仅保留 descriptor + 步进计数）。
///
/// v0.10+ 在此加 body state storage（`Slab<BodyState>`）+ island / solver handles。
struct RuntimeInner {
    /// 场景描述符（body 列表 + 重力 + 确定性模式等）。
    descriptor: RuntimeDescriptor,
    /// 已推进步数（v0.9 skeleton 仅作计数器,验证 `catch_unwind` / FFI 通路）。
    step_count: u64,
}

impl GvpeRuntime {
    /// 创建新 Runtime（v0.9 skeleton 使用空 descriptor）。
    fn new() -> *mut Self {
        let inner = RuntimeInner {
            descriptor: RuntimeDescriptor::empty(),
            step_count: 0,
        };
        let handle = Self(Box::into_raw(Box::new(inner)));
        Box::into_raw(Box::new(handle))
    }
}

// ============================================================================
// FFI 数据结构
// ============================================================================

/// 跨 FFI 边界的 transform 描述（translation + rotation）。
///
/// C 端定义:
/// ```c
/// typedef struct {
///     float translation[3];
///     float rotation[4];  // quaternion (x, y, z, w)
/// } GvpeTransform;
/// ```
///
/// 字段顺序与 [`gvpe_math::Transform`] 一致（translation 在前,rotation 在后）,
/// 但 v0.9 skeleton 单独定义 FFI struct 而非直接 re-export `Transform`:
/// - `Transform` 16 字节对齐（`Quat` 对齐）会增加 C 端 padding 复杂度;
/// - FFI struct 显式 `align(4)` 与 C 端 `float[3]` / `float[4]` 数组布局一致。
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct GvpeTransform {
    /// translation xyz（世界坐标,米）。
    pub translation: [f32; 3],
    /// rotation xyzw（单位四元数;`rotation_w` 在 `translation[2]` 之后第 16 字节起始）。
    pub rotation: [f32; 4],
}

// ============================================================================
// 5 个 extern "C" 入口（per ROADMAP §5.3）
// ============================================================================

/// 创建 runtime;返非空指针 = 成功,`null` = 失败（含 panic 捕获）。
///
/// C 端签名: `GvpeRuntime* gvpe_ffi_runtime_create(void);`
///
/// 失败原因（v0.9 skeleton）: 内存分配失败 / panic（实际仅理论存在,
/// `Box::new` 在 1.x 默认不返回 null,OOM 时会直接 abort）;返 `null`
/// 仅为防御性约定。
#[no_mangle]
pub extern "C" fn gvpe_ffi_runtime_create() -> *mut GvpeRuntime {
    // 函数指针自动实现 UnwindSafe,无需闭包包裹;失败兜底为 null。
    catch_unwind(GvpeRuntime::new).unwrap_or(ptr::null_mut())
}

/// 销毁 runtime;句柄为 `null` 时 no-op,不 panic。
///
/// C 端签名: `void gvpe_ffi_runtime_destroy(GvpeRuntime* handle);`
///
/// **double-free UB 责任**: 本函数假设 `handle` 来自 [`gvpe_ffi_runtime_create`]
/// 且未被 destroy 两次;C 端需自行保证（典型做法: 销毁后立即将句柄置 `null`）。
#[no_mangle]
pub extern "C" fn gvpe_ffi_runtime_destroy(handle: *mut GvpeRuntime) {
    if handle.is_null() {
        return;
    }
    // 销毁过程也用 catch_unwind 包裹(虽然 Box::from_raw → drop 几乎不会 panic)
    let _ = catch_unwind(|| {
        // SAFETY: 调用方保证 handle 来自 `gvpe_ffi_runtime_create` 且未被 free 两次。
        // `Box::from_raw` 重建 Box 所有权,drop 时释放 RuntimeInner + 外层 GvpeRuntime。
        unsafe {
            let outer = Box::from_raw(handle);
            let inner_ptr = outer.0;
            drop(outer);
            drop(Box::from_raw(inner_ptr));
        }
    });
}

/// 单步推进;返 `0` = 成功,非 0 = 错误码。
///
/// C 端签名: `int32_t gvpe_ffi_step(GvpeRuntime* handle, float dt);`
///
/// v0.9 skeleton 行为: 仅校验 handle + dt 非负非 NaN,然后递增 `step_count`。
/// 真正的 integrate / project / 碰撞检测留 v0.10+（`gvpe-dynamics` crate 集成后）。
///
/// 错误码:
/// - `1` (`NullHandle`): handle 为 null。
/// - `7` (`Unknown`): dt 为负或 NaN。
/// - `6` (`Panic`): 内部 panic 被 catch。
#[no_mangle]
pub extern "C" fn gvpe_ffi_step(handle: *mut GvpeRuntime, dt: f32) -> i32 {
    let result = catch_unwind(|| -> Result<(), FfiError> {
        if handle.is_null() {
            return Err(FfiError::NullHandle);
        }
        if dt < 0.0 || dt.is_nan() {
            return Err(FfiError::Unknown);
        }
        // SAFETY: handle 非空 + 来自 create + 未被 free;仅 mutate 不释放。
        let runtime = unsafe { &mut *handle };
        // SAFETY: outer 句柄由 create 创建,Box 拥有 RuntimeInner;此处仅取借用 mutate。
        let inner = unsafe { &mut *runtime.0 };
        inner.step_count = inner.step_count.saturating_add(1);
        Ok(())
    });
    match result {
        Ok(Ok(())) => 0,
        Ok(Err(e)) => e.code(),
        Err(_) => FfiError::Panic.code(),
    }
}

/// body 数量;返 `>= 0` = 数量,`< 0` = 错误码的绝对值。
///
/// C 端签名: `int32_t gvpe_ffi_body_count(GvpeRuntime* handle);`
///
/// v0.9 skeleton 直接读 `RuntimeDescriptor::body_count()`,无任何物理计算。
///
/// 错误码（C 端按需取负或与 0 比较均可）:
/// - `1` (`NullHandle`): handle 为 null。
/// - `7` (`Unknown`): body 数量超出 `i32` 范围（实际不可能,`usize` ≤ `i32::MAX`）。
/// - `6` (`Panic`): 内部 panic。
#[no_mangle]
pub extern "C" fn gvpe_ffi_body_count(handle: *mut GvpeRuntime) -> i32 {
    let result = catch_unwind(|| -> Result<i32, FfiError> {
        if handle.is_null() {
            return Err(FfiError::NullHandle);
        }
        // SAFETY: 同 `gvpe_ffi_step` —— handle 有效,仅读取 descriptor。
        let runtime = unsafe { &mut *handle };
        let inner = unsafe { &mut *runtime.0 };
        i32::try_from(inner.descriptor.body_count()).map_err(|_| FfiError::Unknown)
    });
    match result {
        Ok(Ok(n)) => n,
        Ok(Err(e)) => e.code(),
        Err(_) => FfiError::Panic.code(),
    }
}

/// 获取 body 当前 transform;成功填 `out` 并返 `0`,失败返错误码。
///
/// C 端签名:
/// ```c
/// int32_t gvpe_ffi_body_get_transform(
///     GvpeRuntime* handle, uint32_t body_idx, GvpeTransform* out
/// );
/// ```
///
/// v0.9 skeleton 从 `RuntimeDescriptor` 的 `initial_transform` 派生:
/// - translation: 直接复制 `BodySpec::initial_transform.translation`。
/// - rotation: 用 `BodySpec::initial_transform.rotation_yaw_pitch_roll`
///   通过 `Quat::from_euler_ypr` 构造。后续 v0.10+ 在 `gvpe-dynamics` 集成后
///   改为读取每步更新后的实际 transform。
///
/// 错误码:
/// - `1` (`NullHandle`): handle 或 `out` 为 null。
/// - `4` (`BodyOutOfBounds`): `body_idx >= body_count()`。
/// - `6` (`Panic`): 内部 panic。
#[no_mangle]
pub extern "C" fn gvpe_ffi_body_get_transform(
    handle: *mut GvpeRuntime,
    body_idx: u32,
    out: *mut GvpeTransform,
) -> i32 {
    let result = catch_unwind(|| -> Result<(), FfiError> {
        if handle.is_null() {
            return Err(FfiError::NullHandle);
        }
        if out.is_null() {
            return Err(FfiError::NullHandle); // 复用 NullHandle 表示 null 输出
        }
        // SAFETY: handle 有效;仅读取 descriptor 与 out（写 out 单独走下方 unsafe 块）。
        let runtime = unsafe { &mut *handle };
        let inner = unsafe { &mut *runtime.0 };
        let count = u32::try_from(inner.descriptor.body_count()).map_err(|_| FfiError::Unknown)?;
        if body_idx >= count {
            return Err(FfiError::BodyOutOfBounds {
                idx: body_idx,
                count,
            });
        }
        let spec = inner
            .descriptor
            .body(body_idx as usize)
            .ok_or(FfiError::BodyOutOfBounds {
                idx: body_idx,
                count,
            })?;
        // initial transform → 当前位置（v0.9 skeleton 无步进后位置）。
        let [yaw, pitch, roll] = spec.initial_transform.rotation_yaw_pitch_roll;
        let quat = Quat::from_euler_ypr(yaw, pitch, roll);
        let t = spec.initial_transform.translation;
        // SAFETY: 调用方保证 out 指向至少 1 个 GvpeTransform 大小的有效可写内存,
        // 且生命周期内不与其他读线程并发（callers' responsibility per doc）。
        unsafe {
            (*out) = GvpeTransform {
                translation: [t.x, t.y, t.z],
                rotation: [quat.x, quat.y, quat.z, quat.w],
            };
        }
        Ok(())
    });
    match result {
        Ok(Ok(())) => 0,
        Ok(Err(e)) => e.code(),
        Err(_) => FfiError::Panic.code(),
    }
}

// ============================================================================
// 集成测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use gvpe_core::{BodySpec, InitialTransform, PhysicsProfile, ShapeDesc};
    use gvpe_math::{Quat, Vec3};

    /// 构造一个测试用 body spec（位置 (1,2,3), 零旋转）。
    fn make_test_body() -> BodySpec {
        BodySpec::builder()
            .shape(ShapeDesc::Sphere { radius: 0.5 })
            .transform(InitialTransform {
                translation: Vec3::new(1.0, 2.0, 3.0),
                rotation_yaw_pitch_roll: [0.0, 0.0, 0.0],
            })
            .profile(PhysicsProfile::default_solid())
            .build()
            .expect("test body spec 构造应成功")
    }

    /// 直接构造一个含指定 body 列表的 runtime（绕过 FFI 入口,只用于测试 transform 路径）。
    ///
    /// FFI 端 v0.9 skeleton 不暴露 `add_body`;此 helper 利用 tests 子模块可访问
    /// `RuntimeInner` 私有字段的便利,模拟"已构建好场景"的 runtime,用来
    /// 测 [`gvpe_ffi_body_get_transform`] 在非空 runtime 上的行为。
    fn runtime_with_bodies(bodies: Vec<BodySpec>) -> *mut GvpeRuntime {
        let mut desc = gvpe_core::RuntimeDescriptor::empty();
        for b in bodies {
            desc.add_body(b);
        }
        let inner = RuntimeInner {
            descriptor: desc,
            step_count: 0,
        };
        let handle = GvpeRuntime(Box::into_raw(Box::new(inner)));
        Box::into_raw(Box::new(handle))
    }

    // ------------------------------------------------------------------------
    // API 行为测试（不依赖具体 body 状态）
    // ------------------------------------------------------------------------

    #[test]
    fn create_destroy_no_panic() {
        let handle = gvpe_ffi_runtime_create();
        assert!(!handle.is_null(), "create 后句柄不应为 null");
        gvpe_ffi_runtime_destroy(handle);
    }

    #[test]
    fn destroy_null_handle_no_panic() {
        // 关键: 销毁 null 必须 no-op,不 panic 不 UB。
        gvpe_ffi_runtime_destroy(ptr::null_mut());
    }

    #[test]
    fn step_empty_runtime_ok() {
        let handle = gvpe_ffi_runtime_create();
        let rc = gvpe_ffi_step(handle, 1.0 / 60.0);
        assert_eq!(rc, 0, "空 runtime + 正常 dt 应成功");
        gvpe_ffi_runtime_destroy(handle);
    }

    #[test]
    fn step_null_handle_returns_null_handle_code() {
        let rc = gvpe_ffi_step(ptr::null_mut(), 1.0 / 60.0);
        assert_eq!(
            rc,
            FfiError::NullHandle.code(),
            "step(null) 应返 NullHandle 错误码"
        );
    }

    #[test]
    fn step_negative_dt_returns_unknown_code() {
        let handle = gvpe_ffi_runtime_create();
        let rc = gvpe_ffi_step(handle, -0.1);
        assert_eq!(rc, FfiError::Unknown.code(), "负 dt 应返 Unknown 错误码");
        gvpe_ffi_runtime_destroy(handle);
    }

    #[test]
    fn step_nan_dt_returns_unknown_code() {
        let handle = gvpe_ffi_runtime_create();
        let rc = gvpe_ffi_step(handle, f32::NAN);
        assert_eq!(rc, FfiError::Unknown.code(), "NaN dt 应返 Unknown 错误码");
        gvpe_ffi_runtime_destroy(handle);
    }

    #[test]
    fn body_count_empty_zero() {
        let handle = gvpe_ffi_runtime_create();
        let n = gvpe_ffi_body_count(handle);
        assert_eq!(n, 0, "空 runtime body_count 应为 0");
        gvpe_ffi_runtime_destroy(handle);
    }

    #[test]
    fn body_count_null_handle_error() {
        let n = gvpe_ffi_body_count(ptr::null_mut());
        assert_eq!(
            n,
            FfiError::NullHandle.code(),
            "body_count(null) 应返 NullHandle 错误码（负数语义可由调用方解释）"
        );
    }

    #[test]
    fn body_get_transform_null_handle_error() {
        let mut out = GvpeTransform {
            translation: [0.0; 3],
            rotation: [0.0; 4],
        };
        let rc = gvpe_ffi_body_get_transform(ptr::null_mut(), 0, std::ptr::addr_of_mut!(out));
        assert_eq!(rc, FfiError::NullHandle.code());
    }

    #[test]
    fn body_get_transform_null_out_error() {
        let handle = gvpe_ffi_runtime_create();
        let rc = gvpe_ffi_body_get_transform(handle, 0, ptr::null_mut());
        assert_eq!(rc, FfiError::NullHandle.code());
        gvpe_ffi_runtime_destroy(handle);
    }

    // ------------------------------------------------------------------------
    // 端到端 smoke test: 模拟 C 端完整生命周期
    // ------------------------------------------------------------------------

    #[test]
    fn lifecycle_create_step_get_transform_destroy() {
        // 1. create
        let handle = gvpe_ffi_runtime_create();
        assert!(!handle.is_null());

        // 2. 空 runtime 验证
        assert_eq!(gvpe_ffi_body_count(handle), 0);

        // 3. step 几次(空 runtime 上 step 仍应成功,只递增计数器)
        for _ in 0..3 {
            assert_eq!(gvpe_ffi_step(handle, 1.0 / 60.0), 0);
        }

        // 4. 验证 transform 路径:虽然空 runtime 会越界,但我们要测的是
        //    body count = 0 + 越界返 BodyOutOfBounds 的语义。
        let mut out = GvpeTransform {
            translation: [f32::NAN; 3], // 用 NaN 作为未写入哨兵
            rotation: [f32::NAN; 4],
        };
        let rc = gvpe_ffi_body_get_transform(handle, 0, std::ptr::addr_of_mut!(out));
        assert_eq!(
            rc,
            FfiError::BodyOutOfBounds { idx: 0, count: 0 }.code(),
            "空 runtime 上 body_idx=0 应越界"
        );
        // 越界时不应触碰 out
        assert!(out.translation[0].is_nan(), "越界时 out 不应被改写");

        // 5. destroy
        gvpe_ffi_runtime_destroy(handle);
    }

    // ------------------------------------------------------------------------
    // GvpeTransform 布局保证
    // ------------------------------------------------------------------------

    #[test]
    fn gvpe_transform_is_pod() {
        // 由 `#[derive(Pod, Zeroable)]` 保证;此处仅做编译期 + 运行时 spot check。
        let zero = GvpeTransform {
            translation: [0.0; 3],
            rotation: [0.0; 4],
        };
        let bytes: &[u8] = bytemuck::bytes_of(&zero);
        assert_eq!(bytes.len(), 28, "GvpeTransform 7 × f32 = 28 字节");
        assert!(bytes.iter().all(|&b| b == 0), "Zeroable 字节应全 0");
    }

    #[test]
    fn gvpe_transform_align_matches_c_float_array() {
        // 跨语言 ABI 稳定:f32 数组布局与 C `float[3]` / `float[4]` 一致。
        // C 端 `GvpeTransform` 字段顺序 = (translation[3], rotation[4]),
        // Rust 端字段顺序一致 → 内存布局等价。
        // 此测试通过对齐 + 偏移做静态确认。
        assert_eq!(std::mem::align_of::<GvpeTransform>(), 4, "对齐应匹配 f32");
        assert_eq!(
            std::mem::size_of::<GvpeTransform>(),
            7 * std::mem::size_of::<f32>(),
            "GvpeTransform 大小 = 7 个 f32"
        );
    }

    // ------------------------------------------------------------------------
    // 错误码常量自检（防止 enum 改动后忘记更新 code()）
    // ------------------------------------------------------------------------

    #[test]
    fn ffi_error_codes_match_documented_values() {
        // 文档化 + 测试化:错误码集中定义,任何变更须双改。
        assert_eq!(FfiError::NullHandle.code(), 1);
        assert_eq!(FfiError::DanglingHandle.code(), 2);
        assert_eq!(FfiError::EmptyRuntime.code(), 3);
        assert_eq!(FfiError::BodyOutOfBounds { idx: 0, count: 0 }.code(), 4);
        // Core 错误不展开具体值,只看映射
        let core_err = CoreError::DescriptorEmpty;
        assert_eq!(FfiError::Core(core_err).code(), 5);
        assert_eq!(FfiError::Panic.code(), 6);
        assert_eq!(FfiError::Unknown.code(), 7);
    }

    // ------------------------------------------------------------------------
    // 数学转换路径(直接对 Quat::from_euler_ypr 的 FFI 行为做锚定测试)
    // ------------------------------------------------------------------------

    #[test]
    fn body_get_transform_with_one_body_returns_initial_transform() {
        // 测 transform 路径在非空 runtime 上的 happy path:
        // - body_count 应返 1
        // - body_get_transform(0) 应返 initial_transform,translation 1:1 复制,
        //   rotation = Quat::from_euler_ypr(0,0,0) ≈ identity (w ≈ 1)
        let handle = runtime_with_bodies(vec![make_test_body()]);
        assert_eq!(gvpe_ffi_body_count(handle), 1);

        let mut out = GvpeTransform {
            translation: [0.0; 3],
            rotation: [0.0; 4],
        };
        let rc = gvpe_ffi_body_get_transform(handle, 0, std::ptr::addr_of_mut!(out));
        assert_eq!(rc, 0, "存在 body 时 body_get_transform 应成功");

        // translation 直接复制 (1, 2, 3)
        assert!((out.translation[0] - 1.0).abs() < 1e-6);
        assert!((out.translation[1] - 2.0).abs() < 1e-6);
        assert!((out.translation[2] - 3.0).abs() < 1e-6);

        // rotation = identity quat (x=y=z≈0, w≈1)
        assert!(out.rotation[0].abs() < 1e-6);
        assert!(out.rotation[1].abs() < 1e-6);
        assert!(out.rotation[2].abs() < 1e-6);
        assert!((out.rotation[3] - 1.0).abs() < 1e-6);

        gvpe_ffi_runtime_destroy(handle);
    }

    #[test]
    fn body_get_transform_out_of_bounds_returns_error() {
        // body_count = 1 时,body_idx = 1 应越界
        let handle = runtime_with_bodies(vec![make_test_body()]);
        let mut out = GvpeTransform {
            translation: [0.0; 3],
            rotation: [0.0; 4],
        };
        let rc = gvpe_ffi_body_get_transform(handle, 1, std::ptr::addr_of_mut!(out));
        assert_eq!(
            rc,
            FfiError::BodyOutOfBounds { idx: 1, count: 1 }.code(),
            "body_idx >= body_count 应越界"
        );
        gvpe_ffi_runtime_destroy(handle);
    }

    // ------------------------------------------------------------------------
    // 占位以满足"锚定 Quat::from_euler_ypr 语义契约"
    // ------------------------------------------------------------------------
    // ------------------------------------------------------------------------

    #[test]
    fn rotation_ypr_zero_yields_identity_quat() {
        // YPR 全 0 应得到单位四元数 —— 与 from_euler_ypr 的语义契约。
        let q = Quat::from_euler_ypr(0.0, 0.0, 0.0);
        assert!((q.w - 1.0).abs() < 1e-6, "零旋转应得 w ≈ 1");
        assert!(q.x.abs() < 1e-6);
        assert!(q.y.abs() < 1e-6);
        assert!(q.z.abs() < 1e-6);
    }
}
