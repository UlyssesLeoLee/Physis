# Physical Retrieval Engine（PRE）测试用例一览

## 文书管理表

| 项目 | 内容 |
|---|---|
| 文书编号 | PRE-DOC-09 |
| 版本 | v0.1.4 |
| 状态 | Draft |
| 输入基线 | 08_PRE_Detailed_Design.md（v0.1.3） |
| 说明 | 本文书是 IPA 详細設計書惯例中「テストケース一覧」的落实：为每个需要验证的行为给出用例 ID、前置条件、输入、期望输出、对应需求/设计出处，供实现阶段直接转化为自动化测试。本文书不包含 H1~H5 假设验证实验（见 06 号文档），仅覆盖单元/集成级测试用例。 |
| Mock 支持 | 部分用例适合使用 10~12 号文档定义的 `pre-testkit`（Mock 项目）加速执行、精确构造边界情形；具体哪些 testkit 组件对应哪些用例，见 12_PRE_Testkit_Detailed_Design.md §8 映射表，本文书不重复标注每一行以避免双份维护漂移。是否使用 mock 与是否需要真实层集成测试兜底，遵循 11_PRE_Testkit_Basic_Design.md §2 的双层测试策略。 |

## 改订履历

| 版本 | 日期 | 变更内容 |
|---|---|---|
| v0.1 | 2026-08-17 | 初版：TC-CORE / TC-SOLVER / TC-SIG / TC-ENC / TC-RET / TC-VER / TC-REF / TC-GEN / TC-ATLAS / TC-E2E 十类用例 |
| v0.1.1 | 2026-08-17 | 新增 TC-BEVY 用例类，对应 08号文档 §18 `pre-bevy` 详细设计 |
| v0.1.2 | 2026-08-17 | 新增"Mock 支持"说明，指向新增的 10~12 号文档（`pre-testkit` 需求/基本设计/详细设计） |
| v0.1.3 | 2026-08-17 | TC-BEVY-001 改为从 cargo metadata 动态枚举 workspace 成员，避免硬编码名单导致新增 crate 逃逸检查 |
| v0.1.4 | 2026-08-17 | 原 TC-BEVY 一类拆分重编为 TC-ENG（中立契约）/ TC-BEVY / TC-GODOT / TC-CONF（跨适配层一致性）/ TC-GPU / TC-EMB 六类，对应 08号文档 v0.1.3 的分层重写 |

---

## 用例编号规则

`TC-<模块前缀>-<三位序号>`。模块前缀对应 08 号文档章节：CORE(§2) / SOLVER(§3-6) / SIG(§7) / ENC(§8) / ATLAS(§9) / RET(§10) / VER(§11) / REF(§12) / GEN(§13) / ENG(§18) / BEVY(§19) / GODOT(§20) / CONF(§18.4 套件) / GPU(§22) / EMB(§21, 02号§35) / E2E(跨模块)。

---

## TC-CORE：核心数据结构

| ID | 前置条件 | 输入 | 期望输出 | 需求/设计出处 |
|---|---|---|---|---|
| TC-CORE-001 | 无 | 构造一个字段组全部为最小值（空 Vec/None）的 `PhysicsExperience` | 序列化/反序列化成功，字段组本身存在（非缺失），仅内部字段为空 | PRE-DATA-001, 08§2.1 |
| TC-CORE-002 | 无 | 尝试构造缺少某个顶层字段组（如省略 `provenance`）的记录 | 编译期或构造期报错（Rust 类型系统层面不允许省略顶层字段，非运行时校验） | PRE-DATA-001 |
| TC-CORE-003 | 已有一条 `StandardPhysicalResponse` | 检查其字段是否引用任何 solver 私有类型 | 静态检查（trait bound / 类型系统）通过，无 solver 特定类型出现在该结构定义中 | PRE-PHY-005, 08§2.3 |

## TC-SOLVER：Solver 插件

| ID | 前置条件 | 输入 | 期望输出 | 需求/设计出处 |
|---|---|---|---|---|
| TC-SOLVER-001 | Rigid solver 已注册 | 两球体沿相反方向匀速运动直至碰撞，弹性碰撞（restitution=1） | 碰撞后速度交换（在数值容差内），`contact_events` 记录一次接触，`restitution_estimate ≈ 1.0` | 08§4 |
| TC-SOLVER-002 | XPBD cloth solver 已注册 | 单点固定的方形布料，仅受重力 | 布料下垂稳定后 `deformation.stretch` 收敛到有限值，无发散 | 08§5 |
| TC-SOLVER-003 | XPBD solver 已注册 | 距离约束的 rest_length 设为负数（非法输入） | 返回 `SOLVER_INIT_INVALID_PARAMS`，不进入 step 循环 | 08§15 错误码, PRE-SEC-001 |
| TC-SOLVER-004 | MPM elastic solver 已注册 | 弹性方块自由落体后与地面碰撞并回弹 | `deformation.strain_rms` 在碰撞时刻出现峰值后逐渐衰减，无 NaN | 08§6 |
| TC-SOLVER-005 | 任一 solver 已初始化 | 人为注入导致数值发散的参数（如过大 timestep） | `step()` 在检测到 NaN/Inf 的那一步返回 `SOLVER_NUMERICAL_DIVERGENCE`，该 Experience 标记 Invalid 且不写入 Atlas | PRE-REL-001, 08§3.2 |
| TC-SOLVER-006 | Rigid/XPBD/MPM 三种 solver 均对同一“自由落体接触地面反弹”场景配置好参数 | 分别运行三个 solver | 三者产生的 `StandardPhysicalResponse` 均可被同一 `extract_signature()` 函数处理且不报错（不要求数值相等） | PRE-PHY-003, ADR-003 |
| TC-SOLVER-007 | Reference（单线程）与并行版本（若已实现）solver 均可用 | 相同参数运行两版本 | 两者 `position(t)` 差异在配置的数值容差内 | PRE-PHY-002 |
| TC-SOLVER-008 | XPBD solver 已初始化，几何资源携带命名锚点 | 检查生成的 landmark 集合 | landmark ID 与几何锚点名一致；若无命名锚点则退化为坐标式 ID | 08§3.3 |

## TC-SIG：特征提取

| ID | 前置条件 | 输入 | 期望输出 | 需求/设计出处 |
|---|---|---|---|---|
| TC-SIG-001 | 一条完整 `StandardPhysicalResponse`（XPBD cloth） | 调用 `extract_signature()` | 返回的 `PhysicalSignature` 八个子结构全部存在，数值字段非 NaN | PRE-FR-004, 08§7.1 |
| TC-SIG-002 | `response.deformation = None`（Rigid Body 场景） | 调用 `extract_signature()` | `deformation` 子结构字段全部为 0，函数不报错、不 panic | 08§7.2 |
| TC-SIG-003 | `sample_times` 长度小于 FFT 最小样本数下限 | 调用 `extract_signature()` | `temporal.dominant_freq = None`，不返回随机/噪声主频值 | 08§7.3 |
| TC-SIG-004 | 两条响应：材料参数邻近、宏观现象相同 vs 材料参数迥异、现象不同 | 分别提取 signature 并计算特征距离（非 embedding，是 signature 层） | 相似场景对的特征距离显著小于不相似场景对（预演 H1 的信息保真度前提） | PRE-PHY-003, 06号文档 H1 |
| TC-SIG-005 | 同一响应，两次独立调用 `extract_signature()` | 无随机性输入 | 两次输出逐字节相同（确定性函数） | PRE-ML-001, PRE-NFR-003 |

## TC-ENC：Physical Encoder

| ID | 前置条件 | 输入 | 期望输出 | 需求/设计出处 |
|---|---|---|---|---|
| TC-ENC-001 | 已拟合的 `FeatureNormalizationStats`（encoder_version=1） | 一个 `PhysicalSignature` | 返回 `EmbeddingSet`，四个子向量维度符合约定，且 `encoder_version=1` | 08§8.1 |
| TC-ENC-002 | 两个不同 `encoder_version` 的 `FeatureNormalizationStats` | 同一 signature 分别编码 | 两次输出的 `encoder_version` 字段不同，向量不可直接比较（接口层面不提供跨版本 cosine 计算） | PRE-ML-003, PRE-REPRO-002 |
| TC-ENC-003 | 归一化统计量的某特征标准差为 0（退化情形） | 编码含该特征的 signature | 不产生除零 NaN（实现需对零标准差做保护，如加 epsilon） | 08§8.2（隐含数值稳定性要求，实现阶段需补充） |

## TC-ATLAS：存储

| ID | 前置条件 | 输入 | 期望输出 | 需求/设计出处 |
|---|---|---|---|---|
| TC-ATLAS-001 | 空 Atlas | 写入一条完整 `PhysicsExperience` | relational/vector/blob 三处存储均成功写入，且可通过 `experience_id` 关联读回 | PRE-DATA-001~003, 08§9 |
| TC-ATLAS-002 | Atlas 中已有记录 | 仅查询 metadata（不请求 response） | 不触发 blob 文件读取（可通过文件句柄计数或 mock IO 验证） | PRE-DATA-002, 08§9.1 |
| TC-ATLAS-003 | 写入过程中在 blob 写入后、relational 写入前中断（模拟故障） | 重新查询该 experience_id | 系统能检测到不一致状态（`ATLAS_BLOB_NOT_FOUND` 或等价的一致性校验），不返回损坏的部分记录 | 08§15 错误码, PRE-REL-002 |
| TC-ATLAS-004 | 两个不同 `encoder_version` 的索引均存在 | 按 `encoder_version=1` 检索 | 结果只来自该版本索引，不与 `encoder_version=2` 的向量混合 | PRE-ML-003 |

## TC-RET：检索

| ID | 前置条件 | 输入 | 期望输出 | 需求/设计出处 |
|---|---|---|---|---|
| TC-RET-001 | Atlas 含 10K 条记录（同 encoder_version） | 一个 query embedding，top_n=100 | P95 延迟 < 200ms（对应性能基准，见 06号文档） | PRE-VEC-001 |
| TC-RET-002 | Atlas 含已知 ground-truth 相似记录 | 以该记录的 embedding 为 query | ground-truth 记录出现在 Top-N 结果中（召回率验证） | PRE-VEC-002, H1 |
| TC-RET-003 | metadata filter 条件极窄（如仅匹配 0.1% 记录） | 执行 search + filter | 若过滤后候选数低于阈值，产生 `recall_shortage` 日志记录（不是报错，是可观测事件） | 08§10.2, ISS-007 |
| TC-RET-004 | 全部候选的 `retrieval_score` 均低于 `novel_similarity_threshold` | 执行 search | 返回 `NovelDynamics{reason: LowSimilarity}`，不强行返回最近结果 | PRE-FR-011, 08§10.4 |
| TC-RET-005 | 已知 `behavior_vector` 检索质量与整体检索质量的对照数据 | 单独用 `behavior_vector` 检索 vs 用融合分数检索 | 两者结果集不同且可独立评估（验证子向量独立可查询性） | PRE-VEC-004 |

## TC-VER：仿真验证

| ID | 前置条件 | 输入 | 期望输出 | 需求/设计出处 |
|---|---|---|---|---|
| TC-VER-001 | 一个候选 Experience，观测响应已知 match_ratio=0.65 | 调用 `verify()` | `HeldOutWindow` 长度约为总时长的 35%，且比较仅在该窗口内进行（可通过 mock 验证未使用 MatchWindow 数据点） | 08§11.1~11.2, ISS-009 |
| TC-VER-002 | 候选完全等于观测来源（自比较，理想情形） | 调用 `verify()` | `verification_score` 达到近乎最优（数值容差内趋近 0 误差），作为正确性基线用例 | 08§11.2 |
| TC-VER-003 | 候选与观测宏观完全不同（如布料 vs 刚体碰撞） | 调用 `verify()` | `verification_score` 明显劣于阈值，触发 `NovelDynamics{reason: HighSimulationError}` 路径（若该候选是唯一/最优候选） | PRE-FR-011, 08§11.3 |
| TC-VER-004 | 构造 Top-M 候选，其材料参数差异大但 `verification_score` 接近 | 调用 `detect_identifiability()` | 返回 `Identifiability::Low`，输出层需展示全部 Top-M 而非单一 best | ISS-006, 08§11.4 |
| TC-VER-005 | Top-M 候选参数集中、误差分散 | 调用 `detect_identifiability()` | 返回 `Identifiability::Normal` | 08§11.4（负例，避免假阳性） |
| TC-VER-006 | 仿真过程中 solver 返回错误 | 调用 `verify()` | 返回 `VERIFICATION_RESIM_FAILED` 及具体原因，不吞异常、不返回默认分数 | PRE-REL-002 |

## TC-REF：参数优化

| ID | 前置条件 | 输入 | 期望输出 | 需求/设计出处 |
|---|---|---|---|---|
| TC-REF-001 | 已知目标函数的简单凸测试问题（非物理，纯数值验证 optimizer 本身） | `local_search()` 调用 | 收敛到已知最优解附近（数值容差内） | 08§12.1 |
| TC-REF-002 | 物理场景：给定初始猜测与目标观测 | `local_search()` + `pre-verify::verify` 作为 objective | 若干次迭代后 `verification_score` 相比初始猜测显著改善 | PRE-FR-010 |
| TC-REF-003 | 优化预算设为极小值（如 5 次仿真） | `local_search()` 调用 | 在预算耗尽时返回当前最优（而非报错或死循环），标记未收敛 | 08§15 错误码 `REFINEMENT_BUDGET_EXHAUSTED` |
| TC-REF-004 | 暴力优化（从随机初始猜测开始）vs ANN 检索初始猜测 + 局部优化 | 相同预算下运行两条路径 | 记录两者达到同等误差阈值所需仿真次数，产出 H3 实验所需原始数据 | H3, 06号文档 |

## TC-GEN：数据集生成器

| ID | 前置条件 | 输入 | 期望输出 | 需求/设计出处 |
|---|---|---|---|---|
| TC-GEN-001 | 定义参数空间（如布料 bend/stretch/damping 三维区间） | `sample_parameter_space(n=1000, strategy=LHS)` | 返回 1000 个样本，其边际分布在各维度上近似均匀（LHS 特性可用统计检验验证） | PRE-FR-014, 08§13.1 |
| TC-GEN-002 | 批量生成任务中部分样本触发数值发散 | 运行批量生成 | 发散样本不写入 Atlas，但被完整记录进 `GenerationFailureLog`（含参数与原因） | ISS-008, 08§13.2 |
| TC-GEN-003 | 多核环境 | 并行运行 N 个独立生成任务 | 各任务结果与单独串行运行结果一致（无共享状态引发的数据竞争） | PRE-PERF-002 |

## TC-ENG：宿主中立契约（`pre-engine-api`）

| ID | 前置条件 | 输入 | 期望输出 | 需求/设计出处 |
|---|---|---|---|---|
| TC-ENG-001 | workspace 全部成员已构建 | 从 `cargo metadata` 动态枚举成员，排除各适配层 crate 后逐个 `cargo tree -p` | 均不出现任何宿主 SDK（`bevy`/`godot`/`pyo3`）；新增 crate 自动纳入检查（不依赖硬编码名单） | PRE-ENG-002, 02号§33.8, AC-06 |
| TC-ENG-002 | 不启用任何适配层 feature | `cargo build --workspace --exclude pre-bevy --exclude pre-godot` | 构建成功，不拉取任何宿主 SDK | PRE-ENG-002 |
| TC-ENG-003 | `sample_times = [0.0, 1.0, 2.0]` | `PlaybackCursor::sample(landmark, t=0.5)` | 首尾两帧的线性插值结果（alpha=0.5） | PRE-ENG-004, 08§18.2 |
| TC-ENG-004 | 同上 | `sample(t=-1.0)` 与 `sample(t=5.0)` | 分别钳制返回首帧/末帧，不外推、不 panic | 08§18.2（Before/After） |
| TC-ENG-005 | `sample_times` 仅 1 个采样点 | `sample(...)` | 返回该唯一采样点 | PRE-ENG-004, 08§18.2（SinglePoint） |
| TC-ENG-006 | `sample_times` 为空 | `sample(...)` | 返回 `PlaybackError::EmptyResponse`，不 panic | 08§18.2（Empty） |
| TC-ENG-007 | 传入不属于该 response 的 landmark | `sample(...)` | 返回 `PlaybackError::UnknownLandmark`（**非 panic**——Tier 2/3 下 panic 不得跨边界） | PRE-EMB-004, 08§18.2 |
| TC-ENG-008 | `SpatialConvention::PRE_CANONICAL` | 对已知变换调用 `convert()` | 输出等于输入（恒等换算） | PRE-ENG-006, 08§18.1 |
| TC-ENG-009 | `SpatialConvention::UNREAL`（左手系 + Z-up + ×100） | 对已知变换调用 `convert()` | 手性翻转、轴置换、缩放 100 倍均正确——**本用例是换算路径真正通畅的唯一证据**，不可省略（Bevy/Godot 换算近似恒等，无法暴露该路径缺陷） | PRE-ENG-006, ADR-011, 08§18.4 |
| TC-ENG-010 | `SpatialConvention::UNITY`（左手系 Y-up） | 同上 | 仅手性翻转，无轴置换与缩放 | PRE-ENG-006 |
| TC-ENG-011 | 已提交一个耗时明显超过一帧的查询 | 反复调用 `QuerySession::poll()` | 完成前返回 `Running`，完成后返回 `Done(...)`；`poll()` 本身始终立即返回（非阻塞） | PRE-ENG-005, 08§18.3 |
| TC-ENG-012 | 查询进行中 | 调用 `cancel()` 后 `poll()` | 会话终止，不再返回 `Done`，且不泄漏后台任务 | 08§18.3 |
| TC-ENG-013 | `pre-engine-api` 的公开类型集合 | 静态检查：契约层公开 API 中是否出现无法跨 C ABI 表达的类型 | `LandmarkTransform` 等传输类型均为 POD（数组/标量），无泛型/trait 对象/带载荷枚举——保证 Tier 2 将来可接入 | PRE-ENG-009, 08§18.1, 08§21.3 |

## TC-BEVY：Tier 1 适配层（Bevy）

| ID | 前置条件 | 输入 | 期望输出 | 需求/设计出处 |
|---|---|---|---|---|
| TC-BEVY-001 | 最小 Bevy `App` 注册 `PrePlugin`，加载一条 Experience | 推进若干虚拟帧（`app.update()`，每帧推进 `Time`） | 各 `PreLandmark` 实体的 `Transform.translation` 按 `PlaybackCursor` 的预期序列变化 | PRE-ENG-101, 08§19, AC-06 |
| TC-BEVY-002 | 非循环模式，推进帧数超过 `response.duration` | 检查播放结束事件 | 事件触发且仅触发一次 | PRE-ENG-101, 08§19 |
| TC-BEVY-003 | 启用 `query_bridge`，提交一个耗时远超单帧的查询 | 推进多帧 | 单帧 `app.update()` 墙钟耗时不随查询耗时增长（查询在 `AsyncComputeTaskPool` 中执行，不阻塞主 Schedule） | PRE-ENG-005, PRE-EMB-002, 08§19 |
| TC-BEVY-004 | `pre-bevy` crate 文档 | 检查顶层文档 | 明确声明所支持的单一 Bevy 主版本 | PRE-ENG-007, 08§19 |

## TC-GODOT：Tier 1 适配层（Godot）

| ID | 前置条件 | 输入 | 期望输出 | 需求/设计出处 |
|---|---|---|---|---|
| TC-GODOT-001 | Godot 测试场景含 `PrePlaybackNode` 及绑定的子 `Node3D` | 推进若干 `_process` 周期 | 子节点 `Transform3D.origin` 按预期序列变化 | PRE-ENG-201, 08§20 |
| TC-GODOT-002 | 节点尚未进入场景树 | 在构造时刻检查 `bindings` | 绑定在 `ready()` 中建立而非构造时（构造时子节点不可用） | 08§20（Godot 特有约束 2） |
| TC-GODOT-003 | 提交一个耗时远超单帧的查询 | 推进多帧 | `_process` 不被阻塞；结果经 `call_deferred` 在主线程写入，未从工作线程直接操作场景树 | PRE-ENG-005, 08§20（Godot 特有约束 1） |
| TC-GODOT-004 | `pre-godot` crate 文档 | 检查顶层文档 | 明确声明所支持的单一 Godot 主版本 | PRE-ENG-007 |

## TC-CONF：跨适配层一致性套件（PRE-ENG-008）

| ID | 前置条件 | 输入 | 期望输出 | 需求/设计出处 |
|---|---|---|---|---|
| TC-CONF-001 | 同一条黄金响应数据 | 分别经 `pre-bevy` 与 `pre-godot` 回放并采样 | 两者变换序列在容差内一致——**这是验证中立契约是否真的中立的唯一实证手段**；若不一致，说明本应中立的逻辑事实上泄漏进了某个适配层 | PRE-ENG-008, AC-07, 02号§33.7 |
| TC-CONF-002 | 同一条黄金响应数据 | 适配层输出 vs `conformance_reference()` 参考实现 | 在容差内一致 | PRE-ENG-008, 08§18.4 |
| TC-CONF-003 | 新增任一适配层时 | 运行同一套件 | 无需为该适配层定义新的验收口径即可判定通过/失败 | PRE-ENG-008 |

## TC-GPU：GPU 后端（MVP 不实现，用例为实现后生效）

| ID | 前置条件 | 输入 | 期望输出 | 需求/设计出处 |
|---|---|---|---|---|
| TC-GPU-001 | MVP 阶段（GPU 未实现） | 检查 `PreContextDesc` 结构定义 | `gpu: GpuDeviceSource` 字段存在（即使恒为 `Disabled`）——验证 ADR-012 的架构预留是否落实 | PRE-GPU-002, ADR-012, 08§22.1 |
| TC-GPU-002 | 无可用 GPU 的环境 | 以 `CreateOwn` 构造上下文 | 自动回退 CPU 路径并记录明确原因，不失败退出 | PRE-GPU-005, 08§22.3 |
| TC-GPU-003 | GPU 实现后 | 同一 solver 分别跑 CPU reference 与 GPU | 数值在配置容差内对齐 | PRE-GPU-003, PRE-PHY-002 |
| TC-GPU-004 | `pre-solver-*` 源码 | 静态检查/lint | 不出现任何 `wgpu`/Vulkan/D3D 专有类型（仅调用 `pre-gpu` 抽象） | PRE-GPU-001, 08§22.2 |

## TC-EMB：可嵌入性

| ID | 前置条件 | 输入 | 期望输出 | 需求/设计出处 |
|---|---|---|---|---|
| TC-EMB-001 | 单进程 | 创建两个独立 `PreContext` 并各自加载不同 Atlas | 两者互不干扰（验证无全局可变状态） | PRE-EMB-001, 02号§35 |
| TC-EMB-002 | 提交一个长耗时操作 | 从调用线程观察 | 调用立即返回，工作在后台推进，可轮询——调用线程不被阻塞 | PRE-EMB-002 |
| TC-EMB-003 | 配置线程上限为 N | 运行并行任务 | 实际创建线程数不超过 N | PRE-EMB-003 |
| TC-EMB-004 | Tier 2/3 实现后；构造一个会 panic 的内部条件 | 经 C ABI / PyO3 调用 | panic 被 `catch_unwind` 捕获并转换为错误码/Python 异常，**不跨越语言边界** | PRE-EMB-004, 08§21.1, 08§21.2 |

## TC-E2E：端到端集成

| ID | 前置条件 | 输入 | 期望输出 | 需求/设计出处 |
|---|---|---|---|---|
| TC-E2E-001 | 空 Atlas | 执行 16.1 生成路径产出至少一条 Experience，再执行 16.2 检索验证路径以该 Experience 的留出窗口作为观测 | 该 Experience 出现在检索结果中且验证得分达到 Validated 阈值 | AC-01 |
| TC-E2E-002 | 已有一条 Validated Experience | 执行 16.3 学习闭环路径 | 新记录写回 Atlas 并重新索引，后续以相近观测检索可命中该新记录 | AC-05 |
| TC-E2E-003 | 支持 Rigid/XPBD/MPM 三类 solver 的 Atlas | 新增一个玩具级 solver 插件（仅用于测试插件化边界，不要求物理正确） | 除 `pre-solver-*` 新增 crate 外，`pre-retrieval`/`pre-atlas`/`pre-signature` 代码零改动即可处理新插件产生的记录 | AC-03, PRE-NFR-001 |
| TC-E2E-004 | 任一检索结果 | 请求该结果的评分明细 | 返回结构包含 retrieval_score、各子向量相似度、simulation_error 分维度、stability_score、computational_cost、confidence、identifiability，且非仅返回 ID | AC-04, PRE-OBS-001 |

---

## 覆盖检查

本用例集覆盖 08 号文档 14 个模块章节（§2~§13 共12节、§15 错误码、§18 pre-bevy），且每条用例均标注对应需求 ID 或设计出处，可反向验证 04_PRE_Traceability_Matrix.md 中 "Test" 列的具体化实现。未被 TC 覆盖的章节为 §14（pre-cli，MVP 阶段以 smoke test 为主，未纳入本用例集的结构化枚举）、§16（处理流程时序，由 TC-E2E 间接覆盖）、§17（追加映射，非可测试行为）——均为设计说明性质而非独立可测试单元，不视为覆盖缺口。若实施阶段发现某需求 ID 在本文书中无对应 TC，应视为测试覆盖缺口并补充。
