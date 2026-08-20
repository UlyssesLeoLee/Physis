# 系统测试规格书模板（System Test Spec Template）

> **用途**：MVP 启动后，由 Release Manager 撰写的端到端系统测试规格书。
> **对应工作流步骤**：`28_workflow.md` §10.8 步 76-89（システム試験計画 / 仕様書 / 機能 / シナリオ / 性能 / 負荷 / ストレス / セキュリティ / 障害 / 完了承認）。
> **关联**：`GVPE-DOC-14`（性能预算）、`GVPE-DOC-15`（测试策略）、`GVPE-DOC-16`（许可证）、`GVPE-DOC-26`（技术选型）。

## 0. 元数据

| 字段 | 取值 |
|---|---|
| 测试目标版本 | `<vX.Y.Z-rc.N>` |
| 测试范围 | MVP 全量 / 增量（incremental） |
| 编写者 | — |
| 关联设计文档 | `GVPE-DOC-NN` |
| 关联需求 ID | `GVPE-FR-XXX`, `GVPE-NFR-XXX`, `GVPE-AC-XX` |

## 1. 测试环境

| 维度 | 取值 |
|---|---|
| 平台矩阵 | Win11 x64 / Ubuntu 22.04 x64 / macOS 14 arm64 |
| 集成引擎 | 1+ 集成方项目 |
| 测试工具 | `cargo test`, criterion, `cargo +nightly miri test`, `cargo deny`, `cargo audit` |
| 性能测量 | criterion + 集成方 telemetry |

## 2. 功能测试（78 步）

> 端到端功能场景——构建场景、运行 step、检查输出。

| ID | 场景 | 关联 FR / AC | 通过标准 |
|---|---|---|---|
| ST-FN-01 | 创建空 scene，step 1000 步 | `R-FR-001` | 无 panic，输出空 |
| ST-FN-02 | 单 sphere 重力下落 | `R-FR-002` | 落到地面静止 |
| ST-FN-03 | 1000 sphere 堆叠 | `R-FR-002`, `R-PERF-001` | 稳定，60Hz 中端 PC |
| ST-FN-04 | 关节驱动 2 段摆 | `R-FR-002` | 摆动符合物理预期 |
| ST-FN-05 | sleeping 触发 | `R-FR-002` | 静止物体进入 sleep |
| ST-FN-06 | feature-gate：关闭 Graph/Vector 后行为 | `R-FR-001`, `AC-03` | bit-for-bit identical |
| ST-FN-07 | FFI：完整生命周期 | `R-FR-008` | create → step → destroy 无泄漏 |
| ... | ... | ... | ... |

## 3. 场景测试（79 步）

> 来自 `33_typical_game_scenarios.md` 的典型游戏场景。

| ID | 场景 | 来自 | 通过标准 |
|---|---|---|---|
| ST-SC-01 | Box stack 10 | §33.2 | 不倒，无穿透 |
| ST-SC-02 | 高速 bullet 穿透薄板 | §33.5 | CCD 触发，碰撞检测正确 |
| ST-SC-03 | Rope bridge 100 段 | §33.6 | XPBD 兼容（如启用） |
| ST-SC-04 | Destruction 100 fragments | §33.7 | 性能 ≥ 30Hz |
| ... | ... | ... | ... |

## 4. 性能测试（80 步）

| ID | 场景 | 测量 | 目标 | 关联 |
|---|---|---|---|---|
| ST-PT-01 | 100 body SI step | 平均耗时 | < 4ms | `R-PERF-001` |
| ST-PT-02 | 500 body SI step | 平均耗时 | < 8ms | `R-PERF-001` |
| ST-PT-03 | 1000 body broad phase | 平均耗时 | < 1ms | `R-PERF-001` |
| ST-PT-04 | 1000 body full step | 平均耗时 | < 16ms（@ 60Hz） | `R-PERF-001` |
| ST-PT-05 | memory 峰值 | max RSS | < 200MB（1000 body） | `R-NFR-002` |
| ... | ... | ... | ... | ... |

## 5. 负荷测试（81 步）

| ID | 场景 | 测量 | 通过标准 |
|---|---|---|---|
| ST-LT-01 | 持续 1 小时 1000 body 仿真 | 帧时间稳定性 | p99 / p50 < 1.5 |
| ST-LT-02 | 持续 24 小时稳定性 | 内存增长 | < 5% 漂移（无泄漏） |
| ST-LT-03 | 多线程扩展 1/2/4/8 核 | speedup | 接近线性到 4 核 |

## 6. 压力测试（82 步）

| ID | 场景 | 通过标准 |
|---|---|---|
| ST-ST-01 | 10000 body | 不 OOM，broad phase 正常剪枝 |
| ST-ST-02 | 0.001s timestep 100 步 | 数值稳定 |
| ST-ST-03 | 0.1s timestep 100 步 | 数值稳定 |
| ST-ST-04 | solver_iterations = 100 | 正确性 + 性能线性增长 |
| ST-ST-05 | NaN / Inf 输入 | 错误码，无 panic |

## 7. 安全 / 许可证测试（83 步）

| ID | 场景 | 通过标准 |
|---|---|---|
| ST-SC-01 | `cargo deny check` | 0 violation |
| ST-SC-02 | `cargo audit` | 0 known High / Critical unpatched |
| ST-SC-03 | FFI panic 跨边界 | 不 UB，错误码 |
| ST-SC-04 | `unsafe` 块 miri | 0 UB |
| ST-SC-05 | 句柄 use-after-free | 错误码，无 UB |
| ST-SC-06 | 长字符串 / 越界 | 错误码，无溢出 |
| ST-SC-07 | FFI 并发 | MVP 单 Runtime，无并发 |

## 8. 障碍测试（84 步）

| ID | 场景 | 期望 |
|---|---|---|
| ST-FT-01 | 0 质量 body | 错误码或被忽略 |
| ST-FT-02 | 负质量 | 错误码 |
| ST-FT-03 | 极大初速度 | 不 NaN |
| ST-FT-04 | 退化 box（half_extent = 0） | 正确处理 |
| ST-FT-05 | 极大重力 | 不 NaN |
| ST-FT-06 | 0 timestep | 错误码 |
| ST-FT-07 | 负 timestep | 错误码 |
| ST-FT-08 | 0 solver_iterations | 错误码或 step 0 冲量 |

## 9. 通过标准汇总

| 类别 | 必须全部通过 |
|---|---|
| 功能测试 | ✅ |
| 场景测试（核心 5+） | ✅ |
| 性能测试 | ✅（目标达成 90%+） |
| 负荷测试（1h / 24h） | ✅ |
| 压力测试 | ✅ |
| 安全测试 | ✅（0 violation） |
| 障碍测试 | ✅ |

## 10. 关联工作流步骤

- `28_workflow.md` §10.8 步 76-89
- `28_workflow.md` §11.15-§11.17
- `GVPE-DOC-14`（性能预算详细数字）
- `GVPE-DOC-15`（测试策略）

## 11. 审批

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 | — | | |
| 评审 | | | |
| 批准 | | | |
