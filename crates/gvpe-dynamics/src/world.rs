//! `DynamicsWorld`：主控制器。
//!
//! 职责：
//!
//! - 持有所有 body 状态（`Slab<BodyEntry>` 带世代，use-after-free 检测）
//! - 持有 island 列表（v0.3 MVP：每个 body 自成一个 island；v0.4+ 引入
//!   island formation / merge）
//! - 单步主循环 `step(dt)`：`predict` → `integrate` → `finalize`
//!
//! ## 不做（v0.3 范围外）
//!
//! - 多 island 同步 / 跨 island 约束求解（v0.5+ 引入 `gvpe-solver`）
//! - 接触约束响应 / Sequential Impulse（v0.5+）
//! - 睡眠机制 / 岛重建 / 接触图（v0.6+）
//! - `BodySpec` 加载（v0.3 仅 `BodyHandle + 状态 + profile` 三元组；v0.4+
//!   从 `gvpe-core::RuntimeDescriptor` 桥接）

use gvpe_core::{BodyHandle, IslandHandle, PhysicsProfile};
use gvpe_math::{Quat, Vec3};
use gvpe_memory::{Slab, SlabError, SlabHandle};

use crate::error::{DynamicsError, DynamicsResult};
use crate::state::{RigidBodyState, TimeStepper};
use crate::step;

/// body 内部条目（state + profile + island 引用）。
#[derive(Clone, Debug)]
struct BodyEntry {
    /// 刚体状态。
    state: RigidBodyState,
    /// 物理 profile（拷贝持有，避免外部修改影响本帧计算）。
    profile: PhysicsProfile,
    /// 所属 island 句柄。
    island: IslandHandle,
}

/// `DynamicsWorld`：刚体动力学主控制器。
///
/// ## 不变量
///
/// - `islands` 索引与 [`IslandHandle::0`] 一一对应
/// - `bodies` slot 索引与 [`BodyHandle::index`] 一一对应
/// - 每个活跃 body 必属于一个 island
#[derive(Debug)]
pub struct DynamicsWorld {
    bodies: Slab<BodyEntry>,
    islands: Vec<Vec<BodyHandle>>,
    gravity: Vec3,
    stepper: TimeStepper,
}

impl Default for DynamicsWorld {
    fn default() -> Self {
        Self::new()
    }
}

impl DynamicsWorld {
    /// 新建空 world（默认重力 `(0, -9.81, 0)`，半隐式 Euler）。
    #[must_use]
    pub fn new() -> Self {
        Self {
            bodies: Slab::new(),
            islands: Vec::new(),
            gravity: Vec3::new(0.0, -9.81, 0.0),
            stepper: TimeStepper::default(),
        }
    }

    /// 新建空 world，预分配容量（避免热路径首次 realloc）。
    #[must_use]
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            bodies: Slab::with_capacity(cap),
            islands: Vec::new(),
            gravity: Vec3::new(0.0, -9.81, 0.0),
            stepper: TimeStepper::default(),
        }
    }

    /// 设置全局重力。
    #[inline]
    pub fn set_gravity(&mut self, gravity: Vec3) {
        self.gravity = gravity;
    }

    /// 获取全局重力。
    #[inline]
    #[must_use]
    pub fn gravity(&self) -> Vec3 {
        self.gravity
    }

    /// 设置时间步进算法。
    #[inline]
    pub fn set_stepper(&mut self, stepper: TimeStepper) {
        self.stepper = stepper;
    }

    /// 获取时间步进算法。
    #[inline]
    #[must_use]
    pub fn stepper(&self) -> TimeStepper {
        self.stepper
    }

    /// 活跃 body 数。
    #[inline]
    #[must_use]
    pub fn body_count(&self) -> usize {
        self.bodies.active_count()
    }

    /// island 数。
    #[inline]
    #[must_use]
    pub fn island_count(&self) -> usize {
        self.islands.len()
    }

    /// 派生一个 island handle（v0.3 MVP：每个 body 自成 island）。
    fn alloc_island(&mut self) -> IslandHandle {
        let idx = self.islands.len() as u32;
        self.islands.push(Vec::new());
        IslandHandle::from_raw(idx)
    }

    /// 生成 dynamic body（`mass > 0`），返回 handle。
    ///
    /// # Errors
    ///
    /// 无（当前实现不返回 `Err`，保留 API 以便 v0.4+ 引入约束校验时扩展）。
    pub fn spawn_dynamic(
        &mut self,
        position: Vec3,
        rotation: Quat,
        mass: f32,
        inertia_diag: [f32; 3],
        profile: PhysicsProfile,
    ) -> DynamicsResult<BodyHandle> {
        let state = RigidBodyState::dynamic(position, rotation, Vec3::ZERO, Vec3::ZERO, mass, inertia_diag);
        let island = self.alloc_island();
        let entry = BodyEntry { state, profile, island };
        let SlabHandle { index, generation } = self.bodies.allocate(entry);
        let body = BodyHandle::from_raw(index, generation);
        self.islands[island.0 as usize].push(body);
        Ok(body)
    }

    /// 生成 static body（`mass == 0`），返回 handle。
    pub fn spawn_fixed(&mut self, position: Vec3, rotation: Quat) -> DynamicsResult<BodyHandle> {
        let state = RigidBodyState::fixed(position, rotation);
        let profile = PhysicsProfile::default_static();
        // static body 阻尼无意义但保留（force 累加 no-op 已阻断）
        let island = self.alloc_island();
        let entry = BodyEntry { state, profile, island };
        let SlabHandle { index, generation } = self.bodies.allocate(entry);
        let body = BodyHandle::from_raw(index, generation);
        self.islands[island.0 as usize].push(body);
        Ok(body)
    }

    /// 移除 body（use-after-free 由 `Slab` generation 检测）。
    pub fn remove_body(&mut self, handle: BodyHandle) -> DynamicsResult<()> {
        let entry = self.bodies.get(h2s(handle)).map_err(map_slab_err)?;
        let island_idx = entry.island.0 as usize;
        // 从 island 列表移除该 body（O(n)，n 通常很小，MVP 阶段可接受）
        if let Some(pos) = self.islands[island_idx]
            .iter()
            .position(|h| h.index == handle.index && h.generation == handle.generation)
        {
            self.islands[island_idx].swap_remove(pos);
        }
        self.bodies.free(h2s(handle)).map_err(map_slab_err)?;
        Ok(())
    }

    /// 借取 body 状态（不可变）。
    pub fn body(&self, handle: BodyHandle) -> DynamicsResult<&RigidBodyState> {
        self.bodies
            .get(h2s(handle))
            .map(|e| &e.state)
            .map_err(map_slab_err)
    }

    /// 借取 body 状态（可变）。
    pub fn body_mut(&mut self, handle: BodyHandle) -> DynamicsResult<&mut RigidBodyState> {
        self.bodies
            .get_mut(h2s(handle))
            .map(|e| &mut e.state)
            .map_err(map_slab_err)
    }

    /// 借取 body profile（不可变）。
    pub fn profile(&self, handle: BodyHandle) -> DynamicsResult<&PhysicsProfile> {
        self.bodies
            .get(h2s(handle))
            .map(|e| &e.profile)
            .map_err(map_slab_err)
    }

    /// 借取 body 所属 island 句柄。
    pub fn body_island(&self, handle: BodyHandle) -> DynamicsResult<IslandHandle> {
        self.bodies
            .get(h2s(handle))
            .map(|e| e.island)
            .map_err(map_slab_err)
    }

    /// island 包含的 body 列表。
    pub fn island_bodies(&self, island: IslandHandle) -> DynamicsResult<&[BodyHandle]> {
        if (island.0 as usize) >= self.islands.len() {
            return Err(DynamicsError::IslandInvalid(island));
        }
        Ok(&self.islands[island.0 as usize])
    }

    /// 单步主循环：`predict` → `integrate` → `finalize` → `validate`。
    ///
    /// 阶段说明：
    ///
    /// 1. **predict**：对每个 active body 应用重力 + 阻尼（不移动位置 / 旋转）。
    /// 2. **integrate**：按 `self.stepper` 派发半隐式 / 显式 / RK4。
    /// 3. **finalize**：清零 `force` / `torque` 累加器（下个 `predict` 起点干净）。
    /// 4. **validate**：捕获极端数值（NaN / Inf / rotation-unnormalized），
    ///    出错立刻短路返回 [`DynamicsError::StateNotFinite`]。
    ///
    /// # Errors
    ///
    /// - `dt <= 0` 或 `NaN` → [`DynamicsError::InvalidTimeStep`]
    /// - body 状态数值非法（极端 case）→ [`DynamicsError::StateNotFinite`]
    pub fn step(&mut self, dt: f32) -> DynamicsResult<()> {
        step::validate_dt(dt)?;
        // 阶段 1：predict（对每个 active body 应用重力 + 阻尼）。
        for (_h, entry) in self.bodies.iter_mut() {
            step::predict_with_damping(&mut entry.state, self.gravity, &entry.profile, dt);
        }
        // 阶段 2：integrate。
        for (_h, entry) in self.bodies.iter_mut() {
            step::integrate(&mut entry.state, dt, self.stepper);
        }
        // 阶段 3：finalize（清零 force / torque）。
        for (_h, entry) in self.bodies.iter_mut() {
            step::finalize(&mut entry.state);
        }
        // 阶段 4：validate（捕获极端数值）。
        for (_h, entry) in self.bodies.iter() {
            entry.state.validate()?;
        }
        Ok(())
    }
}

/// 模块级 helper：handle → `SlabHandle`（无 orphan rule 限制）。
#[inline]
fn h2s(h: BodyHandle) -> SlabHandle {
    SlabHandle {
        index: h.index,
        generation: h.generation,
    }
}

/// 映射 `SlabError` → `DynamicsError`。
#[inline]
fn map_slab_err(e: SlabError) -> DynamicsError {
    match e {
        SlabError::GenerationMismatch { .. } => DynamicsError::HandleStale(BodyHandle::INVALID),
        other @ SlabError::OutOfBounds(..) => DynamicsError::Slab(other),
    }
}
