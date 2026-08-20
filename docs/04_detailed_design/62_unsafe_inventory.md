# Unsafe 块清单（Unsafe Inventory）

> **用途**：GVPE 所有 `unsafe` 块的权威登记——位置、不变式、miri 验证、review 要求。
> **对应工作流步骤**：44 クラス設計、45 ロジック設計 → `28_workflow.md` §10.4 步 44/45。
> **关联**：`GVPE-DOC-17` §1-§2（核心类型与分配器）；`GVPE-DOC-26` §18.7（FFI）；`GVPE-DOC-57` §5（unsafe 政策）；`58_data_layout_atlas.md`（POD）；`38_code_review_checklist.md`（review）。

## 0. 元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-62 |
| 文档类型 | 詳細設計書 |
| 版本 | v0.1 |
| 状态 | Draft |
| 适用阶段 | MVP / 实施期 |
| 上游文档 | `GVPE-DOC-17`, `GVPE-DOC-26` §18.7, `GVPE-DOC-57` §5 |
| 下游文档 | 实施期 / `38_code_review_checklist.md` |

## 1. 总原则（重申）

详见 `57_coding_standards.md` §5：

- **最小化**：能用 safe Rust 表达**不**用 `unsafe`；
- **集中**：尽量将 `unsafe` 集中到 `gvpe-memory` / `gvpe-ffi`；
- **审计**：每个 `unsafe` 必须有 `// SAFETY:` 注释 + miri 验证；
- **禁项**：`transmute` / `mem::uninitialized` / 裸指针生命周期 / 绕过借用。

## 2. unsafe 块清单（按 crate 分类）

> 每个 `unsafe` 块登记。MVP 实施期如有新增，**必须**更新本文件。

### 2.1 `gvpe-memory`

#### 2.1.1 `Arena::alloc`（裸指针写入）

```rust
unsafe {
    write_and_borrow(&self.buf, offset, val)
}
```

- **位置**：`src/memory/arena.rs:alloc`；
- **不变式**：
  - `offset + size_of::<T>() <= self.buf.len()`（由调用前 assert 保证）；
  - `offset` 已对齐 `align_of::<T>()`（由 `align_up` 保证）；
  - 写入值不会 panic（要求 `T: Copy` 或手动处理 `Drop`）；
- **miri 验证**：`cargo +nightly miri test gvpe-memory` ✅；
- **Review 要求**：1 名 crate 维护者 + 1 名架构师；
- **未来重构**：评估用 `MaybeUninit::write` 替代裸指针。

#### 2.1.2 `Pool<T>::get_unchecked`

```rust
unsafe {
    &*self.slots[idx].as_ptr()
}
```

- **位置**：`src/memory/pool.rs:get_unchecked`；
- **不变式**：
  - `idx < self.slots.len()`（由调用方保证）；
  - `self.slots[idx].is_some()`（已 acquire）；
- **miri 验证**：✅；
- **Review 要求**：1 名 crate 维护者；
- **未来重构**：改用 `Option<T>` 安全访问。

#### 2.1.3 `Slab<T>::get_unchecked`（同 Pool）

类似 `Pool::get_unchecked`。

### 2.2 `gvpe-ffi`

#### 2.2.1 `extern "C"` 函数体内裸指针解引用

```rust
#[no_mangle]
pub extern "C" fn gvpe_step(rt: *mut Runtime, dt: f32) -> u32 {
    if rt.is_null() { return GVPE_E_NULL; }
    let runtime = unsafe { &*rt };
    // ...
}
```

- **位置**：所有 `extern "C"` 函数入口；
- **不变式**：
  - `rt` 非 null（`is_null()` 提前检查）；
  - `rt` 由 `gvpe_create_runtime` 返回（保证指向有效 `Runtime`）；
  - 集成方**不**在 step 期间 free `rt`（集成方文档明确）；
- **miri 验证**：✅（所有 FFI 函数）；
- **Review 要求**：架构师 + 1 名 FFI 维护者；
- **未来重构**：考虑 `NonNull<Runtime>` 包装。

#### 2.2.2 `catch_unwind` 跨边界（UB 风险）

```rust
let result = std::panic::catch_unwind(|| {
    // ... actual logic
});
```

- **位置**：所有 `extern "C"` 函数；
- **不变式**：
  - `panic::catch_unwind` 在 release build 中**不**捕获 unwind（panic 仍是 abort）—— **不**，catch_unwind 在 release 也捕获 unwind，但要求 `[profile.release] panic = "abort"` 让 catch 失败后变 abort；
  - 实际策略：`gvpe-ffi` 用 `panic = "abort"`，catch_unwind 退化为只接住某些 panic 类型；
- **miri 验证**：不适用（`catch_unwind` 与 miri 兼容性有限）；
- **Review 要求**：架构师 + FFI 维护者；
- **风险**：跨 `extern "C"` 边界的栈展开本身是 UB；推荐所有 `extern "C"` 函数都用 `catch_unwind` 包裹 + `panic = "abort"` 兜底。

#### 2.2.3 `bytemuck::Pod` 派生

```rust
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct BodyHandle { ... }
```

- **位置**：所有 `#[repr(C)]` POD 类型；
- **不变式**：`bytemuck::Pod` 派生要求类型无 padding、所有字段为 `Pod`、布局确定；
- **miri 验证**：✅（通过 `cargo +nightly miri test` 验证 `bytes_of` / `from_bytes`）；
- **Review 要求**：每次新增 `#[repr(C)]` 类型时；
- **测试**：`58_data_layout_atlas.md` §5 的 `test_layouts` 必须通过。

### 2.3 `gvpe-scheduler`

#### 2.3.1 work-stealing 队列（lock-free）

```rust
unsafe {
    // 基于 crossbeam-deque 或自研 Chase-Lev work-stealing deque
    self.local_stealers[other].steal()
}
```

- **位置**：`src/scheduler/worker.rs:find_work`；
- **不变式**：
  - Chase-Lev 算法的安全性证明（论文 + 实现匹配）；
  - `Acquire` / `Release` 内存顺序正确；
- **miri 验证**：✅（重点验证）；
- **Review 要求**：架构师 + 1 名并发专家 + 1 名 scheduler 维护者；
- **未来重构**：评估 `crossbeam-deque` 替代自研（许可证审查通过后）；
- **测试**：长时间 fuzz + miri + 多线程 unit test。

### 2.4 `gvpe-vector`

#### 2.4.1 SIMD intrinsics（`target_feature`）

```rust
#[target_feature(enable = "avx2")]
unsafe fn simd_dot_avx2(a: &[f32], b: &[f32]) -> f32 {
    // AVX2 intrinsics
}
```

- **位置**：`src/vector/simd.rs`（vendor intrinsics 实现）；
- **不变式**：
  - 调用方已确认 CPU 支持（`is_x86_feature_detected!`）；
  - 数组长度对齐 8（AVX2）；
- **miri 验证**：✅；
- **Review 要求**：架构师 + 性能负责人；
- **未来重构**：评估 `core::simd`（nightly）替代 vendor intrinsics。

### 2.5 `gvpe-runtime`

#### 2.5.1 `MaybeUninit` 用法

```rust
let mut buf: [MaybeUninit<ConstraintRow>; 1024] = unsafe {
    MaybeUninit::uninit().assume_init()
};
```

- **位置**：`src/runtime/scratch.rs`；
- **不变式**：所有 `MaybeUninit<T>` 在使用前必须 `assume_init` / 写入；
- **miri 验证**：✅；
- **Review 要求**：1 名 crate 维护者；
- **未来重构**：评估 `array_init` crate（需审查许可证）。

## 3. 总体统计

| 类别 | 数量（MVP 目标） |
|---|---|
| 总 `unsafe` 块 | < 30 |
| `gvpe-memory` | 3-5 |
| `gvpe-ffi` | 5-10 |
| `gvpe-scheduler` | 3-5 |
| `gvpe-vector` (SIMD) | 5-10 |
| `gvpe-runtime` (MaybeUninit) | 1-3 |
| 其他 | 1-3 |

> MVP 启动前应稳定在 < 30 个 `unsafe` 块；超出需架构师 review。

## 4. miri 验证矩阵

| Crate | miri 测试 | CI 必跑 |
|---|---|---|
| `gvpe-memory` | ✅ | ✅ |
| `gvpe-ffi` | ✅ | ✅ |
| `gvpe-scheduler` | ✅ | ✅ |
| `gvpe-vector` (SIMD) | ✅ | ✅ |
| `gvpe-runtime` | ✅ | ✅ |
| 其他 | （按需） | |

CI 步骤（`26_tech_selection.md` §18.12.1 / 9）：

```bash
cargo +nightly miri test -p gvpe-memory
cargo +nightly miri test -p gvpe-ffi
cargo +nightly miri test -p gvpe-scheduler
cargo +nightly miri test -p gvpe-vector --features simd
cargo +nightly miri test -p gvpe-runtime
```

## 5. review 要求矩阵

| 类别 | 强制 reviewer 数 | 强制角色 |
|---|---|---|
| `gvpe-memory` unsafe | 2 | crate 维护者 + 架构师 |
| `gvpe-ffi` unsafe | 2 | 架构师 + FFI 维护者 |
| `gvpe-scheduler` unsafe | 3 | 架构师 + 并发专家 + scheduler 维护者 |
| SIMD intrinsics | 2 | 架构师 + 性能负责人 |
| `MaybeUninit` 用法 | 1 | crate 维护者 |
| `bytemuck::Pod` 派生 | 1 | crate 维护者 |

详见 `38_code_review_checklist.md`。

## 6. 未来重构 / 优化计划

| 计划 | 优先级 | 触发条件 |
|---|---|---|
| 评估 `crossbeam-deque` 替代自研 work-stealing | Medium | 许可证审查通过 + 性能对比不劣于自研 |
| 评估 `core::simd` 替代 vendor intrinsics | Low | `core::simd` 稳定 + 性能持平 |
| 评估 `array_init` crate | Low | 许可证审查通过 |
| `NonNull<Runtime>` 包装 FFI 句柄 | Medium | 维护期重审 |

## 7. 变更管理

- 新增 `unsafe` 块：
  - 更新本文件（添加行）；
  - 走 `42_change_request_form.md`（影响 ABI / 性能时）；
  - 双 reviewer + miri 验证（`38_code_review_checklist.md` §3）；
- 删除 `unsafe` 块：
  - 更新本文件（删除行 + 注明删除原因）；
  - 必须有替换的 safe 实现 + 性能对比。

## 8. 关联

- `GVPE-DOC-17`（详细设计）
- `GVPE-DOC-26` §18.7（FFI 边界）
- `GVPE-DOC-57` §5（unsafe 政策）
- `58_data_layout_atlas.md`（POD 派生）
- `38_code_review_checklist.md`（review 要求）
- `28_workflow.md` §10.4 步 44/45

## 9. 审批

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 | — | | |
| 评审 | | | |
| 批准 | | | |
