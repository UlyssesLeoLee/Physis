# PRE Testkit（自动测试用 Mock 项目）基本设计书

## 文书管理表

| 项目 | 内容 |
|---|---|
| 文书编号 | PRE-DOC-11 |
| 版本 | v0.1.1 |
| 状态 | Draft |
| 输入基线 | 10_PRE_Testkit_Requirements.md |
| 关联文书 | 12（Testkit 详细设计书）、08（核心详细设计，本文书引用其 trait 定义）、09（测试用例一览） |

## 改订履历

| 版本 | 日期 | 变更内容 | 作成者 |
|---|---|---|---|
| v0.1 | 2026-08-17 | 初版：架构定位、双层测试策略、组件设计、依赖方向 | Claude |
| v0.1.1 | 2026-08-17 | 新增 §3.7 跨适配层一致性套件（PRE-ENG-008）的归属与实现要点；`MockBevyHarness` 泛化说明 | Claude |

## 承认栏

| 角色 | 承认日期 | 签署 |
|---|---|---|
| Rust系统负责人（ST-03） | — | 未承认 |

---

## 1. Architecture Overview

`pre-testkit` 是一个 leaf crate（不被任何生产代码依赖），依赖 `pre-core` 以复用核心类型定义（`PhysicsExperience` 等），并（可选）依赖各核心 crate 的 trait 定义以提供对应的 mock 实现：

```
                         pre-core（类型定义，被所有人依赖）
                               ▲
              ┌────────────────┼────────────────┐
              │                │                │
        pre-solver-api    pre-atlas(trait)   pre-bevy(可选)
              ▲                ▲                ▲
              │                │                │
              └──────────── pre-testkit ─────────┘
                     （MockSolverPlugin / InMemoryAtlas /
                      FixtureBuilder / GoldenDataset /
                      MockBevyHarness / approx_eq helpers）
                               ▲
                               │ [dev-dependencies] only
              ┌────────────────┼────────────────┐
        pre-core测试代码   pre-solver-*测试代码  pre-bevy测试代码  ...（各 crate 自身的 #[cfg(test)] 或 tests/ 目录）
```

依赖方向的关键约束（对应 PRE-TK-001）：箭头方向必须保证 `pre-testkit` 不出现在任何 `[dependencies]`（生产依赖）中，只出现在 `[dev-dependencies]`。这与 ADR-009 对 `pre-bevy` 的依赖面控制是同一类架构纪律的重复应用——每次引入一个新的"横切关注点 crate"，都要明确回答"谁能依赖它、它能依赖谁"。

## 2. 双层测试策略（Testing Strategy 补充，对应 08号文档 §27 的细化）

08 号文档 §27 Testing Strategy 已提出"单元测试/数值回归测试/集成测试/假设验证实验"四层，`pre-testkit` 的引入使其中的"单元测试"与"集成测试"进一步分化为显式的两级：

| 测试层级 | 依赖对象 | 速度 | 目的 | 典型用例 |
|---|---|---|---|---|
| **Mock 层**（testkit） | `MockSolverPlugin`/`InMemoryAtlas`/Golden Dataset | 秒级，CI 主力 | 验证逻辑正确性：数据结构变换、算法分支、错误处理路径、边界情形 | 09号文档中标注"testkit"的 TC-* |
| **真实层**（集成测试） | 真实 `pre-solver-rigid/xpbd/mpm`、真实 SQLite+HNSW、真实 Bevy App | 分钟级，CI 中独立 job 或 nightly | 验证数值行为正确性：真实物理是否合理、真实存储是否正确落盘、真实 Bevy 渲染是否可用 | TC-E2E-*、TC-SOLVER 中依赖真实数值断言的用例（如 TC-SOLVER-001 恢复系数） |

**强制要求**：任何被 Mock 层测试覆盖的行为，若该行为涉及"是否正确反映物理/存储真实语义"（而非"是否正确处理某个数据结构分支"），必须在真实层至少有一条对应的集成测试兜底。09 号文档中，`TC-SOLVER-001`/`002`/`004`（物理正确性断言）应保持针对真实 solver 运行，不应被 mock 替代；而 `TC-SOLVER-003`/`005`（错误处理路径）适合用 `MockSolverPlugin` 精确构造。09 号文档的用例逐条标注见其 v0.1.2 改订记录。

## 3. Component Design

### 3.1 MockSolverPlugin

实现 `SolverPlugin` trait（08号文档 §3.1），内部持有一个 `ResponseScript` 枚举，决定 `init()`/`step()` 的行为：

```
ResponseScript
├── Fixed(Vec<RawSolverState>)            # 固定返回预设序列，用于确定性测试
├── DivergeAtStep(u32)                    # 在第 N 步返回 NaN/Inf，用于 TC-SOLVER-005
├── FailOnInit(SolverError)               # init() 直接失败，用于参数校验类测试
├── FailAtStep(u32, SolverError)          # 第 N 步返回错误，用于 TC-VER-006
```

### 3.2 InMemoryAtlas

实现与真实 `pre-atlas`（08号文档 §9）相同的存储访问 trait，内部用三个独立的内存结构分别对应 relational/vector/blob 三类存储（保留三者分离的语义，呼应 OQ-TK-01 的初步倾向）：

```
InMemoryAtlas
├── metadata: HashMap<ExperienceId, ExperienceMetadata>   # 对应 relational store
├── vectors: HashMap<EncoderVersion, HashMap<ExperienceId, EmbeddingSet>>  # 对应 vector store
├── blobs: HashMap<ExperienceId, StandardPhysicalResponse> # 对应 blob store
└── blob_read_counter: AtomicUsize   # 供 TC-ATLAS-002 断言"仅查询 metadata 不触发 blob 读取"
```

`blob_read_counter` 是本设计相对真实 `pre-atlas` 的额外可观测钩子——真实实现不需要这类计数器（生产代码不为测试目的增加复杂度），但 mock 实现可以自由添加，因为它只服务于测试断言。

### 3.3 FixtureBuilder

对 08号文档 §2 的每个核心类型提供构造器，采用链式 API：

```
FixtureBuilder::experience()
    .with_solver(SolverId::Xpbd)
    .with_material(MaterialSpec::default_cloth())
    .build()   // 返回带有合理默认值的 PhysicsExperience
```

默认值来源：优先复用 Golden Dataset 中已有条目的字段作为默认值模板，而不是另造一套"builder 专用默认值"，减少两套数据的维护成本与相互漂移风险。

### 3.4 GoldenDataset

以静态数据文件形式（格式留 12 号文档确定，如 RON 或 JSON）随 `pre-testkit` crate 一起提交入库，运行时由 `GoldenDataset::load()` 反序列化，不在测试运行期动态生成（保证同一份数据被反复使用、任何变化都在 git diff 中可见）。

### 3.5 MockHostHarness（宿主测试宿主，以 Bevy 为首个实现）

封装 `bevy::app::App`，提供确定性时间推进：

```
MockBevyHarness::new()
    .with_plugin(PrePlugin)
    .spawn_landmark_entities(response)
    .advance_frames(60, dt=1.0/60.0)   // 虚拟推进 60 帧，不依赖真实 sleep
    .assert_transform(landmark_id, expected_position, epsilon)
```

### 3.6 跨适配层一致性套件（PRE-ENG-008）

02号文档 §33.7 规定一致性套件的实现归属 `pre-testkit`。要点：

- 输入为 Golden Dataset 中的固定响应条目（§3.4），避免套件自身引入随机性。
- 参考值来自 `pre-engine-api::conformance_reference()`（08号文档 §18.4），而非某个适配层的输出——若以某适配层为基准，该适配层自身的缺陷就会被固化为"正确答案"。
- 必须包含**非恒等换算**用例（以 `SpatialConvention::UNREAL` 构造，无需真实 Unreal 环境）。仅测 Bevy/Godot 无法暴露换算路径缺陷，因为两者换算近似恒等（ADR-011 后果条）。
- Tier 1 适配层可在 mock 层运行本套件；真实宿主环境下的集成仍需真实层兜底（§2 双层测试策略）。

### 3.7 近似相等断言辅助

一组独立的、不依赖任何 mocking 框架的手写宏/函数（呼应 PRE-TK-010 的"避免重量级依赖"要求），统一 `f64`/`Vec3`/向量的容差比较。

## 4. Determinism 与版本绑定策略

Golden Dataset 的生成必须绑定当前的 `schema_version`/`encoder_version`（08号文档 §2.3, §8.3）。当核心 schema 演进（新增/修改字段）时：

1. 若变更向后兼容（如新增可选字段）：Golden Dataset 可保持不变，`pre-testkit` 加载逻辑需处理版本差异。
2. 若变更不兼容（如字段语义改变）：必须重新生成 Golden Dataset 并同时提交新旧版本对比说明，CI 检测到版本不匹配时应使相关测试显式失败而非静默跳过（PRE-TK-008）。

## 5. 依赖方向与 Feature 设计

参考 08号文档 §18.1 对 `pre-bevy` 的 feature 拆分思路，`pre-testkit` 同样按需拆分 feature，避免不需要 Bevy mock 的测试也被迫编译 `bevy`：

```toml
[features]
default = ["solver-mock", "atlas-mock", "fixtures"]
solver-mock = []
atlas-mock = []
fixtures = []
bevy-mock = ["bevy", "pre-bevy"]   # 仅 pre-bevy 的测试需要启用
```

## 6. Evolution Strategy

- 若 Mock 层与真实层的测试结果出现系统性不一致（mock 通过但真实层持续失败，或反之），说明 mock 的行为脚本已经偏离真实实现的契约，需要重新审视 `ResponseScript`/`InMemoryAtlas` 的实现是否仍准确反映 trait 契约——这属于"testkit 自身的正确性维护"，不属于产品缺陷。
- 若 Golden Dataset 规模需要扩大（如新增 solver 类型），扩展方式是追加新条目而非重写现有条目，保持历史回归基线的稳定性。
