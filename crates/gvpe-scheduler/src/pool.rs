//! Worker 线程池抽象：v0.4+ 真并行阶段的占位。
//!
//! MVP **不**创建任何 OS 线程；本模块提供：
//! - trait [`WorkerPool`] —— 调度器与具体线程池实现的解耦点
//! - [`WorkerPoolBuilder`] —— 顺序池构造（v0.3 实装，仅记录配置，不创建线程）
//! - [`SequentialPool`] —— 顺序池实现（`worker_count = 1`），MVP 默认
//!
//! 设计参考：`docs/01_architecture/09_parallel_design.md` §6.3（"thread pool
//! with dependency counters"）+ §6.4（MVP = "simple thread pool"，工作窃取
//! 非门禁）。

use crate::stage::StageKind;

/// Worker 池抽象。
///
/// MVP 唯一实装是 [`SequentialPool`]（单 worker 顺序执行）。v0.4+ 接入 rayon /
/// tokio / crossbeam 时，调度器仅通过本 trait 交互，零侵入升级。
pub trait WorkerPool {
    /// worker 数量（MVP = 1）。
    fn worker_count(&self) -> u32;

    /// 是否为顺序池（MVP = true）。
    ///
    /// 调度器可用此标记做 fast-path 优化（避免不必要的依赖计数器维护）。
    fn is_sequential(&self) -> bool;
}

/// Worker 池构造器。
///
/// MVP 阶段仅构造 [`SequentialPool`]；v0.4+ 扩展 `with_parallel(n)` /
/// `with_steal(n)` 等变体。
#[derive(Clone, Copy, Debug, Default)]
pub struct WorkerPoolBuilder {
    desired_workers: u32,
}

impl WorkerPoolBuilder {
    /// 新建 builder（默认 `worker_count = 1`，即顺序池）。
    pub const fn new() -> Self {
        Self {
            desired_workers: 1,
        }
    }

    /// 设置 worker 数（**MVP 限制**：传入值 > 1 时记入配置但不创建额外线程，
    /// 实际执行仍为单线程；`is_sequential()` 仍返 `true`）。
    ///
    /// v0.4+ 此方法会真正创建 `n - 1` 个工作线程。
    #[must_use]
    pub const fn worker_count(mut self, n: u32) -> Self {
        self.desired_workers = n;
        self
    }

    /// 构造顺序池（MVP）。
    pub fn build_sequential(self) -> SequentialPool {
        SequentialPool {
            configured_workers: self.desired_workers.max(1),
        }
    }

    /// 构造 worker 池（**MVP 限制**：等价于 [`build_sequential`](Self::build_sequential)）。
    pub fn build(self) -> SequentialPool {
        self.build_sequential()
    }
}

/// 顺序 worker 池：MVP 默认。
///
/// `configured_workers` 记录用户期望 worker 数（v0.4+ 升级用），但实际
/// `worker_count()` 永远返回 1，`is_sequential()` 永远返回 `true`。
#[derive(Clone, Copy, Debug)]
pub struct SequentialPool {
    pub(crate) configured_workers: u32,
}

impl SequentialPool {
    /// 单 worker 顺序池（默认）。
    pub const fn new() -> Self {
        Self {
            configured_workers: 1,
        }
    }

    /// 用户配置的 worker 数（v0.3 = 提示值；v0.4+ = 实际值）。
    #[inline]
    pub const fn configured_workers(self) -> u32 {
        self.configured_workers
    }
}

impl Default for SequentialPool {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkerPool for SequentialPool {
    #[inline]
    fn worker_count(&self) -> u32 {
        1
    }

    #[inline]
    fn is_sequential(&self) -> bool {
        true
    }
}

/// 4 阶段在顺序池上的"理论并行度"（per `09_parallel_design.md` §6.2）。
///
/// MVP 实测总 = 1（顺序池）。`max_concurrent_stages()` 反映阶段间并行可能性：
/// 4 阶段全 fan-out 形态下理论峰值 4。
#[allow(dead_code)] // 公开 API 常量，v0.4+ 真并行时实际使用；当前无内部调用。
pub const fn max_concurrent_stages() -> u32 {
    4
}

// 让 [`StageKind`] 在本模块可见（rustdoc 链接不报 missing_docs）。
#[allow(dead_code)]
const _STAGE_KIND_LINK: StageKind = StageKind::Predict;
