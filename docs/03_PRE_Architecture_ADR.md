# PRE 架构决策记录（ADR）

## 文书管理表

| 项目 | 内容 |
|---|---|
| 文书编号 | PRE-DOC-03 |
| 版本 | v0.1.3 |
| 状态 | Draft |
| 关联文书 | 02_PRE_Basic_Design.md（各 ADR 在设计书中被引用的位置见其正文） |

## 改订履历

| 版本 | 日期 | 变更内容 |
|---|---|---|
| v0.1 | 2026-08-17 | 初版：ADR-001～ADR-008 |
| v0.1.1 | 2026-08-17 | 补充文书管理表 |
| v0.1.2 | 2026-08-17 | 新增 ADR-009：Bevy 是 Engine Adapter 而非核心依赖 |
| v0.1.3 | 2026-08-17 | ADR-009 由 ADR-010 泛化（单引擎→多宿主分层）；新增 ADR-010（分层宿主集成模型）、ADR-011（规范空间与单位约定）、ADR-012（GPU 设备注入而非自建） |

---

## ADR-001: Why Physical Response rather than Material Label?

**状态**：Accepted

**背景**：核心索引对象是用材料名称（"steel"/"rubber"）还是物理响应（条件+作用→响应）？

**决策**：以 Physical Response（条件+激励→响应）为核心索引对象，材料标签仅作为 metadata 过滤字段。

**理由**：材料标签是离散、粗粒度、且跨材料仍可能共享相似动力学行为（如某些橡胶与某些软体组织在特定应变范围内响应相似）；反之同名材料在不同参数下响应差异巨大。以响应为核心可以捕捉这种连续性，支持跨材料类比检索，直接服务于 inverse physics 的目标（观察到响应、反推可能的材料而非反过来）。

**后果**：Signature/Embedding 设计必须围绕响应特征，而非材料分类特征；材料标签退化为 filter 维度（PRE-VEC-003）。

---

## ADR-002: Why Retrieval is Candidate Generation Only?

**状态**：Accepted

**决策**：向量检索永远只产生候选（Top-N/Top-K），最终判定必须经过 Simulation Verification。

**理由**：Embedding 相似度是学习/工程近似，不具备物理保真度保证；直接以 cosine similarity 作为最终答案违反 "Physics is ground truth" 原则（需求第31/39节），且无法检测 embedding 空间的错误近邻（false neighbor）。将检索限定为候选生成，把最终正确性判定交还给可信的正向仿真，是本项目区别于"纯 AI 检索系统"的核心设计立场。

**后果**：Pipeline 必须包含 Verification 阶段（不可选）；性能上多一轮仿真开销，但换取正确性保证（对应 H4 待验证假设）。

---

## ADR-003: Why Standard Response is Solver Independent?

**状态**：Accepted

**决策**：引入 Raw Solver State → Standard Physical Response → Physical Signature → Physical Embedding 四层分离，其中 Standard Response 层与具体 solver 解耦。

**理由**：若 Signature/Encoder 直接依赖某个 solver 的内部状态格式（如 XPBD 约束残差数组、MPM 粒子/网格双表示），则新增 solver 需要改动检索与编码核心代码，违反插件化目标（PRE-NFR-001），也使跨 solver 检索（H2）在架构上不可能。四层分离将"solver 特定"与"物理语义"显式切割，转换逻辑收敛在每个 solver 插件自己的 `to_standard_response` 实现中。

**后果**：每个新 solver 插件必须实现该转换，工程成本前置到插件开发阶段；但换来检索/编码/存储核心代码零改动扩展性。

---

## ADR-004: Why 3DGS is Observation Backend?

**状态**：Accepted

**决策**：Dynamic 3DGS/4DGS 仅作为 `ObservationBackend` 的一种未来实现，产出必须落到与仿真侧相同的 `StandardPhysicalResponse` schema，核心 Runtime 不感知 3DGS 存在。

**理由**：3DGS 是观测/重建技术，其表示（Gaussian 数量、协方差、拓扑）与物理语义无直接映射关系，且技术本身快速演进（3DGS→4DGS→未来方法）。若核心数据模型绑定 3DGS 表示，未来技术更替将引发核心重构。将其限定为 Observation Backend 的一种实现，是 PRE-FR-015 与非目标 NG2（不实现 3DGS/4DGS 重建或渲染管线本身）所要求的"禁止强耦合"的直接落实。

**后果**：V0.1 不实现该 backend，仅预留 trait；Phase 2 设计必须证明"能把 3DGS 输出转成 Standard Response 且保留足够物理信息"，否则该抽象需重新评估。

---

## ADR-005: Single Vector vs Multi Vector?

**状态**：Accepted（MVP 采用简化版 Multi-Vector：3 个语义子向量 + 1 个融合用 global_vector）

**背景**：单一 embedding 是否足以表达全部物理性质？

**决策**：采用 Multi-Vector（behavior/deformation/temporal + global），而非单一向量；但 MVP 刻意只取三个语义子空间（而非需求文档列出的六个），并用一个 global_vector 承担 ANN 粗召回索引角色。

**理由**：单一向量会把运动学、形变、时序等异质信息压缩进同一距离空间，容易造成"总体相似但物理上错误近邻"（如运动轨迹相似但材料行为完全不同）。但完全体的六子向量在 MVP 阶段验证成本过高（每多一个子空间即多一套消融实验与索引维护）。三个语义子向量是"能支撑 H1/H2 消融实验的最小可验证集合"，global_vector 用于控制 ANN 索引维度和粗召回效率，是工程折衷而非否定完全体设计。

**后果**：contact_vector / material_vector / constraint_vector 留待 Phase 2，根据 MVP 阶段消融实验结果决定是否拆分（若 behavior_vector 内部出现明显的检索质量瓶颈则拆出 contact_vector 等）。

---

## ADR-006: Relational + Vector + Blob storage strategy?（含图数据库结论）

**状态**：Accepted

**决策**：MVP 采用「Relational(metadata) + Vector(embedding) + Blob(原始响应)」三分存储，三者通过 `experience_id` 关联；**不引入图数据库**。

**理由**：
- metadata（solver 类型、材料类别、验证状态等）具有明确 schema、需要精确过滤查询 → 关系型/文档型最合适。
- embedding 需要近似最近邻查询，语义与前者完全不同 → 专用向量索引。
- 原始响应（逐帧场数据）体积大、访问模式是"按 id 整体读取"而非"查询" → blob/对象存储最合适，且必须与前两者物理分离以满足 PRE-DATA-002（metadata/embedding 查询不触发大体积读取）。
- 图数据库：需求文档要求"证明关系型/向量存储无法满足查询模式后才可引入"（PRE-DATA-004）。MVP 规模与查询模式（主要是"按条件过滤 + 向量检索"，无深度多跳关系遍历需求）不构成引入图数据库的证据。Entity/Relation/Field 若真需要表达为图，MVP 阶段用关系表的外键 + JSON 字段可以覆盖。

**后果**：若 Phase 3（Inverse Physics 复杂约束/接触图）出现明确的多跳查询需求（例如"找出所有通过约束链间接影响此刚体的场"），需重新开 ADR 评估图数据库引入。

---

## ADR-007: Metadata Filtering — Post-filter vs Pre-filter

**状态**：Accepted（MVP：post-filter）

**决策**：MVP 使用 post-filter：先对 `global_vector` 做 ANN Top-N 召回，再按 metadata 过滤到 Top-K。

**理由**：Pre-filter（先按 metadata 分区再各分区建索引）在 metadata 组合数量增长时会造成索引数量爆炸和维护复杂度上升；MVP 规模（10K~100K）下 post-filter 的"召回不足风险"可以通过适当放大 N 缓解，工程复杂度显著更低。

**后果**：若线上验证发现某些稀有 metadata 组合导致 post-filter 后候选不足（Top-N 召回后过滤剩余过少），需要评估分区索引或加大 N 的动态策略，记为 Evolution Strategy 触发条件之一。

---

## ADR-008（合并说明）: 图数据库暂缓引入

见 ADR-006 后半部分，不单独重复展开。

---

## ADR-009: Why Bevy is an Engine Adapter, not a Core Dependency?

**状态**：Accepted — 其原则已由 **ADR-010 泛化**至全部宿主（Bevy/Godot/Unity/Unreal/DCC）。本 ADR 保留作为该原则的首次提出与论证记录，具体约束以 ADR-010 与 PRE-ENG-002 为准。

**背景**：用户要求 PRE 优先支持 Bevy（Rust 原生 ECS 游戏引擎）集成，用于回放仿真结果、未来可能双向对接场景数据。核心问题：Bevy 依赖应该渗透进 `pre-core`/`pre-solver-*`/`pre-retrieval` 等核心 crate，还是被限定在一个独立的适配层？

**决策**：新增 `pre-bevy` crate 作为唯一允许依赖 `bevy` 的组件；`pre-core` 及其余全部核心 crate 不得直接或传递依赖 `bevy`。`pre-bevy` 单向依赖 `pre-core`（读取核心数据类型），核心不反向依赖 `pre-bevy`。

**理由**：
1. **与 ADR-004（3DGS 是 Observation Backend）同构的风险**：Bevy 是一个独立演进、迭代速度快、historically 有多次破坏性 API 变更（ECS 存储模型、渲染管线、Schedule 机制均经历过大改）的外部项目。若核心数据结构（如 `StandardPhysicalResponse`）直接使用 Bevy 类型（如 `bevy::Transform`），Bevy 每次主版本升级都会强制核心跟着改动或锁死在旧版本——这与四层分离模型（ADR-003）保护核心免受外部表示变化影响的初衷直接冲突。
2. **PRE 的核心价值与渲染引擎无关**：PRE 的价值在于物理响应的检索与验证能力，这个能力不应该要求下游使用者引入一个完整游戏引擎作为依赖。保持核心 crate 的 `bevy`-free，意味着 PRE 可以被嵌入到任何 Rust 项目（CLI 工具、Web 服务、其它引擎的适配层）而不被迫拖入 Bevy 的整个依赖树（渲染后端、窗口系统、音频等）。
3. **不违反 NG1**：适配层不实现渲染/场景管理/脚本系统，只做数据搬运（`StandardPhysicalResponse` → `Transform` 时间序列；未来可选地 Bevy 场景 → `InitialState`），真正的引擎能力仍完全来自 Bevy 本身，PRE 没有"变成"游戏引擎。

**后果**：
- 不使用 `pre-bevy`（或未来其它引擎适配层）的用户，其构建产物中不包含 Bevy 的任何代码/依赖，编译时间与二进制体积不受影响。
- 新增一个"跨 crate 边界的数据搬运层"，意味着 `pre-bevy` 需要独立维护自己的类型转换代码（`LandmarkId` ↔ Bevy `Entity`，`Vec3`(PRE) ↔ `Vec3`(Bevy/glam) 等），属于可接受的工程成本，且这类转换代码天然是可单元测试的边界代码。
- `pre-bevy` 的版本演进与 Bevy 主版本绑定（见 01号文档 OQ-07），需要单独的发布节奏，不与核心 crate 的版本号统一管理。
- 若未来出现第二个 Engine Adapter（如 Godot-rust、Unity FFI 绑定等），应遵循同一模式新增独立 crate，而不是把多个引擎的类型揉进核心或揉进 `pre-bevy` 本身。


---

## ADR-010: 宿主集成为何按「接入机制」分层，而非为每个引擎各写一套？

**状态**：Accepted（泛化并取代 ADR-009 的适用范围）

**背景**：目标宿主包含 Bevy（Rust）、Godot（Rust 绑定 / GDExtension）、Unity（C#）、Unreal（C++），以及 Blender/Maya/Houdini 等以 Python 为主要扩展语言的 3D 软件。ADR-009 当初只面对 Bevy 一个宿主，其结论「做成独立适配 crate」在单宿主下成立，但没有回答多宿主下的两个问题：适配层之间如何避免重复实现？非 Rust 宿主如何接入？

**决策**：按**接入机制**分为四层——Tier 0（Rust 库 API）、Tier 1（Rust 链接适配层：Bevy/Godot）、Tier 2（C ABI：Unity/Unreal）、Tier 3（Python 绑定：DCC 软件）；并新增宿主中立契约 crate `pre-engine-api`，承载全部宿主共用逻辑（回放插值、查询会话状态机、中立变换类型、坐标单位换算）。适配层只保留「中立类型 ↔ 宿主类型映射」与「宿主特有调度接入」。

**理由**：

1. **按接入机制分层，而非按宿主分层**，是因为真正决定实现方式的是 ABI 与语言生态，不是引擎的品牌或知名度。Bevy 与 Godot 虽是两个完全不同的引擎，但同为「Rust 可直接链接」，其适配层结构高度相似；而 Unity 与 Unreal 虽然一个 C# 一个 C++，却共享同一个真正的技术约束——必须经过 C ABI，不能传递 Rust 专有类型。按宿主分层会让这个共性被四份重复代码掩盖。

2. **中立契约层的必要性来自一个具体的失效模式**：回放插值若由各适配层自行实现，四个宿主就有四份插值代码，而插值的困难之处恰恰在边界情形（时间早于首个采样点、晚于末个采样点、只有单个采样点）。这些边界几乎必然被四份实现处理得不一致，且这类不一致在单个宿主内测试时完全看不出来——只有跨宿主比对才会暴露。把它收敛到一处，才使 AC-07（两个适配层输出一致）成为一个有意义的验收项。

3. **Tier 2 的边界约束必须在设计阶段确定，即使不实现**。C ABI 无法表达 Rust 泛型、trait 对象、带载荷枚举、`Result`。若 `pre-engine-api` 的中立契约中出现这类类型，Tier 2 将永远无法接入，而这个问题在只做 Bevy/Godot 时完全不会暴露（它们能直接消费这些类型）。因此 Tier 2 的约束实际上是**反向约束了中立契约的设计**——这是"先规划形态"最实在的收益。

**后果**：

- 新增 `pre-engine-api` 一层，Tier 1 适配层比原 `pre-bevy` 更薄。
- PRE-BEVY-001~006 废止，迁移至 PRE-ENG-*（对照表见 01号文档 §29.5）。
- MVP 只实现 Tier 1 的 Bevy（Godot 紧随其后），Tier 2/3 仅设计不实现——但其约束立即生效于中立契约的设计。
- 若未来出现第五类宿主（如 C 语言宿主、WASM 宿主），应先判断它落入哪个 Tier，而非默认新增一层。

---

## ADR-011: 为何要定义 PRE 自己的规范空间与单位约定？

**状态**：Accepted

**背景**：目标宿主的坐标约定并不统一——Bevy/Godot 为右手系 Y-up，Unity 为**左手系** Y-up，Unreal 为**左手系 Z-up 且默认单位为厘米**，Blender 为右手系 Z-up。

**决策**：PRE 定义唯一规范约定（右手系、Y-up、-Z 前向、SI 米），并在 `pre-engine-api` 中以数据形式描述每个宿主的换算（`SpatialConvention`），由各适配层声明所用换算；换算函数集中实现并纳入一致性测试套件（PRE-ENG-008）。

**理由**：若不在契约层固定规范约定，各适配层会各自"就地修正"手性与单位。这类缺陷的表现是「同一条物理响应在 Unreal 里镜像了、在 Unity 里旋转方向反了、尺度差 100 倍」，而归因极其困难——因为它可能来自 solver、Response 记录、适配层映射或宿主自身设置中的任何一处。

更关键的是**一个只做 Bevy 和 Godot 就必然踩中的陷阱**：这两个宿主的换算都接近恒等，因此换算层极可能被实现成隐式恒等（甚至根本没有换算层）。等到接入 Unreal 时才会发现，整个数据通路里**没有可以插入换算的位置**——这不是补一个函数就能解决的问题，而是要在已成型的通路上开口子。

**后果**：即使 MVP 只交付 Bevy（换算恒等），`SpatialConvention` 也必须真实存在并被调用，不允许因为"当前恒等"而省略。一致性测试套件中应至少包含一个非恒等换算的用例（可用 Unreal 的换算参数构造，无需真实 Unreal 环境），以证明该路径确实通畅。

---

## ADR-012: GPU 设备由宿主注入，而非 PRE 自建

**状态**：Accepted

**背景**：PRE 的 solver 存在 GPU 加速需求（02号文档 §21/§34），而宿主（游戏引擎、3D 软件）运行时几乎必然已持有一个 GPU 设备。

**决策**：PRE 的 GPU 初始化架构以「接受宿主注入设备/队列」为主路径，仅在宿主未提供时自建设备。即使 MVP 不实现 GPU 后端，初始化架构也必须预留该形态。

**理由**：

1. 宿主已有设备时 PRE 另建一个，会导致显存重复占用、跨设备同步开销，以及部分驱动下的资源共享失败——而资源共享正是 GPU 加速在嵌入场景下的价值所在。
2. **这两种形态在架构上不兼容，且改造代价不对称**。「自建并独占设备」会把设备创建、生命周期管理与错误处理散布在初始化路径的各处；改为「接受注入」时这些位置全部要重写。反之，先按「可注入」设计，退化为自建只是传入 `CreateOwn` 的一个分支。
3. 因此这属于「即使不实现也必须现在决定」的一类，与 PRE-EMB-001（无全局状态、显式上下文句柄）同源——两者都是可嵌入性对初始化架构的硬约束。

**后果**：`PreContext` 的构造必须从一开始就能接收可选的外部 GPU 设备，即便 MVP 阶段该参数恒为 `None`。跨宿主导出可共享设备句柄的可行性逐宿主不同（OQ-10），该评估结论可能反过来限制 PRE-GPU-004 零拷贝互操作的可达范围。
