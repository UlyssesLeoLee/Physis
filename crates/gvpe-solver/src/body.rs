//! `RigidBody`：刚体运行时状态。
//!
//! 依据 [`GVPE-DOC-04`] §4.5（Law → Model → Solver 可追溯表）的 `RigidBodyModel` 落地。
//! 求解器运行时所需的最小 body 状态集：质量、惯量、速度、位姿、sleep 标志。
//!
//! MVP 简化：
//! - 惯性张量按对角矩阵存储（`[I_xx, I_yy, I_zz]`），假设 body 主轴与世界轴对齐（无旋转惯量耦合）；
//!   真实斜对角张量走 gvpe-math `Mat3` 在 v0.4+ 扩展。
//! - 不存引力 / 累计外力 —— 由调用方在 `integrate` 前 push 到 `lin_acc` / `ang_acc`。
//!
//! [`GVPE-DOC-04`]: ../../../docs/01_architecture/04_architecture.md

use gvpe_core::BodyHandle;
use gvpe_math::{Quat, Transform, Vec3};
use gvpe_memory::Slab;

/// 刚体运行时状态。
///
/// 字段布局（求解器访问频率从高到低）：
/// - `lin_vel` / `ang_vel`：热路径，SI 循环每行访问。
/// - `inv_mass` / `inv_inertia_diag`：热路径。
/// - `position` / `rotation`：积分后写入。
/// - `sleep` / `frames_below_threshold`：warm path（每帧 1 次）。
#[derive(Clone, Debug, PartialEq)]
pub struct RigidBody {
    /// 句柄（与 slab 槽对应）。
    pub handle: BodyHandle,
    /// 位姿。
    pub position: Vec3,
    /// 旋转（围绕 body 中心）。
    pub rotation: Quat,
    /// 线速度（世界系）。
    pub lin_vel: Vec3,
    /// 角速度（世界系，rad/s）。
    pub ang_vel: Vec3,
    /// 质量（kg）；`0` = 静态体。
    pub mass: f32,
    /// 质量倒数；静态体为 0。
    pub inv_mass: f32,
    /// 主轴对齐对角惯性张量倒数 `[I_xx^-1, I_yy^-1, I_zz^-1]`。
    /// 静态体为 `[0, 0, 0]`。
    pub inv_inertia_diag: [f32; 3],
    /// 累计外力（世界系）；积分前由调用方 push，积分后清零。
    pub accumulated_force: Vec3,
    /// 累计外力矩（世界系）。
    pub accumulated_torque: Vec3,
    /// 当前 sleep 状态。
    pub sleep: SleepState,
    /// 连续低于阈值的帧数（仅 active 状态递增）。
    pub frames_below_threshold: u32,
    /// 外部索引（slab slot 的 index）；与 `handle.index` 同值，作缓存。
    pub slot_index: u32,
}

impl RigidBody {
    /// 构造动态刚体。
    #[inline]
    #[must_use]
    pub fn new_dynamic(handle: BodyHandle, slot_index: u32, mass: f32, inertia_diag: [f32; 3]) -> Self {
        let inv_mass = if mass > 0.0 { 1.0 / mass } else { 0.0 };
        let inv_inertia = [
            if inertia_diag[0] > 0.0 {
                1.0 / inertia_diag[0]
            } else {
                0.0
            },
            if inertia_diag[1] > 0.0 {
                1.0 / inertia_diag[1]
            } else {
                0.0
            },
            if inertia_diag[2] > 0.0 {
                1.0 / inertia_diag[2]
            } else {
                0.0
            },
        ];
        Self {
            handle,
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            lin_vel: Vec3::ZERO,
            ang_vel: Vec3::ZERO,
            mass,
            inv_mass,
            inv_inertia_diag: inv_inertia,
            accumulated_force: Vec3::ZERO,
            accumulated_torque: Vec3::ZERO,
            sleep: SleepState::Active,
            frames_below_threshold: 0,
            slot_index,
        }
    }

    /// 构造静态体（`inv_mass = 0` / `inv_inertia = 0`）；永不睡眠。
    #[inline]
    #[must_use]
    pub fn new_static(handle: BodyHandle, slot_index: u32) -> Self {
        Self {
            handle,
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            lin_vel: Vec3::ZERO,
            ang_vel: Vec3::ZERO,
            mass: 0.0,
            inv_mass: 0.0,
            inv_inertia_diag: [0.0; 3],
            accumulated_force: Vec3::ZERO,
            accumulated_torque: Vec3::ZERO,
            sleep: SleepState::Static,
            frames_below_threshold: 0,
            slot_index,
        }
    }

    /// 是否为静态体（`inv_mass == 0`）。
    #[inline]
    #[must_use]
    pub fn is_static(&self) -> bool {
        self.inv_mass == 0.0
    }

    /// 当前 `Transform`。
    #[inline]
    #[must_use]
    pub fn transform(&self) -> Transform {
        Transform::new(self.position, self.rotation)
    }

    /// 给定世界系向量 `w` 在 body 局部系下表示（用 `rotation.conjugate()` 反旋）。
    #[inline]
    #[must_use]
    pub fn world_to_local_vec(&self, w: Vec3) -> Vec3 {
        self.rotation.conjugate().rotate_vec3(w)
    }
}

/// 刚体 sleep 状态。
///
/// 依据 [`GVPE-DOC-07`] §6.4：连续 N 帧速度低于阈值 → `Sleeping`；
/// `Sleeping` body 从 island 活跃计数中排除，直到新接触 / 新外力唤醒。
///
/// 语义（per 02_physics_ontology.md §6）：`Sleeping` 是 `State` 而非永久 `Property`。
///
/// [`GVPE-DOC-07`]: ../../../docs/02_modules/07_solver_design.md
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SleepState {
    /// 静态体（`inv_mass == 0`），永不睡眠，永不积分位移。
    Static,
    /// 活跃体，参与求解。
    Active,
    /// 睡眠体：跳过积分、跳过求解，岛内计数排除。
    Sleeping,
}

impl Default for SleepState {
    fn default() -> Self {
        Self::Active
    }
}

/// `BodySlab`：body 池。`Slab<RigidBody>` 的薄封装，提供 `iter` / `iter_active` 视图。
#[derive(Debug)]
pub struct BodySlab {
    slab: Slab<RigidBody>,
}

impl Default for BodySlab {
    fn default() -> Self {
        Self::new()
    }
}

impl BodySlab {
    /// 构造空 slab。
    #[must_use]
    pub fn new() -> Self {
        Self {
            slab: Slab::new(),
        }
    }

    /// 预分配容量。
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            slab: Slab::with_capacity(capacity),
        }
    }

    /// 分配 dynamic body。
    pub fn insert_dynamic(
        &mut self,
        mass: f32,
        inertia_diag: [f32; 3],
    ) -> Result<(gvpe_core::BodyHandle, u32), crate::SolverError> {
        let slot_index = self.slab.active_count() as u32;
        let h = self.slab.allocate(RigidBody::new_dynamic(
            gvpe_core::BodyHandle::INVALID, // 由 body_slab 重新映射；此处用 INVALID 占位
            slot_index,
            mass,
            inertia_diag,
        ));
        // 重新构造以写入正确 handle
        let body = self.slab.get_mut(h).map_err(|_| crate::SolverError::BodyNotFound {
            index: h.index,
            generation: h.generation,
        })?;
        body.handle = h.into_core_handle();
        Ok((body.handle, slot_index))
    }

    /// 分配 static body。
    pub fn insert_static(&mut self) -> Result<(gvpe_core::BodyHandle, u32), crate::SolverError> {
        let slot_index = self.slab.active_count() as u32;
        let h = self.slab.allocate(RigidBody::new_static(
            gvpe_core::BodyHandle::INVALID,
            slot_index,
        ));
        let body = self.slab.get_mut(h).map_err(|_| crate::SolverError::BodyNotFound {
            index: h.index,
            generation: h.generation,
        })?;
        body.handle = h.into_core_handle();
        Ok((body.handle, slot_index))
    }

    /// 取出 body 引用。
    pub fn get(&self, handle: gvpe_core::BodyHandle) -> Option<&RigidBody> {
        let sh = gvpe_memory::SlabHandle {
            index: handle.index,
            generation: handle.generation,
        };
        self.slab.get(sh).ok()
    }

    /// 取出 body 可变引用。
    pub fn get_mut(&mut self, handle: gvpe_core::BodyHandle) -> Option<&mut RigidBody> {
        let sh = gvpe_memory::SlabHandle {
            index: handle.index,
            generation: handle.generation,
        };
        self.slab.get_mut(sh).ok()
    }

    /// 释放 body。
    pub fn remove(&mut self, handle: gvpe_core::BodyHandle) -> Result<(), crate::SolverError> {
        let sh = gvpe_memory::SlabHandle {
            index: handle.index,
            generation: handle.generation,
        };
        self.slab
            .free(sh)
            .map_err(|_| crate::SolverError::BodyNotFound {
                index: handle.index,
                generation: handle.generation,
            })
    }

    /// 当前 body 总数（活跃 slot 数）。
    #[must_use]
    pub fn len(&self) -> usize {
        self.slab.active_count()
    }

    /// 是否无 body。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.slab.active_count() == 0
    }

    /// 活跃 body 迭代器（filter by `SleepState::Active`）。
    pub fn iter_active(&self) -> impl Iterator<Item = &RigidBody> {
        self.slab.iter().filter_map(|(_, b)| {
            if matches!(b.sleep, SleepState::Active) {
                Some(b)
            } else {
                None
            }
        })
    }

    /// 全部 body 迭代器。
    pub fn iter(&self) -> impl Iterator<Item = &RigidBody> {
        self.slab.iter().map(|(_, b)| b)
    }

    /// 全部 body 可变迭代器。
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut RigidBody> {
        self.slab.iter_mut().map(|(_, b)| b)
    }
}

// ───────── gvpe_memory::SlabHandle ↔ gvpe_core::BodyHandle 互转 ─────────
//
// 避免在 `gvpe-memory` / `gvpe-core` 之间引入循环依赖；`SlabHandle` 与 `BodyHandle`
// 字段同形（index: u32, generation: u32），可直接位重建。
trait SlabHandleExt {
    fn into_core_handle(self) -> gvpe_core::BodyHandle;
}

impl SlabHandleExt for gvpe_memory::SlabHandle {
    #[inline]
    fn into_core_handle(self) -> gvpe_core::BodyHandle {
        gvpe_core::BodyHandle::new(self.index, self.generation)
    }
}
