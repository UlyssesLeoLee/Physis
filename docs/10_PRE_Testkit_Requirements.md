# PRE Testkit（自动测试用 Mock 项目）需求定义书

## 文书管理表

| 项目 | 内容 |
|---|---|
| 文书编号 | PRE-DOC-10 |
| 文书名称 | PRE Testkit 需求定义书 |
| 版本 | v0.1.1 |
| 状态 | Draft |
| 输入基线 | 08_PRE_Detailed_Design.md（v0.1.3）、09_PRE_Test_Case_List.md（v0.1.4） |
| 关联文书 | 11（Testkit 基本设计书）、12（Testkit 详细设计书）、09（测试用例一览，标注哪些 TC 依赖本项目） |
| 定位说明 | `pre-testkit` 是 PRE 的**卫星子项目**（satellite sub-project）：只服务于自动化测试，不是 PRE 运行时的一部分，不对外发布为生产依赖。其需求 ID 使用独立前缀 `PRE-TK-*`，**不**并入 01_PRE_Requirements.md 的核心需求体系与 04_PRE_Traceability_Matrix.md 的追溯矩阵——理由见第4节系统边界。 |

## 改订履历

| 版本 | 日期 | 变更内容 | 作成者 |
|---|---|---|---|
| v0.1 | 2026-08-17 | 初版：PRE-TK-001~010，MVP 范围、验收标准 | Claude |
| v0.1.1 | 2026-08-17 | 随多宿主重构更新：新增 PRE-TK-011（跨适配层一致性套件执行框架）；Mock Bevy Harness 泛化为 Mock Host Harness；旧 PRE-BEVY-* 引用更新 | Claude |

## 承认栏

| 角色 | 承认日期 | 签署 |
|---|---|---|
| Rust系统负责人（ST-03） | — | 未承认 |

---

## 1. 项目背景

09_PRE_Test_Case_List.md 定义了 77 条测试用例（TC-CORE/SOLVER/SIG/ENC/ATLAS/RET/VER/REF/GEN/ENG/BEVY/GODOT/CONF/GPU/EMB/E2E），其中相当一部分若直接对着真实组件写测试，会引入以下问题：

- **速度**：真实 MPM/XPBD solver 的一次仿真可能耗时数秒~数十秒（对应 PRE-PERF-001 的 MVP 基准），若每个单元测试都跑一次真实仿真，测试套件运行时间会随用例数线性增长到不可接受的程度。
- **确定性边界外的噪声**：真实 SQLite/HNSW 索引读写涉及文件系统状态，并行测试之间若不严格隔离临时目录，容易产生难以复现的测试间污染。
- **难以构造边界情形**：TC-SOLVER-005（数值发散检测）、TC-VER-006（仿真失败）等用例需要"仿真过程中出错"这一条件，若不能对 solver 的行为进行编程式控制（脚本化），构造这类边界输入会非常困难甚至不可能（如何让真实 MPM solver 精确地在第 N 步发散？）。
- **回归基线漂移风险**：TC-SIG-005（确定性）、TC-RET-002（已知相似记录召回）等用例需要"已知正确答案"的固定数据，若每次测试都临时生成，无法察觉 `extract_signature`/`encode_v1` 逻辑的意外改动导致的结果漂移。

`pre-testkit` 的目的是为这类测试提供**快速、确定性、可编程控制**的替代实现（mock/fixture/golden dataset），把"验证逻辑正确性"与"验证真实 solver/存储/引擎的性能与集成正确性"分离为两个独立的测试层级（见第2节非目标、11号文档 §2 的双层测试策略）。

## 2. 项目目标

- G1：为 09 号文档中标注"适合 mock"的用例提供可复用的测试基础设施，使这些用例可在秒级内于 CI 中完成，不依赖真实文件系统持久化、真实网络、真实仿真耗时。
- G2：提供可编程控制的 Mock SolverPlugin，能够精确构造"数值发散""仿真失败""特定形变模式"等边界情形，覆盖真实 solver 难以稳定复现的测试场景。
- G3：提供固定的"黄金数据集"（golden dataset），作为 `extract_signature()`/`encode_v1()` 等确定性函数的回归基线。
- G4：提供 Fixture Builder，降低测试代码中构造 `PhysicsExperience`/`StandardPhysicalResponse` 等深层嵌套结构体的样板代码量。
- G5：提供 Mock Host Harness（首个实现为 Bevy），使 TC-BEVY-*/TC-GODOT-*/TC-CONF-* 系列用例可以在无真实窗口/渲染后端的 headless 环境下于 CI 中运行。

## 3. 非目标

- NG-TK1：`pre-testkit` **不替代**真实组件的集成测试。TC-E2E 系列用例仍必须至少有一层针对真实 `pre-solver-*`/`pre-atlas`/`pre-retrieval` 的端到端测试（对应 11号文档 §2 的"双层测试策略"），mock 只加速单元/组件级测试，不能验证真实数值行为是否正确（这是 mock 与真实实现的固有差距，必须被测试策略显式承认而非掩盖）。
- NG-TK2：不对生产环境提供任何 API 或二进制产物；`pre-testkit` 只能出现在 workspace 成员的 `[dev-dependencies]` 中。
- NG-TK3：不追求 Mock SolverPlugin 复现真实物理行为的数值精度——它的职责是"按脚本产生指定的 `RawSolverState` 序列"，不是"正确地做物理仿真"。
- NG-TK4：不为本卫星子项目建立独立的 CI/CD 流水线或独立发布节奏；`pre-testkit` 随主 workspace 一起构建、测试、演进。

## 4. 系统边界（含 ID 命名空间独立的理由）

`pre-testkit` 是纯粹的开发期工具，不出现在任何生产二进制的依赖树中——这一点与各宿主适配层（`pre-bevy`/`pre-godot` 等，生产环境可选依赖，见 01号文档 §29）有本质区别：适配层服务的是"PRE 的使用者"，`pre-testkit` 服务的是"PRE 的开发者"。因此：

- `PRE-TK-*` 需求不计入 01 号文档的核心需求体系，也不出现在 04 号文档的追溯矩阵中——追溯矩阵的目的是"证明核心产品需求被设计与测试覆盖"，而 `pre-testkit` 本身是测试基础设施，不是被追溯的对象，把它混入核心矩阵会稀释矩阵的信噪比。
- `pre-testkit` 的需求-设计追溯改为在本文书与 11/12 号文档内部自洽维护（见 12 号文档末尾的内部映射表），并在 09 号文档中以标注形式说明哪些 TC 依赖它（见 09 号文档改订记录）。

## 5. Functional Requirements

- **PRE-TK-001**：`pre-testkit` 必须是独立 crate，只能以 `[dev-dependencies]` 形式被其它 workspace 成员引用；`cargo build --release`（不含 `--dev`/测试 target）产出的任何生产二进制不得包含 `pre-testkit` 的代码。
- **PRE-TK-002**：必须提供 `MockSolverPlugin`，实现与真实 solver 相同的 `SolverPlugin` trait（08号文档 §3.1），其行为由一个可编程的 `ResponseScript` 描述（如：固定返回某个预设的 `RawSolverState` 序列；在第 N 步注入 NaN；在 `init()` 阶段直接返回错误），使测试可以精确构造 08号文档 §15 错误码表中列出的各类边界情形。
- **PRE-TK-003**：必须提供 `InMemoryAtlas`，实现与真实 `pre-atlas`（08号文档 §9）相同的存储访问接口，但完全基于内存 `HashMap`，不接触文件系统/SQLite/HNSW 索引文件；每个测试用例可创建一个全新的空实例，测试之间零状态泄漏。
- **PRE-TK-004**：必须提供 `PhysicsExperience`/`StandardPhysicalResponse`/`PhysicalSignature`/`EmbeddingSet`（08号文档 §2）的 Fixture Builder，采用 Builder 模式，所有字段有合理默认值，测试代码只需覆盖关心的字段。
- **PRE-TK-005**：必须提供一个固定的 Golden Dataset：不少于 10 条预先生成并检查入库的 `PhysicsExperience`（含其 `PhysicalSignature`/`EmbeddingSet`），覆盖 Rigid/XPBD/MPM 三类 solver 各至少 2 条，用作 `extract_signature()`/`encode_v1()` 的确定性回归基线（对应 TC-SIG-005 的精神，但用固定数据集而非临时生成，使得任何实现变更导致的结果漂移能被 diff 直接看到）。
- **PRE-TK-006**：必须提供近似相等断言辅助函数/宏（如 `assert_approx_eq!(a, b, epsilon)`、`assert_vec3_approx_eq!`），统一测试代码中的浮点数值比较容差，避免各测试用例各自发明不一致的比较逻辑。
- **PRE-TK-007**：必须提供 Mock Host Harness（首个实现 `MockBevyHarness` 封装 `bevy::app::App`），提供 `advance_frames(n, dt)` 以虚拟时钟推进指定帧数而不依赖真实 wall-clock sleep，使 TC-BEVY-*/TC-GODOT-* 可在 headless CI 环境下确定性运行。
- **PRE-TK-011**：必须提供跨适配层一致性套件（PRE-ENG-008）的执行框架：以 Golden Dataset 为输入、以 `pre-engine-api::conformance_reference()` 为参考值，且必须包含非恒等换算用例（`SpatialConvention::UNREAL`），详见 11号文档 §3.6。
- **PRE-TK-008**：Golden Dataset 与其它 fixture 数据必须携带其对应的 `schema_version`/`encoder_version`（08号文档 §2.3, §8.3）；当核心 schema 版本升级时，`pre-testkit` 的 CI 检查必须能够检测到"fixture 版本落后于当前 schema 版本"并使相关测试显式失败（而不是让过期 fixture 静默通过测试）。

## 6. Non-Functional Requirements

- **PRE-TK-009**（推奨）：Golden Dataset 的生成过程必须是可复现的——即存在一个固定种子的生成脚本，任何人可从零重新生成并得到逐字节相同的数据集；这保证黄金数据集不是"手工编造的魔法数字"，而是可审计、可重新生成的。
- **PRE-TK-010**（推奨）：`pre-testkit` 引入的编译时间增量应保持在可接受范围内（不引入额外的重量级依赖，如不为了 Mock 而引入完整的 mocking 框架宏系统，优先用手写的简单实现）——理由与 ADR-009/ADR-010 对适配层依赖面控制的精神一致：测试基础设施本身也应避免成为拖慢开发迭代的负担。

## 7. MVP 范围

第一版 `pre-testkit` 覆盖 PRE-TK-001~008 与 PRE-TK-011（功能需求全部），PRE-TK-009/010（非功能）作为工程纪律要求同步落实，不单独分期。不纳入 MVP：为 FEM stub / Phase 2 的宿主场景导入方向（PRE-ENG-011）预先构造 fixture——待这些能力本身进入实现阶段时再扩展 testkit。

## 8. Acceptance Criteria

- **AC-TK-01**：09 号文档中标注"使用 pre-testkit"的全部用例，在 CI 中单次运行总耗时（不含编译）应控制在数秒级，且可在无网络、无真实文件系统持久化状态的容器环境中稳定通过。
- **AC-TK-02**：`MockSolverPlugin` 能够精确复现 TC-SOLVER-005（数值发散于指定步）与 TC-VER-006（仿真失败）两个此前依赖真实 solver 难以稳定构造的边界用例。
- **AC-TK-03**：Golden Dataset 的重新生成脚本执行两次，产出的数据文件逐字节相同（验证 PRE-TK-009 的可复现性要求）。
- **AC-TK-04**：`cargo build --release`（仅构建生产 target，不构建测试）产出物的依赖树中不包含 `pre-testkit`（验证 PRE-TK-001 的边界约束，方法与 AC-06 验证 PRE-ENG-002 的 `cargo tree` 检查同构）。

## 9. Risks（简要，完整登记见 12 号文档内部风险说明）

- Mock 与真实实现行为出现漂移（mock 认为通过，真实实现实际有 bug）——这正是 NG-TK1 明确"不能替代集成测试"的原因，缓解手段是保持双层测试策略，而非试图让 mock 做到完全保真。
- Golden Dataset 本身编码了某个时间点的"正确答案"，若该答案本身是错的（如早期 `extract_signature()` 实现有隐藏 bug 而被固化进黄金数据集），后续正确的修复反而会被回归测试拦下——缓解手段：Golden Dataset 的更新必须伴随明确的 PR 说明"为什么改变是预期的"，不允许静默更新。

## 10. Open Questions

- OQ-TK-01：`InMemoryAtlas` 是否需要模拟真实 `pre-atlas` 的 metadata/blob 分离读取行为（PRE-DATA-002 的"metadata 查询不触发 blob 读取"），以便 TC-ATLAS-002 也能用 mock 版本测试？初步倾向：需要，理由是该约束是架构级要求，mock 若不模拟这一分离，会让测试对该约束失去覆盖能力；具体实现方式留 12 号文档详细设计阶段决定。
