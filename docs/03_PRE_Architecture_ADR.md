# PRE 架构决策记录（ADR）

## 文书管理表

| 项目 | 内容 |
|---|---|
| 文书编号 | PRE-DOC-03 |
| 版本 | v0.1 |
| 状态 | Draft |
| 关联文书 | 02_PRE_Basic_Design.md（各 ADR 在设计书中被引用的位置见其正文） |

## 改订履历

| 版本 | 日期 | 变更内容 |
|---|---|---|
| v0.1 | 2026-08-17 | 初版：ADR-001～ADR-008 |
| v0.1.1 | 2026-08-17 | 补充文书管理表 |

---

## ADR-001: Why Physical Response rather than Material Label?

**状态**：Accepted

**背景**：核心索引对象是用材料名称（"steel"/"rubber"）还是物理响应（条件+作用→响应）？

**决策**：以 Physical Response（条件+激励→响应）为核心索引对象，材料标签仅作为 metadata 过滤字段。

**理由**：材料标签是离散、粗粒度、且跨材料仍可能共享相似动力学行为（如某些橡胶与某些软体组织在特定应变范围内响应相似）；反之同名材料在不同参数下响应差异巨大。以响应为核心可以捕捉这种连续性，支持跨材料类比检索，直接服务于 inverse physics 的目标（观察到响应、反推可能的材料而非反过来）。

**后果**：Signature/Embedding 设计必须围绕响应特征，而非材料分类特征；材料标签退化为 filter 维度（PRE-VEC-003）。

---

## ADR-002: Why Retrieval is Candidate Generation Only?

**状态**：Accepted

**决策**：向量检索永远只产生候选（Top-N/Top-K），最终判定必须经过 Simulation Verification。

**理由**：Embedding 相似度是学习/工程近似，不具备物理保真度保证；直接以 cosine similarity 作为最终答案违反 "Physics is ground truth" 原则（需求第31/39节），且无法检测 embedding 空间的错误近邻（false neighbor）。将检索限定为候选生成，把最终正确性判定交还给可信的正向仿真，是本项目区别于"纯 AI 检索系统"的核心设计立场。

**后果**：Pipeline 必须包含 Verification 阶段（不可选）；性能上多一轮仿真开销，但换取正确性保证（对应 H4 待验证假设）。

---

## ADR-003: Why Standard Response is Solver Independent?

**状态**：Accepted

**决策**：引入 Raw Solver State → Standard Physical Response → Physical Signature → Physical Embedding 四层分离，其中 Standard Response 层与具体 solver 解耦。

**理由**：若 Signature/Encoder 直接依赖某个 solver 的内部状态格式（如 XPBD 约束残差数组、MPM 粒子/网格双表示），则新增 solver 需要改动检索与编码核心代码，违反插件化目标（PRE-NFR-001），也使跨 solver 检索（H2）在架构上不可能。四层分离将"solver 特定"与"物理语义"显式切割，转换逻辑收敛在每个 solver 插件自己的 `to_standard_response` 实现中。

**后果**：每个新 solver 插件必须实现该转换，工程成本前置到插件开发阶段；但换来检索/编码/存储核心代码零改动扩展性。

---

## ADR-004: Why 3DGS is Observation Backend?

**状态**：Accepted

**决策**：Dynamic 3DGS/4DGS 仅作为 `ObservationBackend` 的一种未来实现，产出必须落到与仿真侧相同的 `StandardPhysicalResponse` schema，核心 Runtime 不感知 3DGS 存在。

**理由**：3DGS 是观测/重建技术，其表示（Gaussian 数量、协方差、拓扑）与物理语义无直接映射关系，且技术本身快速演进（3DGS→4DGS→未来方法）。若核心数据模型绑定 3DGS 表示，未来技术更替将引发核心重构。将其限定为 Observation Backend 的一种实现，是 PRE-FR-015 与非目标 NG2（不实现 3DGS/4DGS 重建或渲染管线本身）所要求的"禁止强耦合"的直接落实。

**后果**：V0.1 不实现该 backend，仅预留 trait；Phase 2 设计必须证明"能把 3DGS 输出转成 Standard Response 且保留足够物理信息"，否则该抽象需重新评估。

---

## ADR-005: Single Vector vs Multi Vector?

**状态**：Accepted（MVP 采用简化版 Multi-Vector：3 个语义子向量 + 1 个融合用 global_vector）

**背景**：单一 embedding 是否足以表达全部物理性质？

**决策**：采用 Multi-Vector（behavior/deformation/temporal + global），而非单一向量；但 MVP 刻意只取三个语义子空间（而非需求文档列出的六个），并用一个 global_vector 承担 ANN 粗召回索引角色。

**理由**：单一向量会把运动学、形变、时序等异质信息压缩进同一距离空间，容易造成"总体相似但物理上错误近邻"（如运动轨迹相似但材料行为完全不同）。但完全体的六子向量在 MVP 阶段验证成本过高（每多一个子空间即多一套消融实验与索引维护）。三个语义子向量是"能支撑 H1/H2 消融实验的最小可验证集合"，global_vector 用于控制 ANN 索引维度和粗召回效率，是工程折衷而非否定完全体设计。

**后果**：contact_vector / material_vector / constraint_vector 留待 Phase 2，根据 MVP 阶段消融实验结果决定是否拆分（若 behavior_vector 内部出现明显的检索质量瓶颈则拆出 contact_vector 等）。

---

## ADR-006: Relational + Vector + Blob storage strategy?（含图数据库结论）

**状态**：Accepted

**决策**：MVP 采用「Relational(metadata) + Vector(embedding) + Blob(原始响应)」三分存储，三者通过 `experience_id` 关联；**不引入图数据库**。

**理由**：
- metadata（solver 类型、材料类别、验证状态等）具有明确 schema、需要精确过滤查询 → 关系型/文档型最合适。
- embedding 需要近似最近邻查询，语义与前者完全不同 → 专用向量索引。
- 原始响应（逐帧场数据）体积大、访问模式是"按 id 整体读取"而非"查询" → blob/对象存储最合适，且必须与前两者物理分离以满足 PRE-DATA-002（metadata/embedding 查询不触发大体积读取）。
- 图数据库：需求文档要求"证明关系型/向量存储无法满足查询模式后才可引入"（PRE-DATA-004）。MVP 规模与查询模式（主要是"按条件过滤 + 向量检索"，无深度多跳关系遍历需求）不构成引入图数据库的证据。Entity/Relation/Field 若真需要表达为图，MVP 阶段用关系表的外键 + JSON 字段可以覆盖。

**后果**：若 Phase 3（Inverse Physics 复杂约束/接触图）出现明确的多跳查询需求（例如"找出所有通过约束链间接影响此刚体的场"），需重新开 ADR 评估图数据库引入。

---

## ADR-007: Metadata Filtering — Post-filter vs Pre-filter

**状态**：Accepted（MVP：post-filter）

**决策**：MVP 使用 post-filter：先对 `global_vector` 做 ANN Top-N 召回，再按 metadata 过滤到 Top-K。

**理由**：Pre-filter（先按 metadata 分区再各分区建索引）在 metadata 组合数量增长时会造成索引数量爆炸和维护复杂度上升；MVP 规模（10K~100K）下 post-filter 的"召回不足风险"可以通过适当放大 N 缓解，工程复杂度显著更低。

**后果**：若线上验证发现某些稀有 metadata 组合导致 post-filter 后候选不足（Top-N 召回后过滤剩余过少），需要评估分区索引或加大 N 的动态策略，记为 Evolution Strategy 触发条件之一。

---

## ADR-008（合并说明）: 图数据库暂缓引入

见 ADR-006 后半部分，不单独重复展开。
