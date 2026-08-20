# GVPE — Collision Design（基本設計書）

## 0. 文档元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-06 |
| 文档类型 | 基本設計書 |
| 版本 | v0.2（标准化重写版） |
| 状态 | Draft |
| 适用阶段 | MVP |
| 关联系统 | GVPE / gvpe-collision, gvpe-shape, gvpe-constraint |
| 上游文档（输入基线） | GVPE-DOC-01（00_vision.md §0.1/§0.2），GVPE-DOC-04（04_architecture.md），GVPE-DOC-02（02_physics_ontology.md） |
| 下游文档（被消费于） | GVPE-DOC-07（07_solver_design.md），GVPE-DOC-09（09_parallel_design.md），GVPE-DOC-11（11_vector_design.md，待定） |
| 编写者 | — |
| 审批者 | — |

## 1. 修订历史

| 版本 | 日期 | 修改者 | 修改内容 |
|---|---|---|---|
| v0.1 | — | — | 初稿（原始版本） |
| v0.2 | 2026-08-19 | 标准化 pass | 转为 IPA 基本設計書 格式；叙事中文化；保留全部技术内容；补充文档元数据、修订历史、术语定义、关联文档、审批签字等标准章节 |

## 2. 文档目的

本基本設計書定义 GVPE 碰撞模块（`gvpe-collision`）在粗筛（Broad Phase）与精筛（Narrow Phase）两阶段的算法选型、形状支持范围、接触流形数据结构，以及面向未来 broad-phase 替换的接口稳定性要求。文档目的是把 `01_requirements.md` GVPE-FR-002 落地为可执行的模块设计，并保证碰撞输出仅以运行时约束（而非图节点）形式流入求解器。

## 3. 适用范围

本文件适用于 `gvpe-collision` crate 以及 `gvpe-shape`（形状描述）crate。`gvpe-constraint` 中消费 `ContactManifold` 的接口设计参见 GVPE-DOC-07 `07_solver_design.md`。本文件不约束 broad-phase 内部具体实现细节（实现期 spike 决定），但约束其对外接口契约（参见 §7.2）。

## 4. 术语定义

| 术语 | 说明 |
|---|---|
| 粗筛（Broad Phase） | 通过廉价空间划分从全体 body 中筛出可能相交的候选对 |
| 精筛（Narrow Phase） | 对候选对进行精确的几何相交判定 |
| SAP（Sweep and Prune） | 沿坐标轴扫描并剪枝的粗筛算法，适合帧间运动大致连贯的场景 |
| Dynamic AABB Tree | 动态构建的 AABB 层次包围体树 |
| BVH（Bounding Volume Hierarchy） | 通用包围体层次 |
| Spatial Hash | 空间哈希，适合均匀密度场景 |
| SAT（Separating Axis Theorem） | 分离轴定理，用于凸体精确相交判定 |
| GJK（Gilbert-Johnson-Keerthi） | 凸体距离 / 相交迭代算法 |
| EPA（Expanding Polytope Algorithm） | 与 GJK 配对的穿透深度计算 |
| ContactManifold | 精筛输出，描述两个 body 之间的接触点集合 |
| ContactPoint | 单个接触点：位置、法向、穿透深度 |
| `ContactConstraint` | 接触约束的求解器行（详见 GVPE-DOC-07 §2） |
| `BodyHandle` / `BodyIndex` | body 的运行时句柄 / 索引 |

## 5. 前提与约束

1. 上游基线：本文件以 `00_vision.md` §0.1/§0.2 为自研基线 —— 核心碰撞库不自研替换为任何第三方库；可能仅在非核心部分引入第三方。
2. 上游基线：本文件以 `04_architecture.md` 中 `gvpe-collision` crate 定义为架构输入。
3. 接触输出的形态契约：精筛输出必须流入 `gvpe-constraint` 的 `ContactConstraint` 行，**绝不**作为图的 `Constraint` 节点存在（依据 `02_physics_ontology.md` §9 的绑定规则）。
4. 接口稳定约束：`gvpe-collision` 对外暴露的接口（参见 §7.2）必须支持在不修改 `gvpe-constraint` / `gvpe-solver` 的前提下整体替换粗筛算法。
5. MVP 粗筛算法选型最终决定推迟到实现期 spike，候选表见 §6.1。

## 6. 系统架构 / 模块设计

### 6.1 粗筛（候选算法，MVP 选其一 —— 选型推迟到实现期 spike）

| 算法 | 适配场景 | MVP 选型依据 |
|---|---|---|
| SAP（Sweep and Prune） | 适合帧间运动大致连贯 | 从零自研实现正确性风险最低，作为首选 |
| Dynamic AABB Tree | 通用场景，中等复杂度 | 强力的次选 |
| BVH | 与 AABB Tree 相似，构建更复杂 | 暂缓 |
| Spatial Hash | 适合均匀密度场景 | 暂缓；如 MVP 场景为网格化则重新评估 |

- MVP 优先实现 SAP（从零自研求解器的最低实现风险）。
- 接口形态设计上保证 Dynamic AABB Tree 可在不改动精筛的前提下整体替换 SAP。

### 6.2 精筛形状范围（MVP 子集以粗体标识）

**Sphere**, **Box**, **Plane**, Capsule, Convex Hull, Triangle Mesh, Heightfield, Compound。

### 6.3 精筛算法

- **SAT（Separating Axis Theorem）**：MVP 内 Box-Box、Box-Plane、Sphere-Box 的主选算法。
- **GJK**：为凸包支持保留，post-MVP。
- **EPA**：与 GJK 配对使用，凸包落地后再启用穿透深度计算。

## 7. 接口设计

### 7.1 接触流形数据结构

精筛输出馈入 `gvpe-constraint` 的 `ContactConstraint` 行（参见 GVPE-DOC-07 §2）—— 绝不会是图的 `Constraint` 节点（依据 `02_physics_ontology.md` §9 的绑定规则）。

```rust
struct ContactManifold {
    body_a: BodyHandle,
    body_b: BodyHandle,
    points: SmallVec<[ContactPoint; 4]>,
}
struct ContactPoint { position: Vec3, normal: Vec3, penetration: f32 }
```

### 7.2 面向未来 broad-phase 替换的接口稳定性

`gvpe-collision` 对外仅暴露：

```rust
fn broad_phase(bodies: &[Aabb]) -> Vec<(BodyIndex, BodyIndex)>
```

- 算法选择完全封装在 `gvpe-collision` 内部。
- §6.1 "MVP 选 SAP" 决定可被整体回退而无需触碰 `gvpe-constraint` / `gvpe-solver`。

## 8. 数据模型

- `ContactManifold`：body 对 + 接触点集合。
- `ContactPoint`：位置 `Vec3`、法向 `Vec3`、穿透深度 `f32`。
- `Aabb`：粗筛输入轴对齐包围盒。
- `BodyIndex`：body 索引（用于粗筛输出元组）。

## 9. 处理流程

粗筛 → 精筛 → `ContactManifold` → 流入 `gvpe-constraint` 的 `ContactConstraint` 行 → 求解器。各阶段与执行图（GVPE-DOC-03 §1.C、GVPE-DOC-09 §9.2）的对应关系由 `09_parallel_design.md` 维护。

## 10. 关联需求

| 需求 ID | 中文描述 | 在本文件中的落地点 |
|---|---|---|
| GVPE-FR-002 | 碰撞检测为自研核心，第三方仅在非核心位置引入 | §5.1 前提；§6.1 算法选型 |
| 00_vision.md §0.1/§0.2 | 物理 / 求解 / 编译器 / 碰撞为核心自研 | §5.1 前提 |
| 02_physics_ontology.md §9 | 图节点与运行时约束的绑定规则（运行时约束不是图节点） | §7.1 接触流形契约 |

## 11. 关联文档

- 上游：`docs/00_vision/00_vision.md`（GVPE-DOC-00），`docs/01_architecture/04_architecture.md`（GVPE-DOC-04），`docs/01_architecture/02_physics_ontology.md`（GVPE-DOC-02）
- 下游：`docs/02_modules/07_solver_design.md`（GVPE-DOC-07），`docs/01_architecture/09_parallel_design.md`（GVPE-DOC-09）
- 平行引用：`docs/01_architecture/05_runtime_design.md`（GVPE-DOC-05），`docs/01_architecture/08_memory_design.md`（GVPE-DOC-08）

## 12. 审批签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 |  |  |  |
| 校对 |  |  |  |
| 审批 |  |  |  |

---

## 13. 正文

Input baseline: `04_architecture.md` (`gvpe-collision`), `01_requirements.md` GVPE-FR-002.
Self-developed per `00_vision.md` §0.1/§0.2 — no third-party collision library vendored as the core.

## 6.1 Broad phase (candidates, MVP picks one — decision deferred to implementation spike)

| Algorithm | Fit | MVP choice rationale |
|---|---|---|
| Sweep and Prune (SAP) | good for mostly-coherent motion frame-to-frame | simplest to self-implement correctly first |
| Dynamic AABB Tree | good general-purpose, moderate complexity | strong second candidate |
| BVH | similar to AABB tree, more complex build | deferred |
| Spatial Hash | good for uniform density scenes | deferred, revisit if MVP scenes are grid-like |

MVP implements SAP first (lowest implementation risk for a from-scratch solver), with the
interface shaped so Dynamic AABB Tree can replace it without touching narrow phase.

## 6.2 Narrow phase — shapes (MVP subset bolded)

**Sphere**, **Box**, **Plane**, Capsule, Convex Hull, Triangle Mesh, Heightfield, Compound.

## 6.3 Narrow phase — algorithms

- **SAT** (Separating Axis Theorem): primary for Box-Box, Box-Plane, Sphere-Box in MVP.
- **GJK**: reserved for convex hull support (post-MVP).
- **EPA**: reserved, pairs with GJK for penetration depth once convex hulls land.

## 6.4 Contact manifold

Output of narrow phase feeds `gvpe-constraint`'s `ContactConstraint` rows (`07_solver_design.md`
§2) — never a graph `Constraint` node (`02_physics_ontology.md` §9's binding rule).

```rust
struct ContactManifold {
    body_a: BodyHandle,
    body_b: BodyHandle,
    points: SmallVec<[ContactPoint; 4]>,
}
struct ContactPoint { position: Vec3, normal: Vec3, penetration: f32 }
```

## 6.5 Interface stability for future broad-phase swap

`gvpe-collision` exposes only `fn broad_phase(bodies: &[Aabb]) -> Vec<(BodyIndex, BodyIndex)>` to
the rest of the engine — algorithm choice is fully internal, so §6.1's "MVP picks SAP" decision is
reversible without touching `gvpe-constraint`/`gvpe-solver`.
