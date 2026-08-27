//! [`Scheduler`]：4 阶段串行调度器（MVP）。
//!
//! 调度顺序（与 [`crate::stage::StageKind::ALL`] 一致）：
//! `Predict → Collision → Solver → Integrate`。
//!
//! MVP 行为：调度器在调用 [`Scheduler::run`] 的当前线程上**串行**执行
//! 4 个 stage 的 `run` 方法，**不**创建任何 OS 线程，**不**维护依赖计数器
//! —— 因为顺序执行天然保证阶段依赖。
//!
//! v0.4+ 演进方向：
//! - 引入 [`WorkerPool`](crate::pool::WorkerPool) 真正并行执行 island-level fan-out。
//! - 维护 [`JobHandle`](crate::job::JobHandle) 依赖计数器图。
//! - 任何阶段失败时按 [`ScheduleError`] 决定是短路返回还是记录后继续。

use gvpe_core::IslandHandle;

use crate::pool::SequentialPool;
use crate::stage::{Stage, StageKind};

/// 调度失败错误。
///
/// MVP 阶段任一 stage 失败即返回 `StageFailed { kind, ... }`；
/// 已成功阶段的输出已应用到使用方 world，不可回滚（per
/// `09_parallel_design.md` §5.4 —— MVP 范围"基本多线程"非事务性）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScheduleError {
    /// 指定 stage 失败。
    StageFailed {
        /// 失败阶段。
        kind: StageKind,
    },
    /// 调度器未配置 stage（[`Scheduler::new`] / builder 阶段允许缺省，
    /// 但 [`Scheduler::run`] 时所有 4 阶段必须就位）。
    MissingStage(StageKind),
}

impl core::fmt::Display for ScheduleError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::StageFailed { kind } => write!(f, "schedule failed at stage {kind:?}"),
            Self::MissingStage(kind) => write!(f, "scheduler missing stage {kind:?}"),
        }
    }
}

impl std::error::Error for ScheduleError {}

/// 单次 [`Scheduler::run`] 的统计信息。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SchedulerStats {
    /// 本帧处理的 island 数（4 阶段 processed 之和）。
    pub total_islands_processed: u32,
    /// 本帧跳过的 island 数（4 阶段 skipped 之和）。
    pub total_islands_skipped: u32,
    /// 实际完成的阶段数（0..=4）。
    pub stages_completed: u32,
    /// 失败的阶段数（0 或 1；MVP 一旦失败即返回，不继续后续阶段）。
    pub stages_failed: u32,
}

impl SchedulerStats {
    /// 全部 4 阶段成功完成。
    #[inline]
    pub const fn all_ok(processed: u32, skipped: u32) -> Self {
        Self {
            total_islands_processed: processed,
            total_islands_skipped: skipped,
            stages_completed: 4,
            stages_failed: 0,
        }
    }
}

/// 4 阶段调度器。
///
/// # MVP 行为
///
/// - 持有 4 个具体 stage（类型擦除为 `Box<dyn Stage>`）+ 一个 [`SequentialPool`]。
/// - [`Scheduler::run`] 按 `Predict → Collision → Solver → Integrate` 顺序
///   在调用线程同步执行，任一阶段失败即短路返回 [`ScheduleError`]。
///
/// # 设计契约
///
/// - 阶段 trait 调用方实现 → 调度器零侵入支持 v0.4+ 升级。
/// - Stage trait **不**要求 `Send` / `Sync`（MVP 单线程）。
pub struct Scheduler {
    stages: [Option<Box<dyn Stage>>; 4],
    pool: SequentialPool,
}

impl Scheduler {
    /// 新建空调度器（4 个 stage 槽位全部 `None`）。
    pub fn new() -> Self {
        Self {
            stages: [None, None, None, None],
            pool: SequentialPool::new(),
        }
    }

    /// 指定 worker 数（**MVP 限制**：记入配置但不创建线程；见
    /// [`WorkerPoolBuilder::worker_count`](crate::pool::WorkerPoolBuilder::worker_count)）。
    pub fn with_worker_count(n: u32) -> Self {
        Self {
            stages: [None, None, None, None],
            pool: SequentialPool {
                configured_workers: n.max(1),
            },
        }
    }

    /// 安装具体 stage（覆盖该槽位原有值；返回旧值便于取回）。
    pub fn install_stage(&mut self, stage: Box<dyn Stage>) -> Option<Box<dyn Stage>> {
        let idx = stage.kind().index().expect("stage kind out of range");
        self.stages[idx].replace(stage)
    }

    /// 取出指定 stage（测试 / 升级用）。
    pub fn take_stage(&mut self, kind: StageKind) -> Option<Box<dyn Stage>> {
        let idx = kind.index()?;
        self.stages[idx].take()
    }

    /// 查询 stage 是否就位。
    pub fn has_stage(&self, kind: StageKind) -> bool {
        kind.index()
            .and_then(|i| self.stages.get(i))
            .and_then(Option::as_ref)
            .is_some()
    }

    /// 当前 worker 池引用。
    pub const fn pool(&self) -> &SequentialPool {
        &self.pool
    }

    /// 同步执行一帧：按 `Predict → Collision → Solver → Integrate` 顺序。
    ///
    /// # Errors
    ///
    /// - 任一阶段 `run` 返回 `failed = true` → `StageFailed { kind }`。
    /// - 任一阶段未 [`install_stage`](Self::install_stage) → `MissingStage(kind)`。
    pub fn run(&mut self, islands: &[IslandHandle]) -> Result<SchedulerStats, ScheduleError> {
        let mut stats = SchedulerStats::default();
        for kind in StageKind::ALL {
            let stage = self
                .stages[kind
                    .index()
                    .expect("ALL contains only in-range kinds")]
                .as_mut()
                .ok_or(ScheduleError::MissingStage(kind))?;
            let out = stage.run(islands);
            stats.total_islands_processed =
                stats.total_islands_processed.saturating_add(out.islands_processed);
            stats.total_islands_skipped =
                stats.total_islands_skipped.saturating_add(out.islands_skipped);
            if out.failed {
                return Err(ScheduleError::StageFailed {
                    kind,
                });
            }
            stats.stages_completed = stats.stages_completed.saturating_add(1);
        }
        Ok(stats)
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Debug for Scheduler {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Scheduler")
            .field("pool", &self.pool)
            .field(
                "stages",
                &StageKind::ALL.map(|k| self.has_stage(k)),
            )
            .finish()
    }
}
