# MVP 实验计划（H1~H5 假设验证）

## 文书管理表

| 项目 | 内容 |
|---|---|
| 文书编号 | PRE-DOC-06 |
| 版本 | v0.1 |
| 状态 | Draft |

## 改订履历

| 版本 | 日期 | 变更内容 |
|---|---|---|
| v0.1 | 2026-08-17 | 初版：H1～H5 实验设计 + 附属实验 + Benchmark |

---

目的：在推进 Phase 2（Dynamic 3DGS 集成）之前，用可控实验验证「物理响应能否形成有意义的可检索空间，并显著降低未知物理状态的求解成本」这一核心假设。

## MVP 范围回顾

- Solvers: Rigid, XPBD(cloth/soft body), MPM(elastic)
- Physics Experiences: 10K~100K（数据集生成器批量产出）
- Physical Signature: V1（确定性特征，02号文档§9）
- Physical Encoder: V1（deterministic，非神经网络）
- Retrieval: HNSW + post-filter
- Simulation Verification: 必须实现（不可省略，ADR-002）
- Parameter Refinement: basic（局部搜索）
- Dynamic 3DGS: 不实现（SimulationBackend 冒充观测）

## 实验设计总则（回应 ISS-009）

所有"观测 vs 预测"比较，观测响应必须切分为：

- **匹配窗口**（前 60%~70% 时长）：用于生成 Signature/Embedding、执行检索。
- **held-out 未来窗口**（剩余时长）：仅用于最终误差评分，不参与检索或参数猜测。

这保证 Verification 检验的是"预测未来"能力，而非"拟合已知轨迹"。

---

## H1: 相似物理响应在 Embedding Space 中距离更近

**设计**：从 Atlas 中选取若干"物理上已知相似"的 Experience 对（同 solver、参数邻近、宏观现象类别相同）与"已知不相似"的对（不同现象类别，如自由落体 vs 悬垂布料），比较 embedding 距离分布。

**指标**：相似对与不相似对的 embedding 距离分布是否有显著可分性（如 AUC、t-test）。

**通过标准**：相似对距离显著小于不相似对（p<0.05，且效应量非平凡），且该结论在 landmark 采样 vs 全场采样两种 Response 记录方式下均成立（呼应 ISS-001 消融）。

## H2: 不同 Solver 产生的相似宏观行为能够被检索到

**设计**：构造跨 solver 但宏观行为相近的场景对（例如：XPBD soft body 与 MPM elastic 在相近参数下模拟同一物体的弹性形变），检验以一方为 query 能否在 Top-K 中检索到另一方。

**指标**：跨 solver 检索命中率（Top-K 命中 vs 随机基线）。

**通过标准**：显著高于随机基线。若不成立，记录为 ISS-005 的实验证据，考虑是否需要限定检索范围为"同类现象"而非全局。

## H3: ANN 可以显著减少 inverse physics 搜索空间

**设计**：对比两条路径：
1. Brute-force：直接在参数空间做局部搜索优化，从随机初始猜测开始。
2. ANN 辅助：先检索 Top-K 得到初始猜测，再做同样的局部搜索。

**指标**：达到同等误差阈值所需的仿真次数、总耗时、收敛成功率。

**通过标准**：ANN 辅助路径显著减少仿真次数与耗时（如 >30% 减少），且收敛成功率不低于 brute-force。

## H4: Embedding Retrieval + Simulation Verification 优于单独 Embedding Retrieval

**设计**：比较"仅用 embedding 相似度排序取 Top-1"与"embedding 检索 Top-K + Verification 重排序取最终 Top-1"两种方式，在 held-out 未来窗口上的预测误差。

**指标**：held-out 窗口预测误差（position/velocity/deformation）。

**通过标准**：Verification 重排序后的 Top-1 误差显著低于纯 embedding 排序的 Top-1。

## H5: Observation 加入合理噪声以后仍具有检索能力

**设计**：对 held-out 观测响应注入不同水平的高斯噪声（模拟未来真实观测的不确定性），重复 H1/H3 的检索评估。

**指标**：检索命中率/H3 效率提升 随噪声水平变化的曲线。

**通过标准**：在合理噪声水平（需在实验中标定，如 SNR 对应真实传感器/重建误差范围的估计值）下，检索能力退化但不崩溃（仍显著优于随机基线）。

## 附属实验（回应自审发现的 ISS）

- **ISS-001 消融**：全场 Response 记录 vs Landmark 采样，对比 H1 结论是否一致。
- **ISS-006 验证**：统计 Top-M 候选参数分散度，检验"参数不可辨识"检测机制是否能正确标注已知不可辨识的场景（如构造一个人为的多解场景作为测试用例）。
- **ISS-008 检测**：记录 Dataset Generator 采样失败/发散点的参数分布，检查是否存在系统性偏差区域。

## 基准（Benchmark，对应 PRE-VEC-001 / PRE-PERF-001）

| 指标 | 目标 | 测量方式 |
|---|---|---|
| ANN Top-N(N≤100) 查询延迟 P95 | < 200ms（10K~100K规模，单机无GPU） | 微基准，重复≥1000次查询取分位数 |
| 单条 XPBD cloth Experience 生成耗时（5秒仿真） | < 30s（单核参考实现） | 计时基准 |
| Dataset Generator 并行吞吐 | 随核数近线性提升（无具体SLA，趋势验证即可） | 多核数对比实验 |

## 实验产出与判定流程

每条假设产出：结论（成立/部分成立/不成立）+ 证据（图表/数据）+ 对架构的反馈建议。若 H1 不成立，项目不得推进 Phase 2（3DGS 集成），需回到 Signature/Encoder 设计重新迭代（对应 02号文档 §31 Evolution Strategy 的回退触发条件）。
