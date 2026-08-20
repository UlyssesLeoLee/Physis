# GVPE — 技术选型（基本設計書）

> 输入基线：`GVPE-DOC-00` `00_vision.md`、`GVPE-DOC-01` `01_requirements.md`。
> 作用范围：覆盖 Rust 主语言、编译器与 MSRV、构建系统、目标平台、数学与 SIMD、并发、FFI、错误处理、测试、代码质量、CI、依赖管理等**技术栈层**的决策。
> 关联约束：所有选型必须可追溯到 `GVPE-DOC-01` 中至少一条需求（FR / NFR / GPH / VEC / PERF / LIC）或 `GVPE-DOC-00` 中一条 `GVPE-PROHIBIT-*` 禁令；本文件不重复需求列表的判定细节，只做"选型 ↔ 需求"的双向映射。

## 0. 文档元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-26 |
| 文档类型 | 基本設計書 |
| 版本 | v0.1 |
| 状态 | Draft |
| 适用阶段 | MVP / Phase 1+ |
| 关联系统 | GVPE / 整体技术栈 |
| 上游文档（输入基线） | GVPE-DOC-00, GVPE-DOC-01, GVPE-DOC-04, GVPE-DOC-08, GVPE-DOC-09, GVPE-DOC-10, GVPE-DOC-15, GVPE-DOC-16 |
| 下游文档（被消费于） | GVPE-DOC-17（详细设计在实现层消费本选型基线） |
| 编写者 | — |
| 审批者 | — |

## 1. 修订历史

| 版本 | 日期 | 修改者 | 修改内容 |
|---|---|---|---|
| v0.1 | 2026-08-19 | — | 初稿：基于 `GVPE-DOC-01` v0.2 已登记的 FR / NFR / GPH / VEC / PERF / LIC 给出技术栈决策基线；明确 Rust 主语言、自研数学库与调度器、cbindgen FFI、thiserror 错误模型；附"显式拒绝清单"。 |

## 2. 文档目的

本文档是 GVPE **技术选型层面的基线文档**，回答以下问题：

- 主语言为什么是 Rust？是否考虑过 C++ / Zig / 其他候选？
- 编译器版本、MSRV、Edition 的硬性约束是什么？
- 构建系统、目标平台、crate 形态（cdylib / staticlib / rlib）如何选择？
- 数学库、SIMD、并行调度、FFI 边界、错误处理、测试、CI 各层选什么？**不**选什么？
- 与 `GVPE-DOC-16` 的许可证审查如何联动？

本文件**不**规定：

- 具体模块的 API、字段布局（见 `GVPE-DOC-17`）；
- 物理算法选型（见 `GVPE-DOC-07` / `GVPE-DOC-19` / `GVPE-DOC-22` 等）；
- 许可证审查矩阵本身（见 `GVPE-DOC-16`）。

## 3. 适用范围

- **适用阶段**：MVP 阶段强制；Phase 1+ 持续生效；进入 Phase N 需复核。
- **适用读者**：所有 crate 维护者、集成方、CI 维护者、许可证审计者。
- **强制力**：本文件所列"显式拒绝"清单与"必须通过审查"清单具有约束力；新增第三方依赖前需先在 `GVPE-DOC-16` 走通审查，并在本文件登记一次。
- **不适用**：本文件不替代 `GVPE-DOC-16` 的合规判定，二者是**上游选型 ↔ 下游审查**的协同关系，不是替代。

## 4. 术语定义

| 术语 | 释义 |
|---|---|
| MSRV（Minimum Supported Rust Version） | 库承诺支持的最低 Rust 编译器版本。 |
| Edition | Rust 的语法版本（2015 / 2018 / 2021 / 2024）；影响语法与默认 lint。 |
| `cdylib` | Rust 编译产物形态：动态链接的 C 动态库（`.dll` / `.so` / `.dylib`）。 |
| `staticlib` | Rust 编译产物形态：静态链接的 C 静态库（`.lib` / `.a`）。 |
| `rlib` | Rust 编译产物形态：Rust 内部 crate 产物，供其他 Rust crate 链接。 |
| POD | Plain Old Data；C ABI 友好的扁平结构，无 `Drop` / 隐式引用。 |
| `repr(C)` | Rust 属性，要求类型按 C 规则布局；FFI 边界强制要求。 |
| FFI | Foreign Function Interface；与其它语言互操作的接口。 |
| `cbindgen` | Mozilla 维护的工具，从 Rust 代码自动生成 C / C++ 头文件。 |
| `#[deny(...)]` | Rust 属性，将指定 lint 提升为编译错误。 |
| SIMD | Single Instruction Multiple Data；单条指令对多组数据并行操作。 |
| `portable SIMD` | Rust 官方正在标准化的跨平台 SIMD 抽象（`std::simd`，截至 2026 年仍属 nightly 评估）。 |
| work-stealing | 调度算法：空闲线程从繁忙线程的队列"偷取"任务以平衡负载。 |
| MSRV 政策 | 升级 MSRV 需文档化的策略（本文档 §7.2）。 |
| miri | Rust 官方 UB 检测器；在 CI 中可选运行以验证 `unsafe` 块。 |

## 5. 前提与约束

本节将 `GVPE-DOC-00` 与 `GVPE-DOC-01` 中影响技术选型的约束重新明示（不引入新约束）：

| 约束来源 | 约束内容 | 选型侧影响 |
|---|---|---|
| `GVPE-PROHIBIT-01` | 不得以第三方完整物理引擎作为核心 Runtime。 | 不得引入 `rapier` / `bullet` / `physx` / `jolt` / `box2d` / `nphysics` 等任何"包装即用"物理库作为核心依赖；可作为离线工具的 dev-dependency 评估。 |
| `GVPE-PROHIBIT-02` | 不得薄包装现有物理引擎并以自研名义呈现。 | 自研 crate 必须实现真正的算法路径，不允许只声明接口、底层走第三方。 |
| `GVPE-PROHIBIT-03` | 不得让图数据库执行实时物理求解。 | 图库依赖**必须** feature-gated，且**不得**出现在求解路径可达的 crate 依赖图中。 |
| `GVPE-PROHIBIT-04` | 不得让向量数据库进入 per-frame 热路径。 | 向量库依赖**必须** feature-gated，且**不得**被热路径所触达的 crate 引用。 |
| `GVPE-PROHIBIT-05` | 不得让 LLM / AI 替代基础数值物理。 | 不得引入 `tch` / `candle` / `burn` / `ort` 等推理库到仿真路径。 |
| `GVPE-PROHIBIT-06` | 不得为追求架构优雅而牺牲实时性能。 | 任何"更优雅但更慢"的替代方案默认拒绝；性能基准见 `GVPE-DOC-14`。 |
| `GVPE-NFR-002` | 热路径零 / 近零分配。 | 选型优先考虑无 GC、无隐式堆分配的语言运行时；倾向于自研容器 / arena。 |
| `GVPE-NFR-003` | 依赖方向单向（Graph / Vector / AI → Compiler → Runtime，不得反向）。 | 核心 crate 的 `Cargo.toml` 不得声明对 `gvpe-graph` / `gvpe-vector` / `gvpe-compiler` / `gvpe-inference` / `gvpe-3dgs` 的依赖；`cargo tree` 机械可验证（`AC-02`）。 |
| `GVPE-NFR-001` | Fast Mode / Deterministic Mode 必须在架构上自首版起可区分。 | 选型不能引入"隐式数据竞争 / 隐式浮点求和顺序"——避免无序并发容器与未指定归约序的并行原语。 |
| `GVPE-FR-008` | 必须支持经 C ABI 被 Unity / Unreal / Godot / 自研引擎调用，支持批处理数据交换。 | 至少一个 crate 必须输出 `cdylib` / `staticlib` 目标；头文件由 `cbindgen` 生成。 |
| `GVPE-LIC-001` | 任何图 / 向量数据库在被锁定为依赖前，必须通过 `GVPE-DOC-16` §2 矩阵全项审查。 | 本文档不重复审查逻辑，但所有"可选"依赖在引入前**必须**先在 `GVPE-DOC-16` 通过。 |

## 6. 系统架构 / 技术栈总览

GVPE 技术栈自底向上分以下层；每层选型在正文 §18 中详细论证。

| 层 | 选型 | 决策类别 |
|---|---|---|
| 主语言 | Rust（stable） | **强制**（`GVPE-PROHIBIT-01` 推论 + 见正文 §18.1） |
| 编译器 | rustc stable，MSRV 锁定（见 §7.2） | 强制 |
| 构建系统 | Cargo（rustc 内置）+ workspace | 强制 |
| crate 产物形态 | `cdylib`（`gvpe-ffi`）、`staticlib`（`gvpe-ffi`，可选）、`rlib`（其它 crate） | 强制 |
| 数值 / SIMD | 自研 `gvpe-math`；SIMD 通过 `portable SIMD`（nightly 评估）或 vendor intrinsics | 强制 |
| 并发 / 调度 | 自研 `gvpe-scheduler`（work-stealing） | 强制 |
| 异步运行时 | **不**使用（tokio / async-std / smol 全部拒绝） | 强制 |
| 内存分配 | 自研 arena / pool / slab（`gvpe-memory`）；`std` 默认分配器仅用于启动期与离线工具 | 强制 |
| FFI 头生成 | `cbindgen` | 推荐 |
| 错误处理 | `thiserror`（derive Error）；`anyhow` 仅允许在 `gvpe-inference` / 工具 crate 中 | 推荐 |
| 日志 / 追踪 | `tracing`（评估中，MVP 不强制）；`log` 不引入 | 可选 |
| 测试框架 | `cargo test`（内置）；基准 `criterion`；确定性回放自研；属性测试 `proptest`（评估） | 推荐 |
| 代码质量 | `rustfmt`（强制格式化）、`clippy::pedantic`（deny 级） | 强制 |
| UB 检测 | `miri`（CI 中对核心 `unsafe` crate 评估运行） | 可选 |
| CI | GitHub Actions（最小：stable × {Windows, Linux, macOS} × {x86_64, aarch64}） | 推荐 |
| 文档 | Markdown + `mdbook`（评估）；本轮以纯 Markdown 为主 | 可选 |
| 许可证审查 | 与 `GVPE-DOC-16` 联动；本文件不重复审查逻辑 | 强制 |

## 7. 接口设计

### 7.1 内部接口（Rust crate ↔ crate）

- 所有内部 crate 通过 `Cargo.toml` 的 `[dependencies]` 声明；依赖方向由 `GVPE-NFR-003` 强制；
- 公共 API 用 `pub` + 最小导出面；核心 trait（`PhysicsCompiler` 等，见 `GVPE-DOC-04` §4.4 / `GVPE-DOC-17`）在 `gvpe-core` 定义，所有实现 crate `impl` 之；
- 不在 `gvpe-runtime` 的公共面暴露任何 `String` / `Vec<u8>` / `HashMap`——其输入输出全部 POD（`PhysicsProfile` / `RuntimeDescriptor` / `BodySpec` 等，见 `GVPE-DOC-17` §1.2 / §1.3）。

### 7.2 外部接口（C ABI）

- 仅 `gvpe-ffi` crate 拥有 C ABI 表面（`extern "C"` 函数 + `#[repr(C)]` 类型）；
- 头文件由 `cbindgen` 从 `gvpe-ffi` 的 `cbindgen.toml` 自动生成，提交至 `gvpe-ffi/include/`，版本化随 crate 版本号；
- C ABI 边界使用 `std::panic::catch_unwind`（`GVPE-DOC-10` / `GVPE-DOC-17` §10）将 Rust panic 转化为 C 错误码；
- **不**引入 C++ binding 工具（`cxx` / `autocxx` / `bindgen` 生成的 C++ wrapper）：C ABI 须可被 C 与 C++ 双方消费，但 GVPE 不为 C++ 提供专用封装。

## 8. 数据模型

### 8.1 跨 crate 数据形态

- **POD 输入 / 输出**：`PhysicsProfile`, `RuntimeDescriptor`, `BodySpec`（`GVPE-DOC-17` §1.2 / §1.3）；`#[repr(C)]`，可跨 FFI 安全 memcpy。
- **句柄类型**：`BodyHandle`, `ConstraintHandle`, `IslandHandle`（`GVPE-DOC-17` §1.1）——世代索引（generational index）+ 长度对齐的扁平 `u32` 对；不持有引用。
- **图谱内部表示**：`NodeKind` 枚举（closed enum，`GVPE-DOC-21` §2）——> 编译期即可穷举匹配，无字符串 ID。
- **向量内部表示**：`VectorIndex`（flat-scan fallback，`GVPE-DOC-22`）——纯数值数组，无 `String` / `HashMap` 出现在热路径。

### 8.2 内存布局

- SoA / AoSoA（`GVPE-DOC-05` / `GVPE-DOC-08`）：结构体数组 / 数组的结构体按"分热 / 温 / 冷字段"拆分；
- 对齐：`#[repr(C, align(N))]` 仅在确有性能证据时引入；
- GPU 可上传性：`ConstraintRow` 与 `gvpe-memory` 的 buffer 形态必须保持扁平、可 `bytemuck::Pod`（或手工 `unsafe` `Pod` 实现）可移植（`GVPE-DOC-04` §4.6 / `GVPE-DOC-25`）。

## 9. 处理流程

### 9.1 构建与发布流程

1. 开发者 push → CI 触发 GitHub Actions；
2. CI 步骤：fmt 检查 → clippy（pedantic）→ `cargo test --all-features` → `cargo test --no-default-features --features simd-only`（验证 Graph / Vector feature-gate 出去后核心仍可运行，对应 `GVPE-FR-001`）→ `cargo bench`（criterion，记录于 `GVPE-DOC-14`）→ `cargo tree` 输出捕获到 artifact（验证 `AC-02`）→ `cargo deny check`（许可证）→ `miri`（核心 `unsafe` crate，nightly 工具链）；
3. 通过后构建 `gvpe-ffi` 的 `cdylib` / `staticlib` 产物 + `cbindgen` 头文件；
4. 产物上传至 GitHub Release（与 crate 版本号绑定）。

### 9.2 依赖引入流程

1. 提案者在本文件 §18 添加候选，附：理由、对应需求 ID、许可证、是否 feature-gated、是否进入热路径；
2. 若候选为图 / 向量数据库，**必须**先在 `GVPE-DOC-16` §2 矩阵全项通过；
3. 维护者 review；本文件修订号 +1；
4. 引入 `Cargo.toml` 后，跑 `cargo deny` 与 `cargo tree` 验证 `AC-02` 仍然成立。

## 10. 关联需求

> 本节显式登记本文件每个选型所对应的需求 ID。完整需求描述见 `GVPE-DOC-01` §6 / §7 / §9。

| 选型决策 | 关联需求 |
|---|---|
| 主语言选 Rust | `GVPE-FR-001`, `GVPE-FR-003`, `GVPE-FR-008`, `GVPE-NFR-001`, `GVPE-NFR-002`, `GVPE-NFR-003`, `GVPE-PERF-001`, `GVPE-PERF-002`, `AC-02` |
| 自研 `gvpe-math`，拒绝 glam / nalgebra | `GVPE-PROHIBIT-01`, `GVPE-PROHIBIT-02`, `GVPE-PERF-001`, `GVPE-PERF-002` |
| 自研 `gvpe-scheduler`，拒绝 tokio / rayon 作为核心调度 | `GVPE-NFR-002`, `GVPE-NFR-003`, `GVPE-PERF-002`, `GVPE-GPH-003` |
| 不使用 async runtime | `GVPE-NFR-002`, `GVPE-PROHIBIT-06`, `GVPE-GPH-003` |
| `cbindgen` + `extern "C"` + `#[repr(C)]` | `GVPE-FR-003`, `GVPE-FR-008` |
| `catch_unwind` at FFI 边界 | `GVPE-FR-008`, `AC-01` |
| `thiserror` 错误模型，拒绝 `anyhow` 进核心 | `GVPE-NFR-002`, `GVPE-PROHIBIT-06` |
| `criterion` 基准 | `GVPE-PERF-001`, `GVPE-PERF-002` |
| 自研确定性回放 harness | `GVPE-FR-001`, `GVPE-NFR-001`, `AC-01` |
| 显式拒绝的依赖（见正文 §18.13） | `GVPE-PROHIBIT-01` ~ `GVPE-PROHIBIT-06`, `GVPE-LIC-001` |
| 拒绝 copyleft / 网络条款传染型许可证 | `GVPE-LIC-001` |
| 依赖审查联动 `GVPE-DOC-16` | `GVPE-LIC-001` |

## 11. 关联文档

- **上游（输入基线）**：
  - `GVPE-DOC-00` `docs/00_foundation/00_vision.md`（总论 / 六条禁令 / 不变式）
  - `GVPE-DOC-01` `docs/00_foundation/01_requirements.md`（需求规约 / ID 权威源）
  - `GVPE-DOC-04` `docs/01_architecture/04_architecture.md`（crate map / 依赖方向）
  - `GVPE-DOC-08` `docs/01_architecture/08_memory_design.md`（arena / pool / slab）
  - `GVPE-DOC-09` `docs/01_architecture/09_parallel_design.md`（物理岛 / 调度）
  - `GVPE-DOC-10` `docs/02_modules/10_ffi_design.md`（C ABI 设计）
  - `GVPE-DOC-15` `docs/03_cross_cutting/15_testing_strategy.md`（测试策略）
  - `GVPE-DOC-16` `docs/03_cross_cutting/16_dependency_license.md`（许可证审查矩阵）
- **下游（被消费于）**：
  - `GVPE-DOC-17` `docs/04_detailed_design/17_detailed_design.md`（详细设计在实现层消费本选型）
  - `docs/README.md` 顶部文档索引

## 12. 审批签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 | — | | |
| 校对 | | | |
| 审批 | | | |

---

## 13. 正文

> 本节按"主语言 → 工具链 → 库选型 → 显式拒绝"的顺序展开。每条选型给出：决策、理由（含对应需求 ID）、候选与拒绝项、风险与缓解。

### 18.1 主语言：Rust

#### 18.1.1 决策

GVPE 核心 crate（`gvpe-math`, `gvpe-core`, `gvpe-memory`, `gvpe-shape`, `gvpe-collision`, `gvpe-dynamics`, `gvpe-constraint`, `gvpe-solver`, `gvpe-island`, `gvpe-scheduler`, `gvpe-runtime`, `gvpe-ffi`）**全部**用 **Rust（stable）**实现。其它语言只在以下位置出现：

- C ABI 边界（`gvpe-ffi` 的 `extern "C"` 函数签名）；
- 离线 / 工具脚本（Python / shell，可选，不进入发行包）；
- 基准脚本与 CI YAML（不计入"产品代码"）。

#### 18.1.2 候选与拒绝

| 候选 | 评估 | 决策 |
|---|---|---|
| **Rust** | 见 §18.1.3 | **采纳** |
| C++（C++20/23） | 成熟物理引擎生态（PhysX / Jolt / Bullet）；ABI 复杂、UB 静默、宏/模板心智负担；与 `GVPE-NFR-002` 零分配 + `GVPE-PROHIBIT-02` 自研纯洁性存在张力。 | **拒绝** |
| C（plain C11/17） | 与 C ABI 友好；但无所有权 / 借用检查，热路径易引入 use-after-free / data race；构建系统分裂；现代 SIMD 抽象贫弱。 | **拒绝** |
| Zig | 现代系统语言；与 C ABI 极佳；但 2026 年生态仍较小，物理 / 数值生态几乎为零；与 Rust 互操作（`extern "C"`）可行但增量收益不抵风险。 | **拒绝** |
| Julia | 数值原型语言；JIT 引入不可预测暂停（违反 `GVPE-PERF-002` GC-like pause 红线）；与游戏引擎 ABI 不友好。 | **拒绝** |
| Mojo / 其他 | 2026 年仍属早期评估；无成熟物理生态。 | **拒绝** |

#### 18.1.3 选择 Rust 的具体理由

- **无 GC、零成本抽象**：`GVPE-NFR-001` / `GVPE-NFR-002` / `GVPE-PERF-002` 三条都强约束热路径无 GC 暂停；Rust 编译期即决定对象生命周期。
- **所有权 + 借用检查**：编译期排除大多数 data race；与 `GVPE-NFR-003` 的"无全局可变状态"目标天然契合。
- **FFI 一等公民**：`#[repr(C)]` + `extern "C"` + `cbindgen` 自动生成 C 头，匹配 `GVPE-FR-008`。
- **类型系统强度**：`enum` 闭集合、`PhantomData` 标记、`const generics` 可实现 `GVPE-VEC-002` 要求的"类型层面可区分"的三种签名实例。
- **生态机械化验证**：`cargo tree` 可机械证明 `GVPE-NFR-003` / `AC-02`（不允许在核心 crate 依赖图中出现 `gvpe-graph` / `gvpe-vector` / `gvpe-compiler` / `gvpe-inference` / `gvpe-3dgs`）。
- **构建 / 工具链统一**：Cargo + workspace 单一来源；不引入 CMake / Make / Bazel。
- **社区与生态**：物理 / 数值 / SIMD / FFI 库丰富（虽然本项目自研核心数学与调度，但周边工具链—`criterion`, `thiserror`, `cbindgen`, `tracing`, `proptest`, `bytemuck`—成熟稳定）。

#### 18.1.4 不使用 Rust 的领域

- **GUI / 编辑器**：编辑器侧（如果未来有）允许使用其它语言（如 TypeScript + Tauri / Electron），但**不得**进入发行物理库；
- **离线工具**：脚本、CI 工具、数据导入导出工具可使用 Python / shell；
- **3DGS / ML 流水线**：`GVPE-PROHIBIT-05` 禁止 LLM 替代基础数值物理；但 3DGS 重建作为离线工具（`GVPE-DOC-13`）可使用 Python（PyTorch / OpenCV）实现，**不**与 Rust 求解器共享进程边界。

### 18.2 编译器与 MSRV

#### 18.2.1 决策

- **编译器**：`rustc` stable channel（**不**锁定 nightly）；
- **MSRV**：`1.75`（具体版本号在首次发版时确定并写入 `rust-toolchain.toml` 与 `Cargo.toml` 的 `package.rust-version`）；
- **Edition**：2021（`Cargo.toml` 中 `edition = "2021"`）；当 2024 edition 在本项目目标 MSRV 上稳定后（评估窗口 6 个月），再切换。

#### 18.2.2 候选与拒绝

- **stable**：采纳，理由：与 `cargo` 生态、`cbindgen` 工具链、`miri` 工具链兼容性最广；
- **beta / nightly**：拒绝，理由：`GVPE-PROHIBIT-06`（不牺牲稳定性 / 可复现性换取"新特性"），CI 复现性下降，crate 消费者同步 nightly 成本高。

#### 18.2.3 MSRV 升级政策

- 升级窗口：每年评估一次（写入 release notes）；
- 触发条件：旧 MSRV 不再受 Rust 官方支持、所需依赖的 MSRV 超过本项目 MSRV、`cbindgen` 工具链需新 MSRV；
- 旧 MSRV 维护期：自宣布升级起，旧 MSRV 维护 6 个月；
- 本项目不承诺支持 EOL 的 Rust 版本。

#### 18.2.4 风险与缓解

- **风险**：Rust 生态偶尔引入新行为（编译器错误信息、trait 推导差异）→ CI 锁定 stable patch 版本号；CI 矩阵中固定 `1.75.x`（具体小版本）；
- **风险**：MSRV 升级与下游集成方同步 → 在 release notes 与 CHANGELOG 显式提示。

### 18.3 构建系统：Cargo

#### 18.3.1 决策

- **构建系统**：`cargo`（Rust 官方）；
- **Workspace 结构**：`gvpe/` 根目录一个 `Cargo.toml` workspace，包含 17 个 crate（`gvpe-math` ~ `gvpe-3dgs`，见 `GVPE-DOC-04` §4.1）；
- **产物形态**：
  - `gvpe-ffi` 输出 `cdylib`（默认）+ `staticlib`（可选）；
  - `gvpe-runtime` 输出 `rlib`（被 `gvpe-ffi` 链接）；
  - 其它 crate 输出 `rlib`；
- **工具链文件**：仓库根 `rust-toolchain.toml` 锁定 stable channel；
- **不**引入：Make / CMake / Bazel / Meson / Ninja 作为主构建；`build.rs` 仅用于 `cbindgen` 调用、链接系统库（按需），不作为通用构建逻辑。

#### 18.3.2 候选与拒绝

- **Cargo**：采纳；
- **Bazel**：拒绝（与 Cargo 生态分裂，规模化收益在本项目当前阶段不显著）；
- **CMake + Cargo 子进程**：拒绝（构建路径分裂，CI 维护成本高）；
- **Nix / Nix flake**：可选（评估），但不强制；本文件不强制开发者使用 Nix。

#### 18.3.3 风险与缓解

- **风险**：`build.rs` 滥用 → 限制 `build.rs` 用途清单，写入项目根 `CONTRIBUTING.md`；
- **风险**：`cdylib` / `staticlib` 在不同 OS 上对 panic 与线程局部存储的默认行为不一致 → 显式配置 `[profile.release]` 与 `panic = "abort"`（仅在 `gvpe-ffi` 中），详见 `GVPE-DOC-10`。

### 18.4 目标平台

#### 18.4.1 决策

| 维度 | 目标 |
|---|---|
| 操作系统 | Windows 10+、Linux（glibc 2.31+）、macOS 12+（含 Apple Silicon） |
| 架构 | `x86_64`（主力）、`aarch64`（次主力，覆盖 Apple Silicon 与 ARM 服务器） |
| Rust target | `x86_64-pc-windows-msvc`, `x86_64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, `aarch64-unknown-linux-gnu` |

#### 18.4.2 不承诺

- **i686 / 32-bit x86**：不维护；如有需求方提议，需先通过 `GVPE-DOC-16` + 本文件双重评估；
- **WASM**：MVP 不支持；`gvpe-ffi` 的设计不阻止未来添加，但本轮不投入；
- **Android / iOS 移动 native ABI**：MVP 不交付；游戏引擎侧通过平台抽象层调用本库，不要求 GVPE 自身编译到移动 native；
- **Windows MSVC < 2019 / Linux glibc < 2.31 / macOS < 12**：不维护。

#### 18.4.3 风险与缓解

- **风险**：macOS universal binary 构建复杂 → 暂以 `x86_64-apple-darwin` + `aarch64-apple-darwin` 双 target + lipo 合并（CI 中评估）；
- **风险**：Windows MSVC 工具链许可 → CI 使用 GitHub Actions 默认 MSVC；文档要求开发者本地安装 Visual Studio Build Tools。

### 18.5 数学与 SIMD

#### 18.5.1 决策

- **数学库**：**自研** `gvpe-math` crate，提供：
  - 向量：`Vec2`, `Vec3`, `Vec4`（`f32` / `f64`）；
  - 四元数：`Quat`；
  - 矩阵：`Mat3`, `Mat4`（按需，行优先）；
  - 基础运算：加减乘除、点积、叉积、长度、归一化、混合；
  - 几何：旋转、平移、缩放、`Transform` 组合与求逆；
  - 投影 / 重投影（SAT 必要）；
- **SIMD**：
  - **首选** `portable SIMD`（`std::simd`）——目前 nightly 评估，**不**作为稳定依赖；
  - **次选** vendor intrinsics（`x86_64` 的 SSE / AVX2 / AVX-512；`aarch64` 的 NEON）——稳定可用，按目标平台 `cfg` 分支；
  - 标量回退：所有 SIMD 路径必须有标量实现；
- **不**依赖外部数学库作为核心：见 §18.5.2 拒绝项。

#### 18.5.2 候选与拒绝

| 候选 | 评估 | 决策 |
|---|---|---|
| 自研 `gvpe-math` | 完全可控、零外部依赖、可针对性 SIMD 优化 | **采纳** |
| `glam` | 成熟、广泛使用；外部依赖；按 `GVPE-PROHIBIT-01` 推论，任何"包装即用"的数值核心均与"自研核心纯洁性"目标冲突 | **拒绝**作为核心；可作为**离线工具** dev-dep 评估（仅工具） |
| `nalgebra` | 学术向；通用性导致零分配与 SIMD 抽象不易满足 | **拒绝**作为核心 |
| `cgmath` | 较老；维护频率低 | **拒绝** |
| `ultraviolet` | 与 glam 类似，生态较小 | **拒绝** |

#### 18.5.3 SIMD 使用边界

- 仅在**热路径**（broad phase sweep、narrow phase SAT、积分、约束求解）启用 SIMD；
- 冷路径（编辑器、加载、调试打印）**不**引入 SIMD 复杂度；
- SIMD 路径必须可被 `cargo test --no-default-features --features simd-only` 单独验证（确保 SIMD 代码本身正确，对应 `GVPE-FR-001` feature-gate 精神）。

#### 18.5.4 风险与缓解

- **风险**：`portable SIMD` 仍在 nightly → vendor intrinsics 为主，portable SIMD 作为评估项；
- **风险**：跨平台 SIMD 行为差异（NaN 处理、denormal flush）→ 制定 `gvpe-math` 浮点行为规约（见 `GVPE-DOC-05` §5 与 `GVPE-DOC-14` 性能测试），写入 `CONTRIBUTING.md`；
- **风险**：依赖外部数学库会引入许可证复杂 → 自研 + 仅依赖 Rust 标准库 + `bytemuck`（用于安全 `Pod` cast）。

### 18.6 并发与并行

#### 18.6.1 决策

- **调度器**：**自研** `gvpe-scheduler` crate，work-stealing 任务调度（详见 `GVPE-DOC-09` 与 `GVPE-DOC-17` §8）；
- **任务 DAG**：物理岛（physics island）作为并行单位；岛内顺序、岛间并行（`GVPE-DOC-09` §9）；
- **线程模型**：固定大小线程池（默认 = `num_physical_cores`），由 `gvpe-runtime` 持有；可由宿主通过 `RuntimeDescriptor.thread_pool_size` 覆盖；
- **同步原语**：仅使用 `std::sync`（`Mutex`, `RwLock`, `Arc`, `AtomicU32/U64`）与自研无锁结构；**不**使用第三方并发原语库作为热路径依赖。

#### 18.6.2 候选与拒绝

| 候选 | 评估 | 决策 |
|---|---|---|
| 自研 `gvpe-scheduler` | 完全可控；可针对物理岛模型优化；零外部依赖 | **采纳** |
| `rayon`（核心调度） | 易用、成熟；`par_iter` 抽象与本项目"以物理岛为并行单位"模型不直接对应；引入数据竞争 / 隐式共享风险 | **拒绝**作为核心调度；**可评估**作为离线工具 / 测试用例的并行化方案（dev-dep 允许） |
| `tokio` | async runtime；与热路径"无 GC pause"约束冲突；调度开销对每帧 60-240Hz 不友好 | **拒绝** |
| `async-std` / `smol` | 同 `tokio` 拒绝理由 | **拒绝** |
| `crossbeam`（channel / queue） | 评估中；若自研 work-stealing 队列需借鉴其 API，可作为内部子模块参考实现，但**不**作为公共依赖 | 评估 |
| `parking_lot` | 性能更优的 `Mutex`；但 `GVPE-NFR-003` 要求"无全局状态"——标准 `std::sync::Mutex` 已足够；引入额外 crate 增加许可证面 | **拒绝**作为核心；可作为**离线工具** dev-dep 评估 |

#### 18.6.3 不使用 async runtime 的具体理由

- `GVPE-NFR-002` 零分配 + `GVPE-PERF-002` 无 GC-like pause：async runtime 的 future 状态机在跨 await 点有栈分配 / 堆分配开销，且调度器本身有不可预测的唤醒延迟；
- `GVPE-GPH-003` 图**不得**被 per-frame 热路径查询：async 模型的"await on graph query"会反向鼓励热路径触达 Graph；
- 物理仿真本质是**每帧严格预算**的计算图，dataflow 风格（task DAG）比 async-await 更贴切。

#### 18.6.4 风险与缓解

- **风险**：自研 work-stealing 队列 bug 引入 use-after-free / data race → `miri` 在 CI 评估运行；`cargo test --all-features` 含并发测试；fuzzing（`cargo-fuzz`，评估）；
- **风险**：跨平台线程亲和性 / NUMA 差异 → 评估 `core_affinity`（dev-dep，仅工具），不作为核心依赖。

### 18.7 FFI 与 C ABI

#### 18.7.1 决策

- **C ABI 表面**：仅在 `gvpe-ffi` crate 暴露 `extern "C"` 函数 + `#[repr(C)]` 类型；
- **头文件生成**：`cbindgen`（Mozilla / `dtolnay` 维护，MPL-2.0）；
- **panic 安全**：`std::panic::catch_unwind` 在每个 C ABI 入口处捕获 panic，转化为 C 错误码（不 unwind through C 边界，对应 `GVPE-DOC-10`）；
- **句柄**：所有跨 FFI 句柄使用不透明指针（`*mut OpaqueStruct`）或 POD `u32` 索引（与 `BodyHandle` 编码兼容）；
- **不**引入 C++ binding 工具；
- **不**使用 `bindgen`（仅消费 C 头，不提供）；
- **不**使用 `cxx` / `autocxx`（无 C++ 互操作需求）。

#### 18.7.2 候选与拒绝

| 候选 | 评估 | 决策 |
|---|---|---|
| `cbindgen` | 主流、与 Cargo 集成好、配置文件可版本化 | **采纳** |
| 手工 `extern "C"` + 手工头 | 控制力最强；但维护成本高、易漂移 | 部分采纳（由 `cbindgen` 自动生成头，函数体仍手工） |
| `cxx` / `autocxx` | C++ 互操作专用；本项目无 C++ 侧需求 | **拒绝** |
| `bindgen` | 反向工具（消费 C 头）；本项目不消费 C 库 | **拒绝** |

#### 18.7.3 风险与缓解

- **风险**：`cbindgen` 与 Rust 新版本不兼容 → CI 锁定 `cbindgen` 版本号；
- **风险**：跨平台 C 调用约定差异（x86_64 SysV / Windows x64 / aarch64 AAPCS）→ 严格使用 `#[repr(C)]` + 平台无关的 POD 类型；CI 矩阵覆盖三大 OS。

### 18.8 错误处理

#### 18.8.1 决策

- **核心 crate 错误模型**：使用 `thiserror`（derive `Error` trait）定义具体错误类型（`CompileError`, `RuntimeError`, `ShapeError` 等），公开 `Result<T, ErrorType>`；
- **`anyhow`**：**不**在核心 crate 引入；**仅**允许在 `gvpe-inference`（未来）/ 离线工具 / 测试代码中使用 `anyhow::Result`；
- **panic 政策**：
  - 核心 crate（`gvpe-*` 求解相关）：**panic = bug**；`#[deny(unsafe_op_in_unsafe_fn)]` 等 deny lint 强制；
  - 测试代码：允许 `unwrap` / `expect`；
  - FFI 边界：使用 `catch_unwind` 转换为错误码（见 §18.7.1）；
- **Option vs Result**：用 `Option<T>` 表示"值可能不存在"（如查找未命中），用 `Result<T, E>` 表示"操作可能失败"（如形状不支持）。

#### 18.8.2 候选与拒绝

| 候选 | 评估 | 决策 |
|---|---|---|
| `thiserror` | 主流、零运行时开销、derive 友好、MIT/Apache-2.0 | **采纳** |
| `anyhow`（核心） | 适合应用层；抹平错误类型，与"类型化错误"目标冲突 | **拒绝**（核心） |
| `eyre` | 与 `anyhow` 类似 | **拒绝**（核心） |
| `snafu` | 上下文丰富；但增加 crate 数量与心智负担 | **拒绝**（评估中） |
| 手工 `enum Error { ... }` | 完全可控；但样板代码多 | 部分采纳（与 `thiserror` 配合，`thiserror` 解决样板） |

#### 18.8.3 风险与缓解

- **风险**：跨 crate 错误类型不兼容 → 所有核心错误类型在 `gvpe-core` crate 集中定义，公共 trait 暴露；
- **风险**：panic 泄漏 → `gvpe-ffi` 必须使用 `catch_unwind`（CI 测试覆盖）。

### 18.9 日志与追踪（评估）

#### 18.9.1 决策

- **MVP**：**不**引入日志库到核心 crate；`println!` / `eprintln!` 仅允许在 `gvpe-inference` / 离线工具 / dev-dep 中；
- **评估项**：`tracing`（`tokio-rs/tracing`，MIT）——若未来需要结构化日志（事件、性能 span），可作为 feature-gated 可选依赖；
- **不**引入 `log`（生态更老、`tracing` 已是事实标准）。

#### 18.8.2 风险与缓解

- **风险**：核心 crate 静默失败 → 显式 `Result` + 错误传播 + FFI 边界错误码即足够；日志不是 silent failure 的修复方式。

### 18.10 测试与基准

#### 18.10.1 决策

- **单元测试 / 集成测试**：`cargo test`（内置）；
- **基准**：`criterion`（`bheisler/criterion.rs`，MIT/Apache-2.0）——用于性能回归与 `GVPE-DOC-14` 性能预算验证；
- **确定性回放**：**自研** harness（`gvpe-testkit` crate 或 `tests/determinism/` 目录），记录每帧输入（seed + 初始状态），重放并逐字节比较输出——对应 `GVPE-NFR-001` / `AC-01`；
- **属性测试（评估）**：`proptest`（`AltSysrq/proptest`，MIT/Apache-2.0）——可作为 fuzz-like 测试的轻量替代；
- **Fuzzing（评估）**：`cargo-fuzz`（仅 nightly）——核心 `unsafe` crate 与序列化层评估；
- **覆盖率**：`cargo-llvm-cov`（CI 中评估）；
- **不**依赖 GUI 测试框架；游戏引擎集成测试在引擎侧处理。

#### 18.10.2 候选与拒绝

| 候选 | 评估 | 决策 |
|---|---|---|
| `cargo test` | 内置；无依赖 | **采纳** |
| `criterion` | 成熟、稳定、统计显著性检验 | **采纳** |
| `proptest` | 轻量属性测试 | **采纳**（评估后） |
| `cargo-fuzz` | libFuzzer 绑定 | 评估 |
| `quickcheck` | 类 proptest；生态较老 | **拒绝**（`proptest` 已覆盖） |
| `mockall` | mock 框架；本项目核心算法不依赖外部接口 mock，物理仿真难以 mock | **拒绝**（评估） |

#### 18.10.3 风险与缓解

- **风险**：自研 determinism harness 维护成本 → 仅覆盖关键不变量（broad phase 输出、求解器状态哈希、约束满足度），不全量重放；
- **风险**：`criterion` 与 `cargo test` 在热路径上有微小差异 → benchmark 与 test 分离，CI 跑两遍。

### 18.11 代码质量与 lint

#### 18.11.1 决策

- **格式化**：`cargo fmt`（基于 `rustfmt`），CI 必须通过；`rustfmt.toml` 锁定风格（max width = 100, tab_spaces = 4, edition = 2021）；
- **Lint**：`cargo clippy --all-targets --all-features -- -D clippy::pedantic -D clippy::nursery`（pedantic + nursery 作为 deny）；
- **`#[deny]` lint 集合**（写入 workspace 根 `src/lib.rs` 或各 crate 入口）：
  - `unsafe_op_in_unsafe_fn`（Rust 2024 行为前移）；
  - `missing_debug_implementations`（库公共类型）；
  - `missing_docs`（公共 API 强制文档化）；
  - `rust_2018_idioms`；
  - `non_ascii_idents`（中文标识符警告，但不 deny；保留与中文叙事文档的兼容性）；
- **不**强制 `clippy::restriction`（过于严苛，与实用主义冲突）；
- **Miri**：`cargo +nightly miri test` 在核心 `unsafe` crate 的 CI 中评估运行。

#### 18.11.2 风险与缓解

- **风险**：`clippy::pedantic` 升级引发大量新警告 → 升级窗口写入 release notes；先 warn 后 deny 灰度。

### 18.12 CI

#### 18.12.1 决策

- **CI 平台**：**GitHub Actions**（与项目仓库托管位置保持一致；如未来迁移 GitLab，迁移 CI 配置）；
- **CI 矩阵**（最小集）：
  - OS × arch：`windows-latest × x86_64`、`ubuntu-latest × x86_64`、`ubuntu-22.04-arm × aarch64`、`macos-latest × x86_64`、`macos-latest × aarch64`；
  - Rust 工具链：stable（与 `rust-toolchain.toml` 锁定）；
- **CI 步骤**（顺序）：
  1. `cargo fmt --all -- --check`
  2. `cargo clippy --all-targets --all-features -- -D warnings`
  3. `cargo test --all-features`
  4. `cargo test --no-default-features --features simd-only`（验证 feature-gate）
  5. `cargo test --no-default-features`（验证无任何 feature 仍可构建与运行最小子集）
  6. `cargo bench`（仅 main 分支，记录结果）
  7. `cargo tree --workspace --no-dedupe > tree.txt`（artifact，配合 `AC-02` 验证）
  8. `cargo deny check`（许可证、advisories、bans、sources）
  9. `cargo +nightly miri test`（仅核心 unsafe crate，评估）
- **不**强制自托管 runner；如未来需要更长 benchmark 时间或自托管 GPU runner，再评估。

#### 18.12.2 候选与拒绝

- **GitHub Actions**：采纳；
- **GitLab CI**：评估（如未来迁移 GitLab）；
- **Jenkins / Buildkite / CircleCI**：拒绝（生态分裂，维护成本高）；
- **Drone / Travis**：拒绝（已不推荐使用）。

### 18.13 显式拒绝清单（按需复查）

> 本节是 `GVPE-PROHIBIT-*` 在 crate / library 层的具体落地。任何后续变更都需要在 `GVPE-DOC-16` 重新审查 + 本文件更新。

| 类别 | 显式拒绝 | 理由 |
|---|---|---|
| 物理引擎 | `rapier`, `bullet-rs`, `physx-rs`, `jolt-rs`, `box2d-rs`, `nphysics`, `parry`（dimforge 系） | `GVPE-PROHIBIT-01` / `GVPE-PROHIBIT-02` |
| 物理相关 | `rapier3d`, `rapier2d`, `xpbd-parser` | 同上 |
| 数学核心 | `glam`, `nalgebra`, `cgmath`, `ultraviolet`, `simdeez`（若作为核心） | `GVPE-PROHIBIT-01` 推论 + 自研可控 |
| 异步 runtime | `tokio`, `async-std`, `smol`, `embassy`（async 部分） | `GVPE-NFR-002`, `GVPE-GPH-003`, `GVPE-PERF-002` |
| ML / 推理 | `tch`（libtorch）, `candle-core`, `candle-nn`, `burn`, `ort`（onnxruntime）, `tract` | `GVPE-PROHIBIT-05` |
| 图数据库（核心） | `neo4j-rs`, `memgraph`, `indradb`, `petgraph`（若作为核心存储） | `GVPE-PROHIBIT-03`；`petgraph` **可**作为离线工具 / dev-dep，但**不**进入核心求解路径 |
| 向量数据库（核心） | `lancedb-rs`, `qdrant-client`, `milvus`, `pinecone` | `GVPE-PROHIBIT-04` |
| 并发原语 | `parking_lot`（核心）, `crossbeam`（核心, channel 评估外） | 许可证面增加 / 隐式共享状态风险 |
| C++ binding | `cxx`, `autocxx`, `bindgen`（消费 C 头场景除外） | 无 C++ 互操作需求 |
| 错误（核心） | `anyhow`, `eyre` | 与类型化错误目标冲突 |
| 日志 | `log`（被 `tracing` 替代） | 生态老化 |
| 序列化 | `serde`（核心, `gvpe-graph` / `gvpe-vector` feature-gated 模块除外） | 核心 crate 零依赖、零反射 |
| 拷贝 / 反射 | `bytemuck`（核心布局需 `Pod` 的模块**采纳**） | 不拒绝（明确采纳） |
| 随机数 | `rand`（核心求解路径）, `fastrand`（同上） | 求解路径不依赖随机性；仅测试 / 工具使用 |
| 时间 | `chrono`, `time`, `std::time::Instant`（热路径外） | 求解路径用 frame counter；外部时间仅日志使用 |
| 网络 / IO | `reqwest`, `hyper`, `tokio`（含 io feature）, `async-channel` | 物理库**不**做网络 IO |
| 加密 | `ring`, `rustls`, `openssl` | 物理库**不**做加密 |
| 解析 | `nom`, `pest`, `combine`（核心） | 配置文件解析仅离线工具 |
| GUI | `egui`, `iced`, `tauri`, `gtk-rs` | 物理库**不**做 GUI |
| 拷贝型许可证 | `GPL-*`, `AGPL-*`, `LGPL`（静态链接场景） | `GVPE-LIC-001` + 游戏引擎分发兼容性 |
| 源代码传染 | `SSPL`, `BUSL`, `Elastic License`, `Commons Clause` | 同上 + 商业风险 |

### 18.14 附录：选型与需求 ID 速查

> 与 §10 关联需求表互补，按"选型决策"反向索引。

| 需求 ID | 对应选型（节号） |
|---|---|
| `GVPE-FR-001` | §18.1 (Rust), §18.5 (自研 math), §18.10 (determinism harness) |
| `GVPE-FR-002` | §18.5 (math), §18.6 (scheduler) |
| `GVPE-FR-003` | §18.7 (FFI), §18.8 (typed errors) |
| `GVPE-FR-004` | §18.5 (schema completeness 不在本文件，引用 GVPE-DOC-02) |
| `GVPE-FR-005` | §18.6 (scheduler 与 graph 完全分离) |
| `GVPE-FR-006` | §18.1 (Rust 类型系统), §18.5 (math) |
| `GVPE-FR-007` | §18.5 (PhysicsLOD 在数据结构层预留，引用 GVPE-DOC-17) |
| `GVPE-FR-008` | §18.7 (cbindgen + extern "C") |
| `GVPE-NFR-001` | §18.6 (无 async), §18.10 (determinism harness) |
| `GVPE-NFR-002` | §18.1 (Rust 无 GC), §18.6 (无 async), §18.8 (panic 政策) |
| `GVPE-NFR-003` | §18.3 (Cargo workspace), §18.6 (scheduler 自研), §18.13 (拒绝 tokio 等) |
| `GVPE-NFR-004` | §18.13 (许可证策略), §18.7 (依赖审查联动 GVPE-DOC-16) |
| `GVPE-GPH-003` | §18.6 (无 async), §18.13 (拒绝图数据库进核心) |
| `GVPE-VEC-001` | §18.6 (无 async), §18.13 (拒绝向量数据库进核心) |
| `GVPE-VEC-002` | §18.1 (Rust 类型系统), §18.5 (math 闭 enum) |
| `GVPE-PERF-001` | §18.1 (Rust zero-cost), §18.5 (SIMD), §18.6 (并行) |
| `GVPE-PERF-002` | §18.1 (Rust 无 GC), §18.6 (无 async), §18.8 (panic → error) |
| `GVPE-LIC-001` | §18.13 (拒绝清单), §18.7 (依赖审查联动) |
| `GVPE-PROHIBIT-01` | §18.13 (拒绝清单) |
| `GVPE-PROHIBIT-02` | §18.5 (自研 math), §18.6 (自研 scheduler) |
| `GVPE-PROHIBIT-03` | §18.13 (拒绝图数据库进核心) |
| `GVPE-PROHIBIT-04` | §18.13 (拒绝向量数据库进核心) |
| `GVPE-PROHIBIT-05` | §18.13 (拒绝 ML / 推理) |
| `GVPE-PROHIBIT-06` | §18.1 (Rust 而非 C++ 模板), §18.6 (自研 scheduler 而非"更优雅但慢"的抽象) |
| `AC-01` | §18.10 (determinism harness) |
| `AC-02` | §18.3 (cargo tree artifact), §18.6 (依赖方向机械验证) |
| `AC-03` | §18.1 (Rust repr(C) POD), §18.7 (FFI 一致性) |
| `AC-04` | 引用 `GVPE-DOC-02` §Review；本文件不重复本体评审 |

### 18.15 附录：选型变更流程

1. 提案者在本文件 §6 总览表 + §18 正文相应小节添加候选，附理由、关联 ID、许可证、feature-gate 方案；
2. 若候选为图 / 向量数据库 / 任何 LLM 相关：**必须**先在 `GVPE-DOC-16` §2 矩阵全项通过，否则不得进入本文件登记；
3. 维护者 review（PR 流程）；通过后本文件修订号 +1，候选从"评估"转为"采纳"或"拒绝"；
4. 引入 `Cargo.toml` 后，跑 `cargo deny` 与 `cargo tree` 验证 `AC-02` 仍成立；
5. CI 矩阵追加测试用例（如适用）。

### 18.16 附录：与未来阶段的衔接

- **Phase 1+**：若引入 GPU compute 后端（`NG3` 当前禁止），本文件需补充 `wgpu` / `cuda-rs` / `vulkano` 等评估节；本轮不预留选型；
- **Phase 1+**：若引入 3DGS 流水线（`NG2` 当前禁止），本文件需补充 Python / C++ 互操作评估节；本轮不预留选型；
- **Phase 1+**：若 `GVPE-DOC-16` 通过的图 / 向量数据库候选浮现，本文件 §18.13 拒绝清单可逐项解除（需附许可证 + 性能 + 维护性评估）；
- **Phase 1+**：若 Rust Edition 2024 在目标 MSRV 上稳定且 crate 生态适配，本文件 §18.2.1 升级 Edition 评估。
