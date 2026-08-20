# GVPE — 测试策略（テスト戦略）（基本設計書）

## 0. 文档元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-15 |
| 文档类型 | 基本設計書 |
| 版本 | v0.2（标准化重写版） |
| 状态 | Draft |
| 适用阶段 | MVP |
| 关联系统 | GVPE / 全部 crate |
| 上游文档（输入基线） | GVPE-DOC-01（验收标准 AC-*）、GVPE-DOC-02（`02_physics_ontology.md` §Review、ONT-ISS-001）、GVPE-DOC-14（`14_performance_budget.md` §14.5）、GVPE-DOC-05（`05_runtime_design.md` §5.3）、GVPE-DOC-03（`03_graph_schema.md` §6） |
| 下游文档（被消费于） | — |
| 编写者 | — |
| 审批者 | — |

## 1. 修订历史

| 版本 | 日期 | 修改者 | 修改内容 |
|---|---|---|---|
| v0.1 | — | [原作者] | 初稿（原始版本） |
| v0.2 | 2026-08-19 | 标准化 pass | 转为 IPA 基本設計書 格式；叙事中文化；保留全部技术内容；补充文档元数据、修订历史、术语定义、关联文档、审批签字等标准章节 |

## 2. 文档目的

本文档规定 GVPE 项目的测试层级、各项测试的设计准则、与其上游约束的对应关系。目标：

- 把验收标准（AC-01/02/03、ONT-ISS-001、§14 性能预算）逐条落到可执行测试上。
- 把“动态枚举 crate，不写死硬编码列表”作为测试设计纪律固化下来。
- 把“本体自洽性”从手稿 review 升级为可重复执行的检查。
- 明确“金标准数据集必须来自真实求解器的已知正确运行，而非手写魔数”。

## 3. 适用范围

- 适用：单元测试、确定性测试、回归 / 基准、本体自洽性 review、集成测试、依赖隔离测试、Compiler 往返测试、求解器正确性夹具。
- 不适用：模糊测试、属性测试、CI 流水线配置（见 §12）。

## 4. 术语定义

| 术语 | 说明 |
|---|---|
| Fast Mode | `05_runtime_design.md` §5.3 中定义的求解模式：关闭 Graph / Vector 特性，保证纯仿真可重复 |
| AC-01/02/03 | `01_requirements.md` 中的三项验收标准 |
| ONT-ISS-001 | `02_physics_ontology.md` §Review 列出的本体自洽性 issue |
| 已知答案物理题 | 手工可解的物理场景，期望结果通过解析法或可信参考实现预先得到 |
| 金标准数据集 | 由真实求解器的“已知正确运行”产出的夹具集，取代手写魔数 |
| 本体 review | 对 `02_physics_ontology.md` schema 自洽性的可重复检查 |
| 编译期依赖隔离 | AC-02：仿真空间 crate 不得依赖 `gvpe-graph` / `gvpe-vector` / `gvpe-compiler` |
| `cargo tree` | 列出某 crate 的传递依赖 |

## 5. 前提与约束

- AC-01 由确定性测试落实。
- AC-02 由依赖隔离测试落实。
- AC-03 由 Compiler 往返测试落实。
- ONT-ISS-001 由本体 review 检查落实。
- 性能预算（§14）由回归 / 基准测试落实。
- 任何硬编码 crate 列表的测试都不被接受——必须从 `cargo metadata` 动态枚举。

## 6. 系统架构 / 模块设计

### 6.1 测试层级

| 层级 | 覆盖范围 | 样例 |
|---|---|---|
| 单元 | 纯函数、数学、单步求解 | 已知两体场景下的 `ConstraintRow` 求解 |
| 确定性 | Fast Mode 可重复性（同构建 / 同机 / 同 seed） | AC-01 |
| 回归 / 基准 | 性能预算（§14） | criterion 套件（§14.5） |
| 本体 review | schema 自洽性 | `02_physics_ontology.md` §Review、本文 §6.4 |
| 集成 | 多 crate 场景仿真 | N 体场景达到预期静止状态 |
| 依赖隔离 | AC-02 编译期边界 | `cargo tree` 检查（继承归档 PRE 项目中“动态 vs 硬编码枚举”的教训） |

## 7. 接口设计

| 测试类型 | 入口 | 输出断言 |
|---|---|---|
| 单元测试 | `cargo test` | 已知结果一致 |
| 确定性 | 跑同一场景两遍 | `SimulationState` 每步逐 bit 一致 |
| 依赖隔离 | `cargo tree <crate>` | 不含 `gvpe-graph` / `gvpe-vector` / `gvpe-compiler` |
| 本体 review | schema 校验器 | （b）拒绝按帧 `State` 写 `gvpe-graph`；（c）MVP 未填充分支 round-trip 通过 |
| Compiler 往返 | `compile(...)` vs 手写 `PhysicsProfile` | 两者相等 |
| 求解器正确性 | 已知答案场景 | 数值容差内一致 |
| 回归 / 基准 | criterion | 阶段耗时落在预算内（当预算被定义后） |

## 8. 数据模型

| 测试对象 | 关键数据 |
|---|---|
| `SimulationState` | 逐 bit 一致性比对（确定性） |
| `PhysicsProfile` | Compiler 往返相等性 |
| `ConstraintRow` | 已知两体场景下的单步结果 |
| N 体静止场景 | 最终静止状态 |
| 本体 schema 节点 | 接受（合法关系）与拒绝（非法关系） |
| 依赖树 | 文本中是否含禁用 crate 名 |

## 9. 处理流程

1. 提交前：单元 / 集成 / 依赖隔离 / 本体 review / Compiler 往返 / 求解器正确性夹具 全绿。
2. CI：自动跑确定性测试、回归 / 基准测试。
3. 任何一项不绿即阻塞合入。
4. 本体 review 与依赖隔离作为持续门禁，不允许在 MR 中以“待修复”绕过。

## 10. 关联需求

| 需求编号 / 项 | 描述 |
|---|---|
| AC-01 | Fast Mode 确定性（同构建 / 同机 / 同 seed，逐 bit 一致） |
| AC-02 | 仿真空间 crate 不依赖 `gvpe-graph` / `gvpe-vector` / `gvpe-compiler` |
| AC-03 | `compile(populate_graph(...)) == hand_constructed_profile` |
| ONT-ISS-001 | 本体自洽性 issue：当前缺少（b）拒绝按帧 `State` 写入 `gvpe-graph`、（c）未填充分支 round-trip 通过 的检查 |
| GVPE-FR-001 | Graph / Vector 特性在 Fast Mode 下被编译掉 |
| GVPE-PROHIBIT-06 | 禁止在热路径上引入无界分配 / 锁竞争 / 可测回归（由基准测试套件守护） |

## 11. 关联文档

- `docs/01_requirements.md`（GVPE-DOC-01）：验收标准 AC-01/02/03、GVPE-FR-001
- `docs/02_physics_ontology.md`（GVPE-DOC-02）§Review、ONT-ISS-001
- `docs/03_graph_schema.md`（GVPE-DOC-03）§6：图谱查询模式（影响夹具设计）
- `docs/05_runtime_design.md`（GVPE-DOC-05）§5.3：Fast Mode 形态
- `docs/14_performance_budget.md`（GVPE-DOC-14）§14.5：基准套件落点

## 12. 明确非目标

模糊测试、属性测试、CI 流水线配置均不在本文档中规定——这些是 `gvpe-core` 真正成为代码后才该决定的实现细节，本节仅作前置指针列出，不预先指定。

## 13. 审批签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 |  |  |  |
| 校对 |  |  |  |
| 审批 |  |  |  |

---

## 14. 正文

> 原始输入基线：`01_requirements.md` 验收标准、`02_physics_ontology.md` §Review（ONT-ISS-001）、`14_performance_budget.md` §14.5。

### 13.1 测试层级

| 层级 | 覆盖范围 | 样例 |
|---|---|---|
| 单元 | 纯函数、数学、单步求解 | 已知两体场景下的 `ConstraintRow` 求解 |
| 确定性 | Fast Mode 可重复性（同构建 / 同机 / 同 seed） | AC-01 |
| 回归 / 基准 | 性能预算（§14） | criterion 套件（§14.5） |
| 本体 review | schema 自洽性 | `02_physics_ontology.md` §Review、本文 §13.4 |
| 集成 | 多 crate 场景仿真 | N 体场景达到预期静止状态 |
| 依赖隔离 | AC-02 编译期边界 | `cargo tree` 检查（继承归档 PRE 项目中“动态 vs 硬编码枚举”的教训） |

### 13.2 确定性测试（AC-01）

在同一机 / 同构建上跑同一场景两次；断言每步的 `SimulationState` 输出逐 bit 一致，Graph / Vector 特性在构建中被编译掉（GVPE-FR-001）。本测试是 `05_runtime_design.md` §5.3 中 Fast Mode 论断的具体可证伪目标。

### 13.3 依赖隔离测试（AC-02）

```
for crate in [gvpe-core, gvpe-collision, gvpe-dynamics, gvpe-constraint,
              gvpe-solver, gvpe-island, gvpe-scheduler, gvpe-runtime]:
    assert "gvpe-graph" not in cargo_tree(crate)
    assert "gvpe-vector" not in cargo_tree(crate)
    assert "gvpe-compiler" not in cargo_tree(crate)
```

必须从 `cargo metadata` 动态枚举 crate，不得写死硬编码列表——归档 PRE 项目的可追溯性矩阵曾因硬编码枚举而在新增 crate 时静默漏检；该修正模式在此以测试设计纪律的形式被重述，而非一次性教训。

### 13.4 本体 review 作为可重复检查（关闭 ONT-ISS-001）

schema 校验器执行以下三步：（a）加载 `02_physics_ontology.md` 中的节点 / 关系类型；（b）尝试向 `gvpe-graph` 做“按帧 `State` 批量写入”并断言被拒绝（即 ONT-ISS-001 当前缺失的具体强制点）；（c）将 MVP 未填充的本体分支（Energy / Wave / Field / Process / PhysicalLaw）以零实例方式走完 schema 校验，断言无校验错误。（b）与（c）同时通过即关闭 ONT-ISS-001——任一项缺失则按 `02_physics_ontology.md` 自身指示保持 open。

### 13.5 Compiler 往返测试（AC-03）

```
graph_profile = compile(populate_graph(known_material))
manual_profile = PhysicsProfile { ...same values, hand-constructed... }
assert graph_profile == manual_profile
```

### 13.6 求解器正确性夹具

已知答案物理题（两球在单轴方向上的完全弹性碰撞；盒子静止于平面直至进入 sleep 状态等），期望结果由解析法手工可解，在数值容差内核对——这是自研求解器对应的、归档 PRE 项目中 `pre-testkit` 金标准数据集纪律（夹具必须来自真实求解器的已知正确运行，而非手工魔数）的等价做法：其推理直接迁移，夹具的来源必须是真实求解器的已验证运行，而不是凭空捏造。

### 13.7 明确非目标

模糊测试、属性测试、CI 流水线配置均不在本文档中规定——这些是 `gvpe-core` 真正成为代码后才该决定的实现细节，本节仅作前置指针列出，不预先指定。
