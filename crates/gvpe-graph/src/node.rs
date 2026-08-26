//! 节点 trait 与 [`NodeId`]。
//!
//! 依据 `GVPE-DOC-03` §6.2：PKG 节点承载"高语义、高连接度、具备来源 / 置信度 / 历史
//! 等溯源信息"的数据。trait [`Node`] **不**要求 `bytemuck::Pod`（PKG 非热路径 POD），
//! 但**要求** `Debug + Clone`（编译器阶段需要打印 / 复制）。

use core::fmt::Debug;

/// PKG 节点 ID 类型（与 `gvpe_core::BodyHandle` 不同——见
/// `gvpe_core::PkqEntity::Id`）。
///
/// 选择 `u64`：
/// - 高位 `u32` 给用户域 ID（持久化 / 跨进程 ID），
/// - 低位 `u32` 给本地 epoch / 写入版本（防止外部 ID 撞库时本地图误读）。
///
/// MVP 不区分高位 / 低位语义（用户可自行分配），但 ID 整体唯一。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(
    /// 节点唯一标识。
    pub u64,
);

impl NodeId {
    /// 构造新节点 ID。
    #[inline]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// 取出原始 `u64`。
    #[inline]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// 哨兵 ID：表示"无节点"（与 `gvpe_core::BodyHandle::INVALID` 同形）。
    pub const INVALID: Self = Self(0);
}

impl core::fmt::Display for NodeId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "NodeId(0x{:016x})", self.0)
    }
}

/// PKG 节点 trait。
///
/// 节点 payload 类型 `Self` 由用户定义；[`crate::Graph<N, E>`] 在编译期
/// 用 `N: Node` 约束。本 trait **不**限制字段形态（PKG 节点是自由数据载体），
/// 只保证 `Debug + Clone` 可用——编译器 / 离线查询工具的最低要求。
///
/// ## 不变量（不强制，仅约定）
///
/// - `id()` 必须稳定，跨图序列化/反序列化后**必须**保持一致（持久化前提）；
/// - 节点 payload **不应**包含 `&mut` 借用（PKG 是 shared-immutable 模型）。
pub trait Node: Debug + Clone {
    /// 节点的稳定 ID（必须等于 [`crate::Graph::add_node`] 时使用的 ID）。
    fn id(&self) -> NodeId;

    /// 节点的人类可读标签（用于诊断输出 / 编译器报告）。
    ///
    /// MVP 仅要求返回 `&'static str` 风格的标签；后续若需要动态标签可放宽。
    fn label(&self) -> &'static str;
}
