# GVPE — Vector Space Detailed Design（詳細設計書）

## 0. 文档元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-22 |
| 文档类型 | 詳細設計書 |
| 版本 | v0.2（标准化重写版） |
| 状态 | Draft |
| 适用阶段 | MVP / Phase N+ |
| 关联系统 | GVPE / gvpe-vector |
| 上游文档（输入基线） | GVPE-DOC-11（`11_vector_design.md`）、GVPE-DOC-17（`17_detailed_design.md` §12，接口阶段） |
| 下游文档（被消费于） | GVPE-DOC-13（`13_3dgs_future_design.md` §13.3 的 Hypothesis→Simulation→Comparison 回路） |
| 编写者 | — |
| 审批者 | — |

## 1. 修订历史

| 版本 | 日期 | 修改者 | 修改内容 |
|---|---|---|---|
| v0.1 | — | — | 初稿（原始版本） |
| v0.2 | 2026-08-19 | 标准化 pass | 转为 IPA 詳細設計書 格式；叙事中文化；保留全部技术内容；补充文档元数据、修订历史、术语定义、关联文档、审批签字等标准章节 |

## 2. 文档目的

本文档定义 GVPE（Graph-Governed Vector Physics Engine）向量空间（Vector Space）子系统的详细设计。具体而言，给出物理签名（Physics Signature）的确定性 V1 提取算法、向量索引（VectorIndex）的 MVP 起步实现（flat-scan 线性扫描），以及「检索仅提议、不裁决」的架构纪律。文档目标是把上游 `11_vector_design.md` 中曾以接口形式存在的占位实现，下沉为可在 `gvpe-vector` crate 中落地的具体算法与数据结构。

## 3. 适用范围

- 适用 crate：`gvpe-vector`、`gvpe-inference`（仅作为消费者提及）。
- 适用阶段：MVP / 早期规模阶段；当真实实例数据量出现且 ANN（Approximate Nearest Neighbor）技术的选型有据可依时，索引内部实现可替换，但本设计所固定的接口形态保持不变。
- 不适用：尚未有驱动用例的能量/波/场/工艺子系统（见 `23号文書`）、尚未设计的流体/FEM 接口（见 `24号文書`）。

## 4. 术语定义

- **物理知识图谱（Physics Knowledge Graph, PKG）**：以 `gvpe-graph` 为后端的语义图结构，描述物理实体、属性、过程之间的因果与构成关系。
- **物理签名（Physics Signature）**：对一个仿真快照（`SimulationStateSnapshot`）按子签名（motion / contact / material / deformation / interaction / energy / wave / field / environment / solver）聚合出的特征向量组，是向量空间的检索键。
- **向量空间（Vector Space）**：与仿真空间（Simulation-Space）正交的、按物理签名相似度检索历史实体与配置的子系统。
- **图谱空间（Graph-Space）**：与仿真空间、向量空间并列的第三空间，运行于 `gvpe-graph` 之上。
- **Flat Scan（线性扫描）**：MVP 起步的检索实现，对全部条目逐项计算相似度，O(N) 时间复杂度。
- **Fused Multi-Vector Similarity（多向量融合相似度）**：对多个子签名分别计算余弦相似度，再以可配置权重线性组合的相似度计算方式。

## 5. 模块详细设计

`gvpe-vector` crate 包含以下三个职责单一的子模块：

1. **签名提取器（Signature Extractor）**：消费 `SimulationStateSnapshot`，输出 `PhysicsSignature`。
2. **向量索引（VectorIndex）**：以 `(EntityId, PhysicsSignature)` 为条目，提供 `search` 接口；MVP 阶段实现为 flat scan。
3. **检索候选（RetrievalCandidate）**：检索结果的数据载体；只承载 `entity: EntityId` 与 `similarity: f32`，明确不携带任何「最终答案」语义。

`gvpe-vector` 的公开 API 仅接受 `&SimulationStateSnapshot`（在帧结束之后取出的 owned/borrowed 副本，见 `05_runtime_design.md` §5.5），从不接受指向 `BodyStateSoA` 的活引用（`17_detailed_design.md` §4.1）——这是类型层面的硬约束，从语法上保证 `gvpe-vector` 不会被误用在仿真中途。

## 6. 类与数据结构

### 6.1 `PhysicsSignature`

```rust
struct PhysicsSignature {
    motion: MotionSignature,
    contact: ContactSignature,
    material: MaterialSignature,
    // 其余子签名（deformation / interaction / energy / wave / field /
    // environment / solver）在 MVP 阶段全部为零值或 None。
    ..PhysicsSignature::default()
}
```

### 6.2 `VectorIndex`

```rust
struct VectorIndex {
    entries: Vec<(EntityId, PhysicsSignature)>,
}
```

### 6.3 `RetrievalCandidate`

```rust
struct RetrievalCandidate {
    entity: EntityId,
    similarity: f32,   // 仅为相似度分数，非最终答案（见 §7.3）
}
```

## 7. 算法详解

### 7.1 签名提取（确定性 V1，与归档 PRE 项目的 V1 编码器一脉相承）

```rust
fn extract_signature(snapshot: &SimulationStateSnapshot) -> PhysicsSignature {
    PhysicsSignature {
        motion: MotionSignature {
            mean_speed: mean(snapshot.bodies.iter().map(|b| length(b.linear_velocity))),
            peak_speed: max(snapshot.bodies.iter().map(|b| length(b.linear_velocity))),
            angular_energy_proxy: sum(snapshot.bodies.iter().map(|b| length_sq(b.angular_velocity))),
        },
        contact: ContactSignature {
            contact_count: snapshot.contact_events.len() as f32,
            mean_restitution: mean(snapshot.contact_events.iter().map(|c| c.restitution_estimate)),
        },
        material: MaterialSignature::from_profiles(&snapshot.profiles_used),   // 直接复制，不推断（02号文書 §5 的 Property 节点值）
        // deformation/interaction/energy/wave/field/environment/solver 子签名：
        // MVP 阶段全部为 zero / None（暂无软体、能量账本、波或场的运行期数据可提取——
        // 分别见 19/12号文書），待相应子系统产出可提取数据后再填充
        ..PhysicsSignature::default()
    }
}
```

聚合操作全部为确定性的 `mean` / `max` / `sum`，不含任何学习参数。这一选型继承自归档 PRE 项目的 V1 编码器 ADR：先做到可解释、可测试，学得型编码器是有证据后的后续升级，不在首版要求之列。

### 7.2 索引结构（fallback：flat scan；ANN 待数据量有据后再选型）

```rust
impl VectorIndex {
    fn search(&self, query: &KnownPhysicsSignature, top_n: usize) -> Vec<RetrievalCandidate> {
        // MVP / 早期规模路径：flat 线性扫描 + 每子签名余弦相似度 +
        // 通过可配置权重融合（绝不使用单一拼接向量 —— 11号文書 §11.1）
        let mut scored: Vec<_> = self.entries.iter()
            .map(|(id, sig)| (*id, fused_similarity(&query.0, sig, &DEFAULT_WEIGHTS)))
            .collect();
        scored.sort_by(|a, b| b.1.total_cmp(&a.1));
        scored.truncate(top_n);
        scored.into_iter().map(RetrievalCandidate::from).collect()
    }
}

fn fused_similarity(a: &PhysicsSignature, b: &PhysicsSignature, w: &SignatureWeights) -> f32 {
    w.motion * cosine(&a.motion.as_vec(), &b.motion.as_vec())
        + w.contact * cosine(&a.contact.as_vec(), &b.contact.as_vec())
        + w.material * cosine(&a.material.as_vec(), &b.material.as_vec())
        // 其余子签名项，对应生产者尚未实现时权重为 0
}
```

`flat scan` 是刻意的起点（而非疏漏）——`11_vector_design.md` §11.5 明确推迟 ANN 技术选型，直到有真实实例数据可供设计索引时再决定；本文档的职责在于保证：当实现从 flat scan 切换为真正的 ANN 结构时，接口（`search`）形态不发生改变。

### 7.3 检索永不裁决，仅提议

`RetrievalCandidate` 在全代码库范围内不被直接作为 ground truth 消费——它仅喂给 `gvpe-inference` 的 Hypothesis→Simulation→Comparison 回路（`13_3dgs_future_design.md` §13.3），真正的物理答案由该回路在仿真中验证。这与归档 PRE 项目的「retrieval proposes, physics verifies」原则完全一致，因 GVPE 的向量空间在该项目中承担的是同一架构角色，故于此重申。

## 8. 错误处理

签名提取与 flat scan 检索均为纯计算，不涉及 IO、不分配新缓冲区（仅借用既有 `Vec`）。故此模块自身不产生可恢复错误：

- 空索引：`search` 直接返回空 `Vec`，调用方需自行处理「无候选」分支。
- 退化签名向量：余弦相似度对零向量返回 `NaN` 或 `0.0`，视实现而定；上层比较逻辑需容忍 `NaN` 排序。
- 任何跨 crate 错误（如图谱层校验失败）由 `gvpe-graph` / `gvpe-compiler` 处理，本模块不引入新的错误类型。

## 9. 性能考量

- `extract_signature` 对 `snapshot.bodies` 与 `snapshot.contact_events` 各做一次线性遍历，时间复杂度 O(N+M)；运行频率为每帧一次或更低（签名仅在帧结束后才提取）。
- `VectorIndex::search` 为 O(N · k) flat scan，其中 k 为子签名数量。MVP 阶段条目数若进入十万级，应切换为 ANN 实现（接口不变）。
- `gvpe-vector` 严格非热路径：其输入是帧末的 `SimulationStateSnapshot` 副本，类型上无法被任何 mid-step 调用触及；故 `GVPE-VEC-001`（签名提取不得阻塞仿真）与 `GVPE-VEC-002`（索引检索不得阻塞仿真）在编译期即被保证。
- 零分配要求：MVP 实现不引入 `Box` / `HashMap`，所有中间数据驻留在 `Vec` 中。

## 10. 测试考量

- **确定性回环测试**：同一 `SimulationStateSnapshot` 在两次 `extract_signature` 调用中必须输出位一致的 `PhysicsSignature`。
- **fused_similarity 单调性测试**：固定 `a`、`w` 后，对 `b` 沿某子签名方向单调变化时，总分应单调变化。
- **flat scan 排序稳定性测试**：相同 `similarity` 的多个候选在 `truncate(top_n)` 之后需具备稳定顺序（实现需采用稳定排序或显式 `EntityId` tie-break）。
- **接口兼容性测试**：`VectorIndex::search` 的签名在 ANN 实现替换前后必须保持字节一致（由 trait 抽象或等价 fuzz 测试覆盖）。
- **非热路径不变量测试**：尝试从 `gvpe-dynamics` mid-step 调用 `gvpe-vector` 的公开 API 应当编译失败（用 `trybuild` 类静态断言测试覆盖）。

## 11. 关联需求

- **GVPE-VEC-001**：签名提取不得阻塞仿真热路径——由 `&SimulationStateSnapshot` 独占输入的类型设计保证。
- **GVPE-VEC-002**：索引检索不得阻塞仿真热路径——同上。
- **`11_vector_design.md` §11.1–§11.5**：原仅以 trait 接口占位的能力，现已具备具体实现。
- **`00_vision.md` §0.5 完备性**：向量空间作为与仿真/图谱并立的第三空间，具备可落地的算法深度。

## 12. 关联文档

- 上游：`docs/00_vision.md` §0.5、`docs/02_modules/11_vector_design.md`、`docs/02_modules/17_detailed_design.md` §12。
- 平级：`docs/02_modules/23_energy_wave_field_process_algorithms.md`（能量/波/场/工艺运行期算法，签名提取的子签名消费者）。
- 下游：`docs/02_modules/13_3dgs_future_design.md` §13.3（Hypothesis→Simulation→Comparison 回路消费 `RetrievalCandidate`）。

## 13. 审批签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 | | | |
| 校对 | | | |
| 审批 | | | |

---

## 14. 正文

> 以下为原文档正文，章节编号保留（§22.1, §22.2, §22.3, §22.4），叙事已并入上方对应章节，本节保留原始技术片段与原文档的引用脉络。

### 22.1 签名提取（确定性 V1 — 镜像归档 PRE 项目的 V1 编码器选型，此处重述以承接其推理：可解释优先、学习模型次之）

参见上方 §7.1。原始代码片段完整保留于 §7.1。

> 原文片段（输入基线声明）：
> Input baseline: `11_vector_design.md`, `17_detailed_design.md` §12 (interface-only).
>
> Deterministic aggregation (mean/max/sum), no learned parameters — same rationale the archived PRE project's ADR gave for its V1 encoder: interpretable and testable first, a learned encoder is a later, evidence-driven upgrade, not a first-pass requirement.

### 22.2 索引结构（fallback：flat scan；ANN 待数据量有据后再选型）

参见上方 §7.2。原始代码片段完整保留于 §7.2。

> 原文片段（设计意图声明）：
> A flat scan is the deliberate starting point (not an oversight) — `11_vector_design.md` §11.5 explicitly defers ANN technology choice until there's real instance data to design an index against; this document's job is only to make sure the *interface* (`search`) doesn't change shape when the *implementation* swaps from flat scan to a real ANN structure later.

### 22.3 检索永不裁决，仅提议

参见上方 §7.3。`RetrievalCandidate` 仅携带 `entity` 与 `similarity`，明确标记「非最终答案」。

### 22.4 非热路径强制（GVPE-VEC-001）

参见上方 §5 与 §9。`gvpe-vector` 公开 API 仅接受 `&SimulationStateSnapshot`，类型层面保证无法被 mid-step 误调。

> 原文片段（需求满足声明）：
> Requirements satisfied: GVPE-VEC-001/002, `11号文書` §11.1–§11.5 (all now have concrete implementations behind their previously interface-only traits).
