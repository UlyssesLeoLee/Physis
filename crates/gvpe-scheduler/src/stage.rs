//! 4 阶段抽象：`Predict` / `Collision` / `Solver` / `Integrate`。
//!
//! 每个阶段实现 [`Stage`] trait；调度器 ([`crate::Scheduler`]) 按固定顺序
//! 串行调用四个具体阶段的 `run` 方法。
//!
//! MVP 阶段内不 fan-out（v0.4+ 演进为真并行）。阶段粒度抽象是**契约边界**：
//! 调用方在自己的 stage 实现内决定是否按 island 分发。

use gvpe_core::IslandHandle;

/// 阶段执行输出。
///
/// MVP 最小化：阶段仅返回处理的 island 数量统计 + 错误标志。
/// 真正的输出数据（contact manifold / 求解后的速度等）由使用方在自己的
/// stage 实现里管理 world / arena —— 本 trait 不直接读写世界状态。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StageOutput {
    /// 处理的 island 数量。
    pub islands_processed: u32,
    /// 跳过的 island 数量（sleeping / 空岛）。
    pub islands_skipped: u32,
    /// 阶段是否失败。
    pub failed: bool,
}

impl StageOutput {
    /// 构造"零输出"。
    #[inline]
    pub const fn new() -> Self {
        Self {
            islands_processed: 0,
            islands_skipped: 0,
            failed: false,
        }
    }

    /// 全 skipped 输出（用于 island 集合为空的合法情形）。
    #[inline]
    pub const fn all_skipped(islands_skipped: u32) -> Self {
        Self {
            islands_processed: 0,
            islands_skipped,
            failed: false,
        }
    }

    /// 全部处理完毕的成功输出。
    #[inline]
    pub const fn ok(islands_processed: u32) -> Self {
        Self {
            islands_processed,
            islands_skipped: 0,
            failed: false,
        }
    }

    /// 失败输出。
    #[inline]
    pub const fn failed() -> Self {
        Self {
            islands_processed: 0,
            islands_skipped: 0,
            failed: true,
        }
    }

    /// 总处理的 island 数（processed + skipped）。
    #[inline]
    pub const fn total(self) -> u32 {
        self.islands_processed + self.islands_skipped
    }
}

/// 阶段种类（与 `09_parallel_design.md` §6.2 的阶段名对齐）。
///
/// `Scheduler::run` 按以下顺序串行调度：
/// `Predict → Collision → Solver → Integrate`。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StageKind {
    /// 阶段 1：刚体动力学（应用外力 / 重力 / 惯性预测）。
    Predict,
    /// 阶段 2：碰撞（broad + narrow phase + contact manifold 生成）。
    Collision,
    /// 阶段 3：约束求解。
    Solver,
    /// 阶段 4：整合（位置 / 速度更新）。
    Integrate,
}

impl StageKind {
    /// 全部 4 个阶段，按调度顺序。
    pub const ALL: [Self; 4] = [Self::Predict, Self::Collision, Self::Solver, Self::Integrate];

    /// 该阶段在调度序列中的下标（0..=3）。其他值返 `None`。
    #[inline]
    pub const fn index(self) -> Option<usize> {
        match self {
            Self::Predict => Some(0),
            Self::Collision => Some(1),
            Self::Solver => Some(2),
            Self::Integrate => Some(3),
        }
    }

    /// 调度顺序中**前一个**阶段。`Predict` 返 `None`。
    #[inline]
    pub const fn predecessor(self) -> Option<Self> {
        match self {
            Self::Predict => None,
            Self::Collision => Some(Self::Predict),
            Self::Solver => Some(Self::Collision),
            Self::Integrate => Some(Self::Solver),
        }
    }

    /// 调度顺序中**后一个**阶段。`Integrate` 返 `None`。
    #[inline]
    pub const fn successor(self) -> Option<Self> {
        match self {
            Self::Predict => Some(Self::Collision),
            Self::Collision => Some(Self::Solver),
            Self::Solver => Some(Self::Integrate),
            Self::Integrate => None,
        }
    }
}

/// 阶段 trait：MVP 签名接受 `&[IslandHandle]` 输入，返回 [`StageOutput`]。
///
/// # MVP 约束
///
/// - **同步**：返回时该阶段已全部完成（无 future / callback）。
/// - **单线程**：trait 不要求 `Send` / `Sync` —— MVP 不创建工作线程。
/// - **无内部状态共享**：每个 stage 实例独立拥有自己的状态。
///
/// # v0.4+ 演进方向
///
/// - 增加 `&mut WorkerPool` 参数以支持 island-level fan-out。
/// - 增加 `&mut FrameScratch` 参数以支持每线程 arena（per
///   `09_parallel_design.md` §6.3 / `08_memory_design.md` §7.1）。
pub trait Stage {
    /// 阶段类型枚举值（用于日志 / 调试 / 统计）。
    fn kind(&self) -> StageKind;

    /// 执行阶段。
    ///
    /// `islands` —— 本帧活跃的 island 集合（已由 `gvpe-island` 产出，
    /// MVP 阶段调用方自行准备）。
    fn run(&mut self, islands: &[IslandHandle]) -> StageOutput;
}

// --- 具体阶段标记结构（空实现，作为 trait 默认实现锚点） -------------------

/// `Predict` 阶段标记。
///
/// MVP 阶段trait 方法是 `&mut self` 调用方注入具体实现。
/// 提供空实现仅为 trait object 调试与测试桩。
#[derive(Clone, Copy, Debug, Default)]
pub struct PredictStage;

/// `Collision` 阶段标记（同 [`PredictStage`] 说明）。
#[derive(Clone, Copy, Debug, Default)]
pub struct CollisionStage;

/// `Solver` 阶段标记（同 [`PredictStage`] 说明）。
#[derive(Clone, Copy, Debug, Default)]
pub struct SolverStage;

/// `Integrate` 阶段标记（同 [`PredictStage`] 说明）。
#[derive(Clone, Copy, Debug, Default)]
pub struct IntegrateStage;
