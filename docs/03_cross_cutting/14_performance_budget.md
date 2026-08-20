# GVPE — 性能预算（性能予算）（基本設計書）

## 0. 文档元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-14 |
| 文档类型 | 基本設計書 |
| 版本 | v0.2（标准化重写版） |
| 状态 | Draft |
| 适用阶段 | MVP |
| 关联系统 | GVPE / `gvpe-runtime`（含所有仿真空间子 crate） |
| 上游文档（输入基线） | GVPE-DOC-01（GVPE-PERF-*）、GVPE-DOC-05（`05_runtime_design.md` §5.5）、GVPE-DOC-00（`00_vision.md` §0.3） |
| 下游文档（被消费于） | GVPE-DOC-15（`15_testing_strategy.md`） |
| 编写者 | — |
| 审批者 | — |

## 1. 修订历史

| 版本 | 日期 | 修改者 | 修改内容 |
|---|---|---|---|
| v0.1 | — | [原作者] | 初稿（原始版本） |
| v0.2 | 2026-08-19 | 标准化 pass | 转为 IPA 基本設計書 格式；叙事中文化；保留全部技术内容；补充文档元数据、修订历史、术语定义、关联文档、审批签字等标准章节 |

## 2. 文档目的

本文档规定 GVPE 在不同空间（Simulation / Vector / Graph）下的目标频率、MVP 场景下的性能基线、按阶段拆分的预算表、回归政策以及基准测试套件的强制要求。目标：

- 在仿真空间（Simulation Space）给出可证伪的 MVP 性能基线，避免“胜利提前宣告”。
- 明确“不预先为任何阶段填写数字”——所有阶段预算都必须先有基准测试套件再有数。
- 把性能回归当作正确性 bug 同等对待（与 `GVPE-PROHIBIT-06` 对齐）。

## 3. 适用范围

- 适用：仿真主循环及其各阶段（Apply Forces / Broad Phase / Narrow Phase / Contact Generation / Island Build / Constraint Solve / Integrate / CCD）、Vector / Graph 空间的频次约束、MVP 场景基线。
- 不适用：未来扩展（Energy / Wave / Field / Process / 3DGS 反演）的性能预算——其驱动用例出现之前不预设数。

## 4. 术语定义

| 术语 | 说明 |
|---|---|
| 仿真空间（Simulation Space） | 帧内驱动的核心求解链路，对应 `step(dt)` |
| 向量空间（Vector Space） | 物理签名提取与检索链路 |
| 图谱空间（Graph Space） | `gvpe-graph` 离线 / 工具节拍下的写入与查询 |
| 性能预算 | 各阶段允许的最大耗时份额；总和不得突破端到端帧预算 |
| 阶段预算 | `step(dt)` 内单阶段（如 `Constraint Solve`）的最大耗时 |
| 回归基线 | 上一轮基准测试的测量结果，作为下次比对基准 |
| 基准测试套件 | criterion 或同等能力的微基准测试集合 |

## 5. 前提与约束

- 满足 `GVPE-PERF-001`：MVP 场景基线（数百刚体、60Hz、单核）。
- 满足 `GVPE-PERF-002`：回归政策（性能回归按 bug 处理）。
- 满足 `GVPE-VEC-001`：向量空间不进入热路径。
- 满足 `GVPE-GPH-003`：图谱空间不进入每帧路径。
- 满足 `GVPE-PROHIBIT-06`：不得以可读性或架构便利为名接受热路径上的无界分配 / 锁竞争 / 测得回归。

## 6. 系统架构 / 模块设计

### 6.1 各空间目标频率

| 空间 | 目标频率 | 依据 |
|---|---|---|
| 仿真空间 | 60–240Hz | `00_vision.md` §0.3 |
| 向量空间 | 1–30Hz 或事件触发 | 绝不进入热路径（GVPE-VEC-001） |
| 图谱空间 | 离线 / 工具节拍 | 绝不进入每帧路径（GVPE-GPH-003） |

### 6.2 MVP 场景预算（GVPE-PERF-001）

基线目标：在多线程未被记功之前，单个中端 CPU 核上稳定支撑数百个动态刚体（球 / 盒 / 平面）跑 60Hz。这是一个故意保守、易被证伪的数字——目的是尽早有一个可以被击穿的数，而不是提前宣告胜利。

### 6.3 各阶段预算拆分（待测量，不得假设）

```
step(dt) budget @ 60Hz = 16.6ms
  Apply Forces        : measure
  Broad Phase          : measure
  Narrow Phase[]        : measure
  Contact Generation    : measure
  Island Build           : measure
  Constraint Solve[]      : measure (dominant cost, expect largest share)
  Integrate[]              : measure
  CCD                       : measure (MVP: near-zero, feature not implemented)
```

没有任何一个阶段在此被赋予具体目标数——除非已存在能测量它的基准套件。在 `15_testing_strategy.md` 的基准套件就位之前就填上虚构的预算数，正是本文档集整体在其它地方都在避免的那种“无根据的精度”。

## 7. 接口设计

| 接口方向 | 形态 | 备注 |
|---|---|---|
| 基准测试套件 → 各阶段 | criterion 风格微基准 | 见 §6.3 中各阶段独立测量 |
| 基准测试套件 → 端到端 | MVP 规模场景基准 | 见 §6.2 |
| CI / 回归系统 → 历史基线 | 上轮测量快照 | 见 §6.4 |

## 8. 数据模型

性能预算本身不引入新数据模型；其消费对象为：

| 对象 | 用途 |
|---|---|
| `SimulationState` | 各阶段输入 / 输出 |
| `ConstraintRow[]` | 约束求解阶段的输入（被预期占最大份额） |
| 各阶段耗时采样 | 来自基准套件的统计输出 |

## 9. 处理流程

1. 基准套件对 §6.3 中每一阶段做独立测量。
2. 端到端场景基准对 §6.2 的 MVP 规模做整体测量。
3. 任何提交若引入热路径上无界分配、热路径结构上的锁竞争、或相对历史基线的可测回归（阈值待定），即触发 §6.4 回归政策。
4. 性能回归按 bug 处理，登记跟踪，直至修复并复测。

## 10. 关联需求

| 需求编号 | 描述 |
|---|---|
| GVPE-PERF-001 | MVP 场景基线：数百刚体 / 60Hz / 单核 |
| GVPE-PERF-002 | 性能回归按 bug 处理；不得以可读性或架构便利为名接受 |
| GVPE-VEC-001 | 向量空间不进入热路径 |
| GVPE-GPH-003 | 图谱空间不进入每帧路径 |
| GVPE-PROHIBIT-06 | 禁止以“方便”为名在热路径上引入无界分配、锁竞争或可测回归 |

## 11. 关联文档

- `docs/00_vision.md`（GVPE-DOC-00）§0.3：仿真空间 60–240Hz 的总目标
- `docs/01_requirements.md`（GVPE-DOC-01）GVPE-PERF-001/002
- `docs/05_runtime_design.md`（GVPE-DOC-05）§5.5：运行时各阶段拆解
- `docs/15_testing_strategy.md`（GVPE-DOC-15）§15.4：基准套件落点

## 12. 基准测试套件强制要求（喂给 `15_testing_strategy.md`）

覆盖 §6.3 中每一阶段的 criterion 或同等能力微基准测试套件，外加 MVP 规模（§6.2）的端到端场景基准，必须在 AC-01（`01_requirements.md`）被视为“可验证的”而非“仅被断言的”之前就位。

## 13. 审批签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 |  |  |  |
| 校对 |  |  |  |
| 审批 |  |  |  |

---

## 14. 正文

> 原始输入基线：`01_requirements.md` GVPE-PERF-*、`05_runtime_design.md` §5.5。

### 14.1 各空间目标频率

| 空间 | 目标频率 | 依据 |
|---|---|---|
| 仿真空间 | 60–240Hz | `00_vision.md` §0.3 |
| 向量空间 | 1–30Hz 或事件触发 | 绝不进入热路径（GVPE-VEC-001） |
| 图谱空间 | 离线 / 工具节拍 | 绝不进入每帧路径（GVPE-GPH-003） |

### 14.2 MVP 场景预算（GVPE-PERF-001）

基线目标：在多线程未被记功之前，单个中端 CPU 核上稳定支撑数百个动态刚体（球 / 盒 / 平面）跑 60Hz。这是一个故意保守、易被证伪的数字——目的是尽早有一个可以被击穿的数，而不是提前宣告胜利。

### 14.3 各阶段预算拆分（待测量，不得假设）

```
step(dt) budget @ 60Hz = 16.6ms
  Apply Forces        : measure
  Broad Phase          : measure
  Narrow Phase[]        : measure
  Contact Generation    : measure
  Island Build           : measure
  Constraint Solve[]      : measure (dominant cost, expect largest share)
  Integrate[]              : measure
  CCD                       : measure (MVP: near-zero, feature not implemented)
```

没有任何一个阶段在此被赋予具体目标数——除非已存在能测量它的基准套件。在 `15_testing_strategy.md` 的基准套件就位之前就填上虚构的预算数，正是本文档集整体在其它地方都在避免的那种“无根据的精度”。

### 14.4 回归政策（GVPE-PERF-002）

任何提交若引入热路径上无界分配、热路径结构上的锁竞争、或相对历史基线的可测回归（阈值待定），即视为性能 bug，按与正确性 bug 同等流程登记——不得以可读性或架构便利为名接受（直接落实 GVPE-PROHIBIT-06）。

### 14.5 基准测试套件强制要求（喂给 `15_testing_strategy.md`）

覆盖 §14.3 中每一阶段的 criterion 或同等能力微基准测试套件，外加 MVP 规模（§14.2）的端到端场景基准，必须在 AC-01（`01_requirements.md`）被视为“可验证的”而非“仅被断言的”之前就位。
