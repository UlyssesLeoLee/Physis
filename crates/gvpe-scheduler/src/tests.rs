//! `gvpe-scheduler` 单元测试。
//!
//! 覆盖维度（per 任务范围"测试 ≥ 9 个：调度顺序 + 阶段依赖"）：
//! 1. [`StageKind`] 拓扑（ALL 长度、index、predecessor、successor）
//! 2. [`StageOutput`] 构造与聚合
//! 3. [`Scheduler`] 调度顺序（用 spy stage 断言严格按
//!    `Predict → Collision → Solver → Integrate` 顺序调用）
//! 4. [`Scheduler`] 失败短路 / 缺 stage 错误
//! 5. [`WorkerPool`] 顺序池契约
//! 6. [`JobHandle`] / [`JobId`] 占位类型

#![allow(clippy::redundant_clone)] // SpyStage 需要每个实例独立持有一个 Rc，Rc::clone 在测试 fixture 中是合法共享模式。

use std::cell::Cell;

use gvpe_core::IslandHandle;

use crate::job::{JobHandle, JobId, JobKind};
use crate::pool::{SequentialPool, WorkerPool, WorkerPoolBuilder, max_concurrent_stages};
use crate::scheduler::{ScheduleError, Scheduler, SchedulerStats};
use crate::stage::{Stage, StageKind, StageOutput};

// ---------- StageKind 拓扑 ------------------------------------------------

#[test]
fn stage_kind_all_has_four_entries_in_order() {
    assert_eq!(StageKind::ALL.len(), 4);
    assert_eq!(StageKind::ALL[0], StageKind::Predict);
    assert_eq!(StageKind::ALL[1], StageKind::Collision);
    assert_eq!(StageKind::ALL[2], StageKind::Solver);
    assert_eq!(StageKind::ALL[3], StageKind::Integrate);
}

#[test]
fn stage_kind_index_pred_succ_chain() {
    // 索引：0..=3 一一对应。
    assert_eq!(StageKind::Predict.index(), Some(0));
    assert_eq!(StageKind::Collision.index(), Some(1));
    assert_eq!(StageKind::Solver.index(), Some(2));
    assert_eq!(StageKind::Integrate.index(), Some(3));

    // 链：Predict -> Collision -> Solver -> Integrate。
    assert_eq!(StageKind::Predict.predecessor(), None);
    assert_eq!(StageKind::Predict.successor(), Some(StageKind::Collision));
    assert_eq!(StageKind::Collision.predecessor(), Some(StageKind::Predict));
    assert_eq!(StageKind::Collision.successor(), Some(StageKind::Solver));
    assert_eq!(StageKind::Solver.predecessor(), Some(StageKind::Collision));
    assert_eq!(StageKind::Solver.successor(), Some(StageKind::Integrate));
    assert_eq!(StageKind::Integrate.predecessor(), Some(StageKind::Solver));
    assert_eq!(StageKind::Integrate.successor(), None);

    // 双向链首尾相接：全集长度 = 4。
    let mut chain_len = 0;
    let mut cursor = Some(StageKind::Predict);
    while let Some(c) = cursor {
        chain_len += 1;
        cursor = c.successor();
    }
    assert_eq!(chain_len, 4);
}

// ---------- StageOutput 聚合 ----------------------------------------------

#[test]
fn stage_output_constructors_and_total() {
    let z = StageOutput::new();
    assert_eq!(z.islands_processed, 0);
    assert_eq!(z.islands_skipped, 0);
    assert!(!z.failed);
    assert_eq!(z.total(), 0);

    let ok = StageOutput::ok(3);
    assert_eq!(ok.islands_processed, 3);
    assert_eq!(ok.total(), 3);
    assert!(!ok.failed);

    let sk = StageOutput::all_skipped(2);
    assert_eq!(sk.islands_skipped, 2);
    assert_eq!(sk.total(), 2);
    assert!(!sk.failed);

    let f = StageOutput::failed();
    assert!(f.failed);
    assert_eq!(f.total(), 0);
}

// ---------- Scheduler 调度顺序（核心）------------------------------------

/// 记录调用顺序的 spy stage。
struct SpyStage {
    kind: StageKind,
    order_log: std::rc::Rc<Cell<Vec<StageKind>>>,
    out: StageOutput,
}

impl SpyStage {
    fn new(kind: StageKind, log: std::rc::Rc<Cell<Vec<StageKind>>>, out: StageOutput) -> Self {
        Self {
            kind,
            order_log: log,
            out,
        }
    }
}

impl Stage for SpyStage {
    fn kind(&self) -> StageKind {
        self.kind
    }
    fn run(&mut self, _islands: &[IslandHandle]) -> StageOutput {
        let mut v = self.order_log.take();
        v.push(self.kind);
        self.order_log.set(v);
        self.out
    }
}

fn fresh_islands(n: u32) -> Vec<IslandHandle> {
    (1..=n).map(IslandHandle::new).collect()
}

#[test]
fn scheduler_runs_stages_in_strict_order() {
    let log: std::rc::Rc<Cell<Vec<StageKind>>> = std::rc::Rc::new(Cell::new(Vec::new()));
    let mut sched = Scheduler::new();
    for k in StageKind::ALL {
        let out = StageOutput::ok(0);
        sched.install_stage(Box::new(SpyStage::new(k, log.clone(), out)));
    }

    let stats = sched.run(&fresh_islands(0)).expect("MVP run must succeed");
    assert_eq!(stats.stages_completed, 4);
    assert_eq!(stats.stages_failed, 0);
    assert_eq!(
        log.take(),
        vec![
            StageKind::Predict,
            StageKind::Collision,
            StageKind::Solver,
            StageKind::Integrate,
        ]
    );
}

#[test]
fn scheduler_aggregates_island_counts_across_stages() {
    let log: std::rc::Rc<Cell<Vec<StageKind>>> = std::rc::Rc::new(Cell::new(Vec::new()));
    let mut sched = Scheduler::new();
    sched.install_stage(Box::new(SpyStage::new(
        StageKind::Predict,
        log.clone(),
        StageOutput::ok(2),
    )));
    sched.install_stage(Box::new(SpyStage::new(
        StageKind::Collision,
        log.clone(),
        StageOutput::all_skipped(1),
    )));
    sched.install_stage(Box::new(SpyStage::new(
        StageKind::Solver,
        log.clone(),
        StageOutput::ok(3),
    )));
    sched.install_stage(Box::new(SpyStage::new(
        StageKind::Integrate,
        log.clone(),
        StageOutput::ok(1),
    )));

    let stats = sched.run(&fresh_islands(4)).expect("run ok");
    // processed = 2 + 0 + 3 + 1 = 6
    // skipped   = 0 + 1 + 0 + 0 = 1
    assert_eq!(stats.total_islands_processed, 6);
    assert_eq!(stats.total_islands_skipped, 1);
    assert_eq!(stats.stages_completed, 4);
    assert_eq!(stats.stages_failed, 0);
}

#[test]
fn scheduler_short_circuits_on_first_failed_stage() {
    let log: std::rc::Rc<Cell<Vec<StageKind>>> = std::rc::Rc::new(Cell::new(Vec::new()));
    let mut sched = Scheduler::new();
    // Collision 阶段失败 → Solver / Integrate 不应被调用。
    sched.install_stage(Box::new(SpyStage::new(
        StageKind::Predict,
        log.clone(),
        StageOutput::ok(0),
    )));
    sched.install_stage(Box::new(SpyStage::new(
        StageKind::Collision,
        log.clone(),
        StageOutput::failed(),
    )));
    sched.install_stage(Box::new(SpyStage::new(
        StageKind::Solver,
        log.clone(),
        StageOutput::ok(0),
    )));
    sched.install_stage(Box::new(SpyStage::new(
        StageKind::Integrate,
        log.clone(),
        StageOutput::ok(0),
    )));

    let err = sched.run(&fresh_islands(0)).expect_err("must fail");
    assert_eq!(err, ScheduleError::StageFailed {
        kind: StageKind::Collision
    });
    // 短路断言：仅 Predict + Collision 被调用，Solver / Integrate 未触发。
    assert_eq!(
        log.take(),
        vec![StageKind::Predict, StageKind::Collision],
    );
}

#[test]
fn scheduler_missing_stage_returns_error() {
    let mut sched = Scheduler::new();
    // 仅装 Solver，其他 3 槽位空。
    let log: std::rc::Rc<Cell<Vec<StageKind>>> = std::rc::Rc::new(Cell::new(Vec::new()));
    sched.install_stage(Box::new(SpyStage::new(
        StageKind::Solver,
        log,
        StageOutput::ok(0),
    )));

    let err = sched.run(&fresh_islands(0)).expect_err("must miss stage");
    // 第一阶段 Predict 缺失即报。
    assert_eq!(err, ScheduleError::MissingStage(StageKind::Predict));
}

#[test]
fn scheduler_with_worker_count_records_config() {
    // MVP 限制：worker_count 传入 n>1 仍 sequential，但 configured_workers 记录。
    let sched = Scheduler::with_worker_count(8);
    assert_eq!(sched.pool().worker_count(), 1);
    assert!(sched.pool().is_sequential());
    assert_eq!(sched.pool().configured_workers(), 8);
    assert!(!sched.has_stage(StageKind::Predict));
}

// ---------- WorkerPool 契约 ------------------------------------------------

#[test]
fn worker_pool_default_is_sequential_single_worker() {
    let pool = WorkerPoolBuilder::new().build();
    assert_eq!(pool.worker_count(), 1);
    assert!(pool.is_sequential());
}

#[test]
fn worker_pool_builder_records_desired_count_but_stays_sequential_mvp() {
    let pool = WorkerPoolBuilder::new().worker_count(4).build();
    assert_eq!(pool.configured_workers(), 4);
    assert_eq!(pool.worker_count(), 1); // MVP 限制
    assert!(pool.is_sequential());
}

#[test]
fn sequential_pool_default_and_constructor_match() {
    let a = SequentialPool::default();
    let b = SequentialPool::new();
    assert_eq!(a.worker_count(), b.worker_count());
    assert_eq!(a.configured_workers(), b.configured_workers());
    assert!(a.is_sequential());
}

#[test]
fn max_concurrent_stages_reflects_four_stage_fanout() {
    // 4 阶段全部并行 fan-out 的理论峰值。
    assert_eq!(max_concurrent_stages(), 4);
}

// ---------- Job / JobHandle 占位 ------------------------------------------

#[test]
fn job_id_invalid_and_marker() {
    assert!(JobId::INVALID.is_invalid());
    assert_eq!(JobId::INVALID, JobId(0));
    assert!(!JobId::new_for_test(7).is_invalid());
    assert_eq!(JobId::new_for_test(7).0, 7);
}

#[test]
fn job_handle_invalid_marker() {
    let h = JobHandle::INVALID;
    assert!(h.is_invalid());
    assert_eq!(h.id, JobId::INVALID);
    assert_eq!(h.kind, JobKind::Stage(StageKind::Predict));
}

// ---------- ScheduleError Display 稳定性 ----------------------------------

#[test]
fn schedule_error_display_is_stable_strings() {
    // DDD Review 关注：消息文本是公开契约的一部分；改动前需评审。
    let a = ScheduleError::StageFailed {
        kind: StageKind::Solver,
    };
    let b = ScheduleError::MissingStage(StageKind::Integrate);
    assert_eq!(a.to_string(), "schedule failed at stage Solver");
    assert_eq!(b.to_string(), "scheduler missing stage Integrate");
}

// ---------- SchedulerStats 工厂 -------------------------------------------

#[test]
fn scheduler_stats_all_ok_factory() {
    let s = SchedulerStats::all_ok(10, 2);
    assert_eq!(s.total_islands_processed, 10);
    assert_eq!(s.total_islands_skipped, 2);
    assert_eq!(s.stages_completed, 4);
    assert_eq!(s.stages_failed, 0);
}
