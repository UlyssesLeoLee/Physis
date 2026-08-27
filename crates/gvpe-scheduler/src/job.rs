//! Job 句柄：v0.4+ 真并行阶段的占位契约。
//!
//! MVP **不**创建实际 job；本模块仅定义类型骨架，使调度器在 v0.4 接入 rayon /
//! crossbeam / tokio 时 API 边界不变。
//!
//! 设计参考：`docs/01_architecture/09_parallel_design.md` §6.3（候选调度机制）
//! 与 §9.3（"thread pool with dependency counters, job B runs only after job
//! A's counter hits zero"）。

use crate::stage::StageKind;

/// Job 类型：MVP 仅 [`Stage`](crate::stage::Stage) 粒度，v0.4+ 扩展为
/// per-island / per-pair 粒度（per `09_parallel_design.md` §6.2 fan-out 点）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JobKind {
    /// 整个 stage 作为一个 job（MVP）。
    Stage(StageKind),
}

impl Default for JobKind {
    /// `Stage(Predict)` 作为占位 default（与 [`JobHandle::INVALID`] 一致）。
    #[inline]
    fn default() -> Self {
        Self::Stage(crate::stage::StageKind::Predict)
    }
}

/// Job 全局唯一 ID（单调递增，从 1 开始；0 保留为无效值）。
///
/// MVP 不创建 job —— 所有 [`JobHandle`] 构造必须经过 [`JobHandle::INVALID`]
/// 或后续 v0.4+ 的真实分配函数。当前阶段提供 `new_for_test` 仅供测试断言。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct JobId(pub u32);

impl JobId {
    /// 无效 ID（0）。
    pub const INVALID: Self = Self(0);

    /// 测试用 ID 构造（**仅** dev / test 代码可见）。
    #[cfg(test)]
    pub(crate) const fn new_for_test(id: u32) -> Self {
        Self(id)
    }

    /// 是否为 [`INVALID`](Self::INVALID)。
    #[inline]
    pub const fn is_invalid(self) -> bool {
        self.0 == 0
    }
}

/// Job 句柄：MVP 阶段为不透明占位。
///
/// v0.4+ 将承载：
/// - 依赖计数器（依赖 job 全部完成时本 job 触发）
/// - 提交时间戳
/// - 关联 `JobKind`
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct JobHandle {
    /// Job ID。
    pub id: JobId,
    /// Job 类型。
    pub kind: JobKind,
}

impl JobHandle {
    /// 无效句柄（`id = INVALID`, `kind = Stage(Predict)` 仅作占位）。
    pub const INVALID: Self = Self {
        id: JobId::INVALID,
        kind: JobKind::Stage(StageKind::Predict),
    };

    /// 是否为 [`INVALID`](Self::INVALID)。
    #[inline]
    pub const fn is_invalid(&self) -> bool {
        self.id.is_invalid()
    }
}
