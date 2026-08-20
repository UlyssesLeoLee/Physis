# GVPE — GPU Compute Backend Detailed Design（詳細設計書）

## 0. 文档元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-25 |
| 文档类型 | 詳細設計書 |
| 版本 | v0.2（标准化重写版） |
| 状态 | Draft |
| 适用阶段 | Phase N+（接口预留，MVP 不交付 `gvpe-gpu`） |
| 关联系统 | GVPE / gvpe-gpu（接口预留）, gvpe-runtime（依赖 trait object） |
| 上游文档（输入基线） | GVPE-DOC-04（`04_architecture.md` §4.6 仅约束性陈述）、GVPE-DOC-17（`17_detailed_design.md` §4.1 CPU 数据布局）、GVPE-DOC-00（`00_vision.md` §0.3 跨引擎兼容性）、GVPE-DOC-10（`10_ffi_design.md` 主机集成面） |
| 下游文档（被消费于） | GVPE-DOC-05（`05_runtime_design.md` §5.3 DeterminismMode 表新增 GPU 调度规则） |
| 编写者 | — |
| 审批者 | — |

## 1. 修订历史

| 版本 | 日期 | 修改者 | 修改内容 |
|---|---|---|---|
| v0.1 | — | — | 初稿（原始版本） |
| v0.2 | 2026-08-19 | 标准化 pass | 转为 IPA 詳細設計書 格式；叙事中文化；保留全部技术内容；补充文档元数据、修订历史、术语定义、关联文档、审批签字等标准章节 |

## 2. 文档目的

本文档对 `04_architecture.md` §4.6 中关于 GPU 兼容性的约束（尚无 `gvpe-gpu` crate 存在、缓冲类型须为 flat/alignable）作**数据布局层**的深化：把"不妨碍 GPU 接入"从一句**断言**变为可被结构/算法深度审视的**审计**。MVP 不交付 `gvpe-gpu`（`01_requirements.md` NG2），但完备性要求设计须达到与 `17_detailed_design.md` 同等的结构/算法深度，并能与 CPU 数据模型相互对照——而非仅以"不会排斥它"一句话了事。

## 3. 适用范围

- 适用 crate：`gvpe-gpu`（仅 trait 接口预留，无实现）、`gvpe-runtime`（通过 trait object 间接依赖）。
- 适用阶段：Phase N+；MVP 不交付，启用须通过 Cargo feature `gpu-solver`（默认关闭）。
- 不适用：GPU 宽相/窄相碰撞检测（明确不在 GPU 求解器范围）；任何具体 compute shader 源码（本文档集不交付）。

## 4. 术语定义

- **SoA（Structure of Arrays，数组结构）**：本设计的所有状态容器（`BodyStateSoA` 等）所采用的数据布局——字段各成连续数组，利于向量化与 GPU 上传。
- **AoSoA（Array of Structures of Arrays）**：在 SIMD 友好的子块粒度上重复 SoA 的混合布局。
- **POD（Plain Old Data）**：与 C ABI 兼容、可平凡复制/对齐的简单数据形态。
- **`#[repr(C)]`**：Rust 中将类型按 C ABI 布局的派生属性。
- **`GpuSolverBackend` trait**：面向 GPU 求解后端的 trait，由 `gvpe-runtime` 持有 trait object 间接依赖；具体后端（Vulkan compute / D3D12 compute）由实现 crate 提供。
- **`GpuBufferHandle`**：后端不透明句柄，表示已上传的 GPU 缓冲。
- **`DeterminismMode`**：`RuntimeDescriptor` 中决定求解器路径确定性的枚举；本文档新增其在 GPU 调度上的硬性规则。
- **`solve_island`**：`17_detailed_design.md` §6 中按物理岛迭代求解的函数，本文将其作为 CPU/GPU 路径对齐的语义锚点。

## 5. 模块详细设计

`gvpe-gpu` crate 的公开 API 仅由 `GpuSolverBackend` trait 及其依赖的类型构成；`gvpe-runtime` 通过 `Box<dyn GpuSolverBackend>` 持有后端，启用 Cargo feature `gpu-solver` 时装配具体实现。

`GpuSolverBackend` 提供的最小操作集：

| 方法 | 输入 | 输出 | 语义 |
|---|---|---|---|
| `upload_bodies` | `&BodyStateSoA` | `GpuBufferHandle` | 将 CPU 端 SoA 上传为 GPU 缓冲 |
| `upload_constraints` | `&[ConstraintRow]` | `GpuBufferHandle` | 上传约束行（按行顺序保持） |
| `dispatch_solve` | bodies 句柄、rows 句柄、`iterations: u32` | — | 对一物理岛启动一次求解 dispatch |
| `download_bodies` | 句柄、`&mut BodyStateSoA` | — | 下载回 CPU 端 SoA |

`upload_bodies` / `download_bodies` 的 `bool` 字段（`sleeping`）按 lane pack/unpack 为 `u32`（0/1）——这是唯一需要的布局变换。

## 6. 类与数据结构

### 6.1 `BodyStateSoA`（CPU 现状，与 `17_detailed_design.md` §4.1 一致）

```rust
struct BodyStateSoA {
    position: Vec<[f32; 3]>,
    orientation: Vec<[f32; 4]>,
    linear_velocity: Vec<[f32; 3]>,
    angular_velocity: Vec<[f32; 3]>,
    inv_mass: Vec<f32>,
    inv_inertia: Vec<[f32; 9]>,
    sleeping: Vec<bool>,
}
```

### 6.2 `GpuSolverBackend` trait

```rust
trait GpuSolverBackend {
    fn upload_bodies(&mut self, states: &BodyStateSoA) -> GpuBufferHandle;
    fn upload_constraints(&mut self, rows: &[ConstraintRow]) -> GpuBufferHandle;
    fn dispatch_solve(
        &mut self,
        bodies: GpuBufferHandle,
        rows: GpuBufferHandle,
        iterations: u32,
    );
    fn download_bodies(&mut self, handle: GpuBufferHandle, out: &mut BodyStateSoA);
}
```

## 7. 算法详解

### 7.1 "不妨碍 GPU 接入" 的具体含义

`04号文書` §4.6 仅陈述了约束并未证明其成立。本节给出证明：

- `BodyStateSoA` 全部字段已是 POD 元素（`#[repr(C)]`-eligible）的 flat `Vec`。
- 审查问题：「是否有任一字段需要指针追踪或堆嵌套数据？」——否；`bool` 是唯一非 GPU 原生元素（按 lane pack 为 `u32`，见 §25.3）。
- 这即是 `04号文書` §4.6 承诺却未执行的**具体审计**。

### 7.2 `gvpe-gpu` crate 边界（预留而非实现）

`gvpe-runtime` 通过 Cargo feature `gpu-solver`（默认关闭——沿用 `23_energy_wave_field_process_algorithms.md` §23.5 的 `GVPE-PROHIBIT-06` 纪律）以 trait object 形式依赖 `GpuSolverBackend`，从不直接依赖任何具体 Vulkan / D3D 类型——这保证了 `04_architecture.md` §4.3 的单向依赖规则不被破坏：`gvpe-runtime` 不新增图形 API 依赖，只多了一个可留空实现的 trait。

### 7.3 后端选择（Vulkan compute / D3D12 compute —— `00_vision.md` 声明的兼容性）

`00_vision.md` 的跨引擎要求已就**主机集成面**（`10_ffi_design.md`）承诺 Vulkan 与 D3D 兼容；`gvpe-gpu` 复用同一承诺处理计算，而非引入第三套 API：

- `wgpu`（或手写的、跨 Vulkan/D3D12 compute 的薄抽象层——实现时决定，不在本文档；同 `16_dependency_license.md` 适用于图谱/向量后端的"先有代码再选库"纪律）提供**一份**可在 Vulkan 与 D3D12 上同时运行的 compute shader 源码（WGSL 或交叉编译），与 sequential impulse 的 `solve_island` 循环（`17号文書` §6）逐行对应，使 CPU / GPU 路径可交叉校验（见 §7.5）。
- `bool` 字段（`sleeping`）在 `upload_bodies` 时打包为 `u32`（0/1），在 `download_bodies` 时解包——这是 §7.1 审计所需的唯一布局变换。

### 7.4 为什么以物理岛（Island）而非整个世界作为 GPU 调度单元

`09_parallel_design.md` 的物理岛（`17号文書` §7 的 Union-Find `build_islands`）已是 CPU 并行单元——`dispatch_solve` 接收一物理岛的 `bodies` / `rows` 切片，而非整个 `BodyStateSoA`，原因相同：物理岛相互独立，故 GPU 调度粒度复用了 CPU 已被证明为真实的边界，不为 GPU 凭空发明新分区方案。

### 7.5 确定性影响（扩展 `05_runtime_design.md` §5.3）

GPU 浮点归约顺序与 CPU 不保证位一致，跨 GPU 厂商也不保证——`RuntimeDescriptor.determinism_mode`（`17号文書` §1.3）由此新增一条显式规则：

- **`Deterministic` 模式必须无条件运行于 `GpuSolverBackend::none()`（CPU 路径）**；
- 仅 **`Fast` 模式**可选用 GPU 后端。

这是对 §5.3 既有模式表的**一行新增**，不是新机制——之所以在本文档明文写定，是因为 `04号文書` §4.6 未涉及该问题；一个声称 GPU 兼容却回避确定性的物理引擎将是不完整的。

## 8. 错误处理

- `GpuSolverBackend` trait 中各方法返回 `GpuBufferHandle`（成功）或具体后端定义的错误；MVP 不交付具体实现，错误类型由实现 crate 定义。
- `dispatch_solve` 的内部错误（设备丢失、dispatch 失败）由后端 `Result` 上抛，由 `gvpe-runtime` 捕获并按 `RuntimeError` 统一转换。
- `download_bodies` 时若 `out` 容量与上传时不一致，行为由后端实现文档化（建议在 trait 文档中明确"调用方负责保持一致"）。

## 9. 性能考量

- **零热路径成本**：feature `gpu-solver` 默认关闭，编译期即不引入；启用时 `GpuSolverBackend::none()` 实现（即 CPU 路径）应为零开销抽象。
- **数据布局零成本**：SoA 各字段独立 `Vec`，`upload_bodies` 可直接以 `bytemuck::cast_slice` 风格重解释；唯一需变换的 `bool → u32` pack/unpack 摊销到单次上传/下载。
- **物理岛粒度**：以岛为单位 dispatch 复用了 CPU 已有的并行边界，避免 GPU 端重新分区，节省调度开销。
- **非目标**：本设计不评估具体 GPU 性能数字——MVP 阶段无具体实现可衡量。

## 10. 测试考量

- **类型系统存在性测试**：`GpuSolverBackend` trait 可被空实现（`impl GpuSolverBackend for NoopBackend`）并在 `--features gpu-solver` 下编译通过。
- **数据布局断言测试**：编译期断言 `BodyStateSoA` 全部字段满足 `bytemuck::Pod`（或等价等价性），`bool` 字段单独验证 pack/unpack 往返一致。
- **交叉校验测试**（未来 GPU 实现后）：对同一 `BodyStateSoA` + `ConstraintRow` 集，分别在 CPU 与 GPU 路径上运行 `solve_island` 一次迭代，断言两路结果在 `Fast` 模式下的浮点容差内一致；`Deterministic` 模式断言 `GpuSolverBackend::none()` 被选中。
- **物理岛调度粒度测试**：mock 一个多岛世界，断言 `dispatch_solve` 被调用的次数 = 物理岛数。
- **特征门控集成测试**：`--no-default-features` 构建不应引用任何 `gvpe-gpu` 符号。

## 11. 关联需求

- **`04_architecture.md` §4.6**：约束现已**演示**而非仅**断言**。
- **`00_vision.md` §0.5**：完备性 + Vulkan / D3D 兼容性承诺。
- **`05_runtime_design.md` §5.3**：DeterminismMode 规则扩展，新增 GPU 调度一行。
- **GVPE-PROHIBIT-06**：默认不启用、关闭时零成本。

## 12. 关联文档

- 上游：`docs/01_architecture/04_architecture.md` §4.3（单向依赖规则）、§4.6（GPU 兼容性约束）、`docs/02_modules/17_detailed_design.md` §4.1（`BodyStateSoA`）、§6（`solve_island`）、§7（Union-Find `build_islands`）、`docs/00_vision/00_vision.md` §0.3（跨引擎兼容性）、`docs/02_modules/10_ffi_design.md`（主机集成面）、`docs/02_modules/16_dependency_license.md`（库选型纪律）。
- 平级：`docs/02_modules/22_vector_detailed_design.md`、`docs/02_modules/23_energy_wave_field_process_algorithms.md`（同属"非 MVP crate、接口预留"类）。
- 下游：`docs/02_modules/05_runtime_design.md` §5.3（DeterminismMode 表）。

## 13. 审批签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 | | | |
| 校对 | | | |
| 审批 | | | |

---

## 14. 正文

> 以下为原文档正文，章节编号保留（§25.1, §25.2, §25.3, §25.4, §25.5, §25.6），叙事已并入上方对应章节，本节保留原始技术片段与原文档的引用脉络。

### 25.1 "不妨碍 GPU 接入" 的具体含义

参见上方 §7.1。`BodyStateSoA` 结构定义完整保留于 §6.1。

### 25.2 `gvpe-gpu` crate 边界（预留而非实现）

参见上方 §7.2。`GpuSolverBackend` trait 完整定义保留于 §6.2。

### 25.3 后端选择（Vulkan compute / D3D12 compute —— `00_vision.md` 声明的兼容性）

参见上方 §7.3。

### 25.4 为什么以物理岛（Island）而非整个世界作为 GPU 调度单元

参见上方 §7.4。

### 25.5 确定性影响（扩展 `05_runtime_design.md` §5.3）

参见上方 §7.5。关键规则：**`Deterministic` 模式必须无条件运行于 `GpuSolverBackend::none()`（CPU 路径）；仅 `Fast` 模式可选用 GPU 后端**。

### 25.6 非目标

- 本文档集不交付 compute shader 源码——`dispatch_solve` 函数体未实现（`todo!()`），与 `21_graph_compiler_detailed_design.md` / `22_vector_detailed_design.md` 的"非 MVP crate 故意停在接口深度"先例一致。
- 不设计 GPU 宽相/窄相——`04号文書` §4.6 与本文档均将"GPU"范围限定为求解阶段（`07_solver_design.md` 已识别此阶段具有最清晰的数据并行结构）；GPU 碰撞检测是独立的未来文档，需有驱动用例再启动。
