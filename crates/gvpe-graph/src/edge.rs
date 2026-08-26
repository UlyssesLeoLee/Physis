//! 边 trait + [`EdgeId`] + [`RelationKind`] 关系词表。
//!
//! 依据 `GVPE-DOC-02` §22 关系词表——MVP 仅实现常用子集；
//! 完整词表按 `GVPE-DOC-03` §9.2 schema 演化纪律增补（每次增补需在 PR / commit
//! body 显式说明触及的 Ontology Review 类别）。

use core::fmt::Debug;

/// PKG 边 ID 类型（与 [`crate::NodeId`] 同形，独立 ID 空间）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EdgeId(
    /// 边唯一标识。
    pub u64,
);

impl EdgeId {
    /// 构造新边 ID。
    #[inline]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// 取出原始 `u64`。
    #[inline]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// 哨兵 ID：表示"无边"。
    pub const INVALID: Self = Self(0);
}

impl core::fmt::Display for EdgeId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "EdgeId(0x{:016x})", self.0)
    }
}

/// 关系词表（`GVPE-DOC-02` §22 子集 + `GVPE-DOC-03` §6.1.A 关系结构）。
///
/// ## MVP 范围
///
/// 仅实现 `gvpe-doc-03 §6.1.A` 显式列出的本体间关系；
/// 词表内词条按 `GVPE-DOC-03` §9.2 纪律在后续 PR 增补。
///
/// ## 已知缺口
///
/// - 边沿条件限定符（`GVPE-DOC-03` §8.1：`{relation: DECREASES, condition: ...}`）
///   **不在 MVP**：当前 [`Edge::relation`] 只返回枚举变体，条件限定符
///   留待边 payload 承载（详见 [`Edge`] trait doc 已知缺口段）。
/// - 反向关系（如 `HAS_MATERIAL` ↔ `MATERIAL_OF`）**不**自动派生：调用方
///   显式选择方向。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RelationKind {
    /// `Entity → Material`（`GVPE-DOC-03` §6.1.A）。
    HasMaterial,
    /// `Entity → Phase`。
    HasPhase,
    /// `Entity → State`。
    HasState,
    /// `Entity → Property`。
    HasProperty,
    /// `Entity → Process`。
    ParticipatesIn,
    /// `Entity → Interaction`。
    InteractsVia,
    /// `Entity → Field`。
    ExistsIn,
    /// `Entity → Energy`。
    Carries,
    /// `Entity → Wave`。
    Generates,
    /// `Entity → PhysicalModel`。
    ModeledBy,
    /// 反向 / 通用关联（不在 §6.1.A 显式词表，作为兜底）。
    Generic,
}

impl RelationKind {
    /// 关系名（用于诊断 / 序列化）。
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::HasMaterial => "HAS_MATERIAL",
            Self::HasPhase => "HAS_PHASE",
            Self::HasState => "HAS_STATE",
            Self::HasProperty => "HAS_PROPERTY",
            Self::ParticipatesIn => "PARTICIPATES_IN",
            Self::InteractsVia => "INTERACTS_VIA",
            Self::ExistsIn => "EXISTS_IN",
            Self::Carries => "CARRIES",
            Self::Generates => "GENERATES",
            Self::ModeledBy => "MODELED_BY",
            Self::Generic => "GENERIC",
        }
    }
}

/// PKG 边 trait。
///
/// 边 payload 类型 `Self` 由用户定义；[`crate::Graph<N, E>`] 在编译期
/// 用 `E: Edge` 约束。`relation()` / `src()` / `dst()` 是边的最低骨架。
///
/// ## 已知缺口
///
/// - 边沿条件限定符（`GVPE-DOC-03` §8.1 condition qualifier）不在 MVP trait 表面，
///   由用户自行在 payload 中承载（如 `MyEdge.condition: Option<String>`）。
/// - 不暴露"反向边"自动派生（详见 [`RelationKind`] doc 已知缺口）。
pub trait Edge: Debug + Clone {
    /// 边的稳定 ID。
    fn id(&self) -> EdgeId;

    /// 关系词表条目（`GVPE-DOC-02` §22）。
    fn relation(&self) -> RelationKind;

    /// 源节点（**有向**边：`src` → `dst`）。
    fn src(&self) -> crate::NodeId;

    /// 目标节点。
    fn dst(&self) -> crate::NodeId;

    /// 边的人类可读标签（用于诊断）。
    fn label(&self) -> &'static str;
}
