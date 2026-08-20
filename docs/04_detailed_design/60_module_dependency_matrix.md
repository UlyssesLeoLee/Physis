# 模块依赖矩阵（Module Dependency Matrix）

> **用途**：17 个 crate 的完整依赖图、各 crate 职责、允许/禁止的依赖关系、编译期不变式。
> **对应工作流步骤**：42 プログラム構造設計、43 モジュール設計 → `28_workflow.md` §10.4 步 42/43。
> **关联**：`GVPE-DOC-04` §4.1-§4.3（crate map + 依赖方向）；`GVPE-DOC-26` §18.6（crate 拓扑）；`28_workflow.md` §10.4 步 42/43。

## 0. 元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-60 |
| 文档类型 | 詳細設計書 |
| 版本 | v0.1 |
| 状态 | Draft |
| 适用阶段 | MVP / 实施期 |
| 上游文档 | `GVPE-DOC-04` §4.1-§4.3 |
| 下游文档 | 实施期 / CI `cargo tree` 验证（`AC-02`） |

## 1. 总体原则

### 1.1 三大空间映射

| 空间 | crates | 数据面 |
|---|---|---|
| **仿真空间**（Data Plane） | `gvpe-math`, `gvpe-core`, `gvpe-memory`, `gvpe-shape`, `gvpe-collision`, `gvpe-dynamics`, `gvpe-constraint`, `gvpe-solver`, `gvpe-island`, `gvpe-scheduler`, `gvpe-runtime`, `gvpe-ffi` | 实时热路径 |
| **向量空间** | `gvpe-vector` | 1-30Hz / 事件触发 |
| **图谱空间** | `gvpe-graph`, `gvpe-compiler`, `gvpe-inference`, `gvpe-3dgs` | 离线 / 低频 |

### 1.2 依赖方向（机械可验证，`AC-02`）

```text
        gvpe-graph / gvpe-vector / gvpe-inference / gvpe-3dgs
                              │
                              ▼
                        gvpe-compiler
                              │
                              ▼
   gvpe-math ← gvpe-core ← gvpe-memory ← gvpe-shape ← gvpe-collision ← gvpe-dynamics
        ← gvpe-constraint ← gvpe-solver ← gvpe-island ← gvpe-scheduler ← gvpe-runtime ← gvpe-ffi
```

- 箭头表示"被依赖"；
- **无**箭头可向上指（编译期 + CI 阻断）；
- 唯一例外：`gvpe-compiler` 可同时依赖 Graph/Vector 和 Runtime 侧（用于 `PhysicsProfile` POD 类型）。

## 2. 完整 crate 依赖矩阵

| From ↓ / To → | math | core | memory | shape | collision | dynamics | constraint | solver | island | scheduler | runtime | ffi | vector | graph | compiler | inference | 3dgs |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| **math** | — | | | | | | | | | | | | | | | | |
| **core** | ✓ | — | | | | | | | | | | | | | | | |
| **memory** | | ✓ | — | | | | | | | | | | | | | | |
| **shape** | ✓ | | | — | | | | | | | | | | | | | |
| **collision** | | | ✓ | ✓ | — | | | | | | | | | | | | |
| **dynamics** | | | | | ✓ | — | | | | | | | | | | | |
| **constraint** | | | | | | ✓ | — | | | | | | | | | | |
| **solver** | | | | | | | ✓ | — | | | | | | | | | |
| **island** | | | | | | | | ✓ | — | | | | | | | | |
| **scheduler** | | | | | | | | | | — | | | | | | | |
| **runtime** | | | | | | | | ✓ | ✓ | ✓ | — | | | | | | |
| **ffi** | | | | | | | | | | | ✓ | — | | | | | |
| **vector** | | | | | | | | | | | | | — | | | | |
| **graph** | | | | | | | | | | | | | | — | | | |
| **compiler** | | ✓ | | | | | | | | | | | (opt) | (opt) | — | | |
| **inference** | | | | | | | | | | | | | ✓ | ✓ | | — | |
| **3dgs** | | | | | | | | | | | | | | | | | — |

- ✓ = 直接依赖（依 `Cargo.toml`）；
- (opt) = 可选依赖（feature-gated）；
- 空 = 无直接依赖（可能间接依赖，须 `cargo tree` 验证）。

## 3. 各 crate 职责

| Crate | 职责 | 关键类型 | 不允许 |
|---|---|---|---|
| `gvpe-math` | 向量 / 四元数 / 矩阵 / 几何 | `Vec3`, `Quat`, `Mat3`, `Transform`, `Aabb` | 任何其他 GVPE crate |
| `gvpe-core` | 句柄 / `PhysicsProfile` / `RuntimeDescriptor` / 错误类型 | `BodyHandle`, `PhysicsProfile`, `RuntimeDescriptor`, `RuntimeError` | gvpe-graph / gvpe-vector / gvpe-compiler / gvpe-inference / gvpe-3dgs |
| `gvpe-memory` | arena / pool / slab 分配器 | `Arena`, `Pool<T>`, `Slab<T>` | gvpe-graph / gvpe-vector / ... |
| `gvpe-shape` | 形状描述 | `ShapeDesc` (enum), `Sphere`, `Box3`, `Plane`, `Capsule`, `ConvexHull`, `TriangleMesh`, `Heightfield`, `Compound` | ... |
| `gvpe-collision` | broad + narrow phase | `broad_phase_sap`, `narrow_phase_sat`, `ContactManifold` | ... |
| `gvpe-dynamics` | rigid body 状态 / 积分 | `BodyStateSoA`, `integrate_*` | ... |
| `gvpe-constraint` | ConstraintRow + 关节分解 | `ConstraintRow`, `JointRow` | ... |
| `gvpe-solver` | SI / PGS / XPBD 求解 | `Solver`, `solve_si`, `solve_xpbd` | ... |
| `gvpe-island` | Union-Find + island 拆分 | `IslandUF`, `Island` | ... |
| `gvpe-scheduler` | work-stealing 任务调度 | `Scheduler`, `JobDag`, `JobNode` | ... |
| `gvpe-runtime` | 帧循环 / 生命周期 | `Runtime`, `step`, `destroy` | ... |
| `gvpe-ffi` | C ABI 表面 | `extern "C"` 函数 / `#[repr(C)]` 类型 | ... |
| `gvpe-vector` | Physics Signature 多向量空间 | `VectorIndex`, `PhysicsSignature` | gvpe-solver 等实时 crate（`VEC-001`） |
| `gvpe-graph` | 物理知识图谱存储 | `GraphStore`, `NodeKind` (closed enum) | 实时 crate（`GPH-003`） |
| `gvpe-compiler` | Graph/Vector → `PhysicsProfile` 编译 | `PhysicsCompiler` trait, `compile()` | （唯一可跨空间依赖的 crate） |
| `gvpe-inference` | 假设生成 + 参数优化 | （post-MVP，仅接口） | ... |
| `gvpe-3dgs` | 3DGS observation 摄入 | （post-MVP，仅接口） | ... |

## 4. 禁止依赖（编译期 + CI 阻断）

### 4.1 核心 crate 禁止依赖（`NFR-003` / `AC-02`）

`gvpe-core`, `gvpe-collision`, `gvpe-dynamics`, `gvpe-constraint`, `gvpe-solver`, `gvpe-island`, `gvpe-scheduler`, `gvpe-runtime` **不**允许依赖：

```text
gvpe-graph, gvpe-vector, gvpe-compiler, gvpe-inference, gvpe-3dgs
```

### 4.2 仿真 crate 禁止依赖

任何仿真空间 crate（§3 表中前 12 行）**不**允许依赖：

```text
gvpe-vector, gvpe-graph, gvpe-compiler, gvpe-inference, gvpe-3dgs
```

（理由：仿真空间必须是 hot-path self-contained，不允许任何 inference 引用）

### 4.3 异步运行时禁止（`26_tech_selection.md` §18.6.3）

任何 crate **不**允许依赖：

```text
tokio, async-std, smol, embassy
```

### 4.4 物理引擎核心禁止（`PROHIBIT-01` / `02`）

任何 crate **不**允许依赖：

```text
rapier, rapier3d, rapier2d, bullet, bullet-rs, physx, physx-rs, jolt, jolt-rs, box2d, nphysics, parry
```

### 4.5 ML 推理禁止（`PROHIBIT-05`）

任何 crate **不**允许依赖：

```text
tch, candle-core, candle-nn, burn, ort, tract
```

### 4.6 数学核心（`PROHIBIT-01` 推论）

仿真空间 crate **不**允许依赖：

```text
glam, nalgebra, cgmath, ultraviolet
```

（`gvpe-math` 自研；详见 `26_tech_selection.md` §18.5.2）

### 4.7 `parking_lot` / `crossbeam` 核心禁止

仿真空间 crate **不**允许依赖：

```text
parking_lot, crossbeam-channel, crossbeam-queue
```

（详见 `26_tech_selection.md` §18.6.2）

## 5. 允许但有条件（feature-gated）

| 依赖 | 允许于 | 条件 |
|---|---|---|
| `tracing` | 所有 crate | 必须 `tracing-event` / `tracing-perf` feature |
| `bytemuck` | 仿真空间 crate | 仅用于 `#[derive(Pod)]` 派生 |
| `thiserror` | 所有 crate | 必用（替代 `anyhow`） |
| `criterion`（dev-dep） | 所有 crate | 仅 `dev-dependencies` |
| `proptest`（dev-dep） | 所有 crate | 仅 `dev-dependencies` |
| `cbindgen`（build-dep） | `gvpe-ffi` | 仅 `build-dependencies` |

## 6. 允许的间接依赖

通过 `gvpe-core` 间接获得（不需在 `Cargo.toml` 显式声明）：

| 间接依赖 | 来源 | 范围 |
|---|---|---|
| `bytemuck` | 仿真空间 crate 用于 POD 派生 | 显式声明 |
| `tracing` | feature-gated | 显式声明 |
| `thiserror` | 所有 crate | 显式声明 |

## 7. 编译期不变式（Rust 类型系统强制）

```rust
// gvpe-runtime 不能直接导入 gvpe-graph 类型
// 通过 PhantomData / 私有字段表达"无图依赖"
pub struct Runtime {
    _phantom: PhantomData<()>,  // 防止自动 derive Send / Sync 时引入图类型
    // ...
}
```

```rust
// gvpe-ffi 仅依赖 gvpe-runtime（不直接依赖 gvpe-core 之外的内部）
// gvpe-ffi/Cargo.toml:
// gvpe-ffi = ["dep:gvpe-runtime", "dep:gvpe-core"]  // 允许
// gvpe-ffi = ["dep:gvpe-solver"]  // 禁止（应通过 gvpe-runtime 间接）
```

## 8. CI 验证

### 8.1 `cargo tree` 阻断（`AC-02`）

```bash
cargo tree -p gvpe-core -p gvpe-collision -p gvpe-dynamics \
  -p gvpe-constraint -p gvpe-solver -p gvpe-island \
  -p gvpe-scheduler -p gvpe-runtime \
  | grep -E 'gvpe-(graph|vector|compiler|inference|3dgs)' \
  && { echo "AC-02 violation"; exit 1; } || echo "AC-02 OK"
```

CI 必跑（依 `28_workflow.md` §11.2 + `39_release_checklist.md` §1.5）。

### 8.2 `cargo deny` 阻断

```toml
# deny.toml
[bans]
multiple-versions = "deny"
deny = [
    # 物理引擎（PROHIBIT-01）
    { crate = "rapier*" },
    { crate = "bullet*" },
    # ... 完整列表见 26_tech_selection.md §18.13
]

[advisories]
yanked = "deny"
unmaintained = "warn"
```

### 8.3 私有 API 检查

```rust
// 每个 crate 入口：明确公共 API 表面
#![doc(hidden)]  // 隐藏所有未文档化项
// 或在 CI 跑：
// cargo public-api -- deny-list-file=docs/api_deny_list.txt
```

## 9. 变更管理

- 任何 crate 拓扑变化 → 走 `42_change_request_form.md`；
- 任何新增依赖 → 走 `26_tech_selection.md` §18.15 流程；
- 重大依赖变化（新增一级依赖）→ 架构师 + 许可证负责人双签。

## 10. 关联

- `GVPE-DOC-04` §4.1-§4.3（crate map + 依赖方向）
- `GVPE-DOC-26` §18.6（crate 拓扑 + 拒绝清单）
- `55_error_code_catalog.md`（错误类型布局）
- `28_workflow.md` §10.4 步 42/43

## 11. 审批

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 | — | | |
| 评审 | | | |
| 批准 | | | |
