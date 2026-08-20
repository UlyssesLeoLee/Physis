# 验收测试规格书模板（UAT Spec Template）

> **用途**：UAT 计划（`34_uat_plan_template.md`）下的具体测试规格，命名 `uat_spec_<partner>.md`。
> **对应工作流步骤**：`28_workflow.md` §10.9 步 91（受入試験仕様書作成）、§11.18-§11.21。
> **关联**：`34_uat_plan_template.md`、`33_typical_game_scenarios.md`、`31_pilot_integration_agreement.md`。

## 0. 元数据

| 字段 | 取值 |
|---|---|
| UAT 协议编号 | `GVPE-UAT-XXX` |
| 文档版本 | v0.X |
| 编写者 | — |
| 测试人员 | 集成方 QA 团队 |
| 关联 | `34_uat_plan_template.md`（计划）、`32_system_test_spec_template.md`（ST 规格） |

## 1. 测试用例格式约定

每个测试用例包含：

| 字段 | 说明 |
|---|---|
| ID | `UAT-FN-NN` / `UAT-SC-NN` / `UAT-PT-NN` 等 |
| 场景 | 简述 |
| 前置条件 | 集成环境、初始状态 |
| 输入 | 具体参数 / 数据 |
| 步骤 | 操作序列 |
| 期望 | 期望输出 / 行为 |
| 测量 | 如有（性能数字等） |
| 通过条件 | 判定标准 |
| 关联需求 | `GVPE-FR-XXX` 等 |
| 备注 | 已知问题、特殊考虑 |

## 2. 功能测试用例（UAT-FN-*）

### 2.1 UAT-FN-01：单球下落

| 字段 | 内容 |
|---|---|
| 场景 | 1 个 sphere 从 10m 高度静止释放 |
| 前置条件 | 集成环境就绪；scene 仅有 1 个 sphere + 1 个 ground plane |
| 输入 | sphere: mass=1, radius=0.5, pos=(0,10,0); ground: y=0 |
| 步骤 | 1. step 60 步 @ 60Hz |
| 期望 | sphere 落到地面；最终静止于 y=0.5 |
| 测量 | 无 |
| 通过条件 | 最终位置 y ∈ [0.45, 0.55] |
| 关联需求 | `R-FR-002` |
| 备注 | 数值积分器精度 |

### 2.2 UAT-FN-02：箱子堆叠

| 字段 | 内容 |
|---|---|
| 场景 | 10 个 box 垂直堆叠 |
| 前置条件 | 10 个 box 已放置；地面静止 |
| 输入 | 每个 box: mass=1, half_extent=(0.5,0.5,0.5) |
| 步骤 | 1. step 300 步 @ 60Hz（5 秒） |
| 期望 | 稳定；最上层 box 位置接近初始 y=10.5 ± 0.1 |
| 通过条件 | 无穿透（任何接触 penetration < 0.01）；无 NaN；sleeping 触发 |
| 关联需求 | `R-FR-002`, `QA-I-02` |

### 2.3 UAT-FN-03：feature-gate 验证（关键）

| 字段 | 内容 |
|---|---|
| 场景 | 关闭 Graph/Vector feature 后行为 |
| 前置条件 | 编译时 `--no-default-features --features simd-only` |
| 输入 | 与 UAT-FN-02 相同场景 |
| 步骤 | 1. 跑完全相同输入；比较输出 |
| 期望 | bit-for-bit identical 输出（deterministic replay） |
| 通过条件 | 输出哈希完全一致 |
| 关联需求 | `R-FR-001`, `AC-03`, `AC-01` |

### 2.4 UAT-FN-04：FFI 完整生命周期

| 字段 | 内容 |
|---|---|
| 场景 | C ABI 创建 scene → step → destroy |
| 前置条件 | cbindgen 头文件已集成 |
| 输入 | 100 个 body |
| 步骤 | 1. `gvpe_create_scene` 2. `gvpe_step` 1000 次 3. `gvpe_destroy_scene` |
| 期望 | 全部返回 `GVPE_OK`；无泄漏 |
| 通过条件 | valgrind / asan 无泄漏；无 UB |
| 关联需求 | `R-FR-008`, `QA-F-02` |

### 2.5 UAT-FN-05：FFI panic 安全性

| 字段 | 内容 |
|---|---|
| 场景 | 触发 Rust 端 panic，验证 catch |
| 前置条件 | 集成方有触发 panic 的方式（test hook） |
| 输入 | 故意传入非法 handle / 越界 buffer |
| 步骤 | 1. 调 `gvpe_set_position(invalid_handle, ...)` 2. 检查返回值 |
| 期望 | 返回 `GVPE_E_INVALID_HANDLE` 或 `GVPE_E_PANIC`，不 UB，不 crash 集成方进程 |
| 通过条件 | 集成方进程存活；后续 gvpe 调用仍正常 |
| 关联需求 | `R-FR-008`, `QA-F-02`, `QA-F-03` |

## 3. 场景测试用例（UAT-SC-*）

> 来自 `33_typical_game_scenarios.md` 的 L1 + L2 场景。

| ID | 场景 | 来源 | 通过条件 | 关联 |
|---|---|---|---|---|
| UAT-SC-01 | Box stack 10 | §33.2.2 | 见 UAT-FN-02 | `R-FR-002` |
| UAT-SC-02 | Sphere pile 500 | §33.2.3 | 3 秒稳定，60Hz | `R-PERF-001` |
| UAT-SC-03 | Jointed pendulum | §33.2.4 | 摆动符合物理 | `R-FR-002` 关节 |
| UAT-SC-04 | Sleeping validation | §33.2.5 | sleep 触发后 step 耗时下降 80% | `QA-I-03` |
| UAT-SC-05 | Tower of Pisa | §33.3.1 | 5 秒稳定 | `QA-I-02` |
| UAT-SC-06 | Bullet through wall | §33.3.2 | CCD 触发，无穿透 | `D-18`, `QA-I-04` |
| UAT-SC-07 | Newton cradle | §33.3.5 | 动量近似守恒 | 求解器正确性 |
| UAT-SC-08 | 1 万 body 压力 | §33.3.7 | 30Hz 不 OOM | `QA-P-01` |
| UAT-SC-09 | Domino chain | §33.3.6 | 100 box 链式倒伏 | 长跑稳定性 |

## 4. 性能测试用例（UAT-PT-*）

| ID | 场景 | 测量 | 目标（中端 PC） | 关联 |
|---|---|---|---|---|
| UAT-PT-01 | 100 body SI step | avg / p99 | avg < 4ms | `R-PERF-001` |
| UAT-PT-02 | 500 body SI step | avg / p99 | avg < 8ms | `R-PERF-001` |
| UAT-PT-03 | 1000 body full step | avg / p99 | avg < 16ms | `R-PERF-001` |
| UAT-PT-04 | 24h 长跑 1000 body | 内存增长 | < 5% 漂移 | 稳定性 |
| UAT-PT-05 | 多线程扩展 1/4/8 核 | speedup | 接近线性到 4 核 | `D-09` |

## 5. 障碍测试用例（UAT-FT-*）

| ID | 场景 | 输入 | 期望 | 关联 |
|---|---|---|---|---|
| UAT-FT-01 | 0 质量 | mass = 0 | 错误码 | `R-FR-002` |
| UAT-FT-02 | 负质量 | mass = -1 | 错误码 | `R-FR-002` |
| UAT-FT-03 | NaN 输入 | position = (NaN, 0, 0) | 错误码 | `QA-I-02` |
| UAT-FT-04 | 极大初速度 | v = 1e10 | 不 NaN，可能 step 内 CCD 触发 | `QA-I-04` |
| UAT-FT-05 | 退化 box | half_extent = 0 | 错误码或正常运行 | `R-FR-002` |
| UAT-FT-06 | 极大重力 | g = 1e10 | 不 NaN | `QA-I-02` |
| UAT-FT-07 | 0 timestep | dt = 0 | 错误码 | `R-FR-002` |
| UAT-FT-08 | 负 timestep | dt = -0.01 | 错误码 | `R-FR-002` |
| UAT-FT-09 | 0 solver_iterations | iter = 0 | step 正常，0 冲量 | `D-17 §1.2` |
| UAT-FT-10 | 句柄 use-after-free | free 后再用 | 错误码 | `QA-F-03` |

## 6. FFI 测试用例（UAT-FFI-*）

| ID | 场景 | 期望 |
|---|---|---|
| UAT-FFI-01 | 创建 / 销毁 1000 scene | 无泄漏 |
| UAT-FFI-02 | 并发 step | MVP 单 Runtime，文档化（不支持并发） |
| UAT-FFI-03 | 长字符串输入（> 1024 字节） | 截断或错误 |
| UAT-FFI-04 | 越界 body index | 错误码 |
| UAT-FFI-05 | 头文件 ABI 校验 | `cargo build` 后 `bindgen` 布局哈希与上一 release 一致 |

## 7. 报告格式

每个测试用例执行后产出：

```
测试 ID: UAT-XX-NN
集成方: XXX
环境: <CPU/GPU/OS/集成引擎版本/GVPE 版本>
执行时间: <YYYY-MM-DD HH:MM>
结果: ☐ 通过 ☐ 失败 ☐ 跳过
性能数字（如有）: <avg / p99 / memory>
日志: <路径>
备注: <任意>
```

汇总为 CSV / JSON，便于自动化分析。

## 8. 关联

- `28_workflow.md` §10.9 步 91、§11.18-§11.21
- `31_pilot_integration_agreement.md` §1, §3
- `33_typical_game_scenarios.md`
- `34_uat_plan_template.md`

## 9. 审批

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 | — | | |
| 评审 | | | |
| 集成方接受 | | | |
