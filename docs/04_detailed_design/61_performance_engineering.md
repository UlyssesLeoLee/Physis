# 性能工程（Performance Engineering）

> **用途**：GVPE 性能工程的方法论、基线、测量、profiling、优化技术。
> **对应工作流步骤**：45 ロジック設計、42 プログラム構造設計 → `28_workflow.md` §10.4 步 42/45。
> **关联**：`GVPE-DOC-14`（性能预算）；`GVPE-DOC-17` §3-§8（详细算法）；`GVPE-DOC-26` §18.5（SIMD）；`32_system_test_spec_template.md`（ST 性能测试）；`58_data_layout_atlas.md`（数据布局）。

## 0. 元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-61 |
| 文档类型 | 詳細設計書 |
| 版本 | v0.1 |
| 状态 | Draft |
| 适用阶段 | MVP / 优化期 |
| 上游文档 | `GVPE-DOC-14`, `GVPE-DOC-17`, `GVPE-DOC-26` §18.5 |
| 下游文档 | 实施期 / `32_system_test_spec_template.md` |

## 1. 性能目标

### 1.1 MVP 目标（`14_performance_budget.md`）

| 场景 | 目标（中端 PC） | 目标（高端 PC） | 备注 |
|---|---|---|---|
| 100 body | 60 Hz | 240 Hz | 最小场景 |
| 500 body | 60 Hz | 120 Hz | 中等场景 |
| 1000 body | 30 Hz | 60 Hz | 大场景 |

### 1.2 性能预算分配

| 阶段 | 1000 body 预算 | 备注 |
|---|---|---|
| Broad phase | 1 ms | SAP |
| Narrow phase | 1 ms | SAT |
| Constraint build | 0.5 ms | |
| Solve | 8 ms | SI，10 iter |
| Integrate | 0.5 ms | |
| Island + sleep | 0.5 ms | |
| Overhead | 0.5 ms | 调度、锁 |
| **单 step 总计** | **12 ms** | < 16 ms（60Hz） |

## 2. 测量方法

### 2.1 Criterion bench

```rust
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_si_step(c: &mut Criterion) {
    let mut group = c.benchmark_group("si_step");
    for &n in &[100, 500, 1000, 5000] {
        let bodies = setup(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| solve_step(&bodies, 0.016));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_si_step, bench_broad_phase, bench_narrow_phase);
criterion_main!(benches);
```

### 2.2 基线管理

```bash
# 首次：保存基线
cargo bench --bench <name> -- --save-baseline v0.1.0

# 后续：对比
cargo bench --bench <name> -- --baseline v0.1.0
```

Criterion 自动报告性能变化（绿色 / 黄色 / 红色）。

### 2.3 集成方机器测量

- bench 数字**不**等同于集成方实际性能（环境差异大）；
- 集成方性能以 `35_uat_spec_template.md` §4 实测为准；
- bench 数字用于**回归检测**，不用于 SLA 承诺。

## 3. Profiling 工具链

### 3.1 工具

| 工具 | 用途 | 平台 |
|---|---|---|
| `cargo flamegraph` | 火焰图（基于 `perf`） | Linux |
| `perf` | 硬件性能计数器 | Linux |
| `Instruments` (Xcode) | 时间分析 | macOS |
| `Windows Performance Analyzer` | ETW 跟踪 | Windows |
| `cargo-asm` | 看生成的汇编 | 跨平台 |
| `cargo-bloat` | 看二进制大小 | 跨平台 |
| `cachegrind` / `callgrind` | cache 模拟 | Linux（Valgrind） |
| `heaptrack` | 堆分配追踪 | Linux |

### 3.2 Profiling 流程

1. **识别热点**：criterion + flamegraph 找耗时最大的函数；
2. **root cause**：cache miss？分支预测失败？分配？锁竞争？
3. **优化**：
   - 数据布局（SoA）→ 改善 cache；
   - SIMD → 改善 throughput；
   - 算法改进 → 改善复杂度；
4. **验证**：跑 bench 前后对比 + criterion 显著性检验；
5. **记录**：commit message 写明优化点 + 数字。

## 4. 优化技术

### 4.1 算法优化

| 优化 | 适用 | 风险 |
|---|---|---|
| SAP 用 insertion sort 利用前帧顺序 | Broad phase | 乱序场景退化 |
| 接触点上限（典型 ≤ 4） | Narrow phase | 极端 case 漏检 |
| Solver warm-start | SI | 累积误差 |
| Island 拆分 | Parallel | 跨岛同步开销 |
| Sleep 跳过 | Step | 唤醒延迟 |

### 4.2 数据布局优化

详见 `58_data_layout_atlas.md`：
- SoA 替代 AoS（热数据）；
- `#[repr(C, align(N))]` 对齐；
- 紧凑字段（无 padding）；
- `SmallVec` 避免堆分配（典型 case）。

### 4.3 SIMD 优化

详见 `26_tech_selection.md` §18.5：
- vendor intrinsics（首选）：x86_64 SSE / AVX2 / AVX-512 + aarch64 NEON；
- `core::simd`（评估中）；
- 标量回退必备；
- SIMD 路径必须有 unit test（与标量输出对比）。

### 4.4 内存优化

- **零分配**热路径（`R-NFR-002`）；
- arena / pool / slab 预分配（`08_memory_design.md`）；
- `bytemuck::Pod` 安全转换；
- **不**使用 `Box` / `Rc` / `Arc` 在热路径。

### 4.5 并行优化

- Island 并行（`09_parallel_design.md`）；
- work-stealing 负载均衡（`59_algorithm_pseudocode_atlas.md` §8）；
- 锁粒度最小化（`57_coding_standards.md` §6）；
- 避免 false sharing（`#[repr(align(64))]` 关键字段）。

### 4.6 编译优化

```toml
# Cargo.toml
[profile.release]
opt-level = 3            # 最大优化
lto = "thin"             # thin LTO（build time / perf 平衡）
codegen-units = 1        # 更好优化（牺牲 build time）
panic = "abort"          # FFI 边界（仅 gvpe-ffi）
strip = true             # 减小二进制

[profile.bench]
inherits = "release"
debug = true             # 保留调试符号（profiling 用）
```

## 5. 性能陷阱（Anti-Patterns）

### 5.1 热路径常见错误

| 错误 | 影响 | 解决 |
|---|---|---|
| `String` / `Vec` 分配 | GC-like pause | 预分配 / 复用 |
| `HashMap` / `BTreeMap` 查找 | 慢（常数大） | 数组 + 线性 / 索引 |
| `format!` | 堆分配 | 避免格式化（直接写） |
| `println!` / `eprintln!` | 锁 + IO | 改 `tracing` |
| `Box<dyn Trait>` | 虚函数调用 + 分配 | enum dispatch / generic |
| `Arc<Mutex<T>>` | 锁竞争 | 局部变量 / atomic |
| `Vec::push`（在循环） | 反复分配 | `Vec::with_capacity` |
| `.clone()` | 大量内存复制 | 借用 / `Cow` |
| 浮点除法 | 比乘法慢 5-10x | 预计算倒数 |

### 5.2 微观优化

- 整数除法 → 移位 / 模运算；
- 浮点开方 → 倒数平方根（精度允许时）；
- 分支预测友好（likely / unlikely hint）；
- 循环展开（编译器自动 / `#[inline]` 配合）；
- SIMD 化热点（依 §4.3）。

## 6. 性能回归检测

### 6.1 测量时机

- **PR 触发**：`cargo bench` smoke 跑（精简 bench）；
- **main 分支**：完整 bench + baseline 对比；
- **Release 前**：完整 bench + 多平台对比（依 `39_release_checklist.md` §1.3）。

### 6.2 阈值

- PR：单 bench 变化 < 10% 不阻断；> 10% 需 review 解释；
- main：单 bench 变化 < 5% 不阻断；5-10% warning；> 10% 阻断（除非 review 特批）；
- Release：所有 bench < 5% 变化（除已知优化项）。

### 6.3 回归处理

1. 自动报警（CI 阻断）；
2. 复现：本地 `cargo bench`；
3. root cause：git bisect 或逐 commit 测；
4. 修复或回滚；
5. 复测通过。

## 7. 与具体算法的性能关联

| 算法 | 性能瓶颈 | 优化方向 |
|---|---|---|
| Broad phase SAP | 退化为 O(n²) | insertion sort 利用前帧顺序 + 退化检测 |
| Narrow phase SAT | 多 shape pair | shape-specific 快速路径（sphere-sphere O(1)） |
| SI solver | 迭代次数 × 约束数 | warm-start + 接触点上限 + solver_iterations 调优 |
| Island | Union-Find | path compression + rank（已实现） |
| Integrate | body 数 | SoA + SIMD 化 |
| 调度 | 锁竞争 / false sharing | `#[repr(align(64))]` + lock-free queue |

## 8. 性能预算管理

### 8.1 预算分配

每个模块的"性能预算"由 `14_performance_budget.md` 给出。模块负责人需：
- 在新代码 PR 中给出预期性能数字（与预算对比）；
- 若超预算，需特别解释 + 架构师 review。

### 8.2 预算例外

- 性能预算可随 release 调整（依集成方反馈 + 实际测量）；
- 调整需走 `42_change_request_form.md`；
- 重大预算调整需架构师 + 项目负责人签字。

## 9. 与 14 / 15 / 17 / 26 的关系

| 文档 | 关系 |
|---|---|
| `14_performance_budget` | **目标**（性能数字 + 场景） |
| `15_testing_strategy` | **测试方法**（criterion + ST） |
| `17_detailed_design` | **实现**（算法 + 数据结构） |
| `26_tech_selection` | **工具选型**（criterion / proptest / miri） |
| `61_performance_engineering`（本文件） | **方法论**（怎么测量 + 怎么优化） |

## 10. 关联

- `GVPE-DOC-14`（性能预算）
- `GVPE-DOC-15`（测试策略）
- `GVPE-DOC-17`（详细设计）
- `GVPE-DOC-26` §18.5（SIMD 选型）
- `GVPE-DOC-25`（GPU 后端）
- `58_data_layout_atlas.md`（数据布局）
- `32_system_test_spec_template.md`（ST 性能测试）
- `28_workflow.md` §10.4 步 42/45

## 11. 审批

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 | — | | |
| 评审 | | | |
| 批准 | | | |
