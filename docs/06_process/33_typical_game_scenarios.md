# 典型游戏场景库（Typical Game Scenarios）

> **用途**：与集成方对齐的"GVPE 必跑"真实游戏场景库；用于功能 / 场景 / 性能 / 集成测试。
> **对应工作流步骤**：`28_workflow.md` §10.8 步 79（シナリオ試験）、§10.9 步 93（業務シナリオ試験）、`31_pilot_integration_agreement.md`、`32_system_test_spec_template.md`。
> **关联**：`GVPE-DOC-14`（性能预算）、`GVPE-DOC-15`（测试策略）。

## 0. 元数据

| 字段 | 取值 |
|---|---|
| 库版本 | v0.1 |
| 适用阶段 | MVP / 集成测试 / 性能基线 |
| 适用集成方 | Unity / Unreal / Godot / 自研 |
| 关联设计文档 | `GVPE-DOC-14`, `GVPE-DOC-15` |

## 1. 场景分级

| 等级 | 含义 | 测试必跑 |
|---|---|---|
| **L1 核心** | 任何 MVP release 必跑 | ✅ |
| **L2 推荐** | 性能 / 集成测试必跑 | ✅ |
| **L3 扩展** | 集成方可选 / 专项测试 | 选跑 |

## 2. L1 核心场景

### 2.1 Single Sphere Drop（单球下落）

- **描述**：1 个 sphere 从 10m 高度静止释放，受重力下落至地面。
- **数量**：1 body。
- **期望**：落到地面静止；sleeping 触发。
- **性能目标**：1000 Hz trivial（不在性能基线）。
- **关联**：`R-FR-002` 求解器 MVP 范围。

### 2.2 Box Stack（箱子堆叠）

- **描述**：10 个 box 垂直堆叠（每层 1 个 box），最低层静止。
- **数量**：10 body。
- **期望**：稳定 5 秒无滑动 / 穿透。
- **性能目标**：≥ 60 Hz（中端 PC）。
- **关联**：`R-FR-002`, `R-PERF-001`。

### 2.3 Sphere Pile（球堆）

- **描述**：500 个 sphere 随机位置落入容器，形成堆。
- **数量**：500 body。
- **期望**：稳定 3 秒；broad phase 有效剪枝。
- **性能目标**：≥ 60 Hz（中端 PC），≥ 30 Hz（低端 PC）。
- **关联**：`R-PERF-001`。

### 2.4 Jointed Pendulum（关节摆）

- **描述**：2 段 box 用 hinge joint 连接，从水平位置释放。
- **数量**：3 body（2 段 + 锚）。
- **期望**：摆动符合单摆物理；最终静止。
- **关联**：`R-FR-002` 关节。

### 2.5 Sleeping Validation（睡眠验证）

- **描述**：1000 个 sphere 静止 5 秒，验证 sleeping 触发后 step 耗时下降。
- **数量**：1000 body。
- **期望**：前 1 秒正常求解；1 秒后 sleeping；step 耗时下降 80%+。
- **关联**：`R-FR-002` sleeping，`QA-I-03`。

## 3. L2 推荐场景

### 3.1 Tower of Pisa（比萨斜塔）

- **描述**：20 层 box 堆叠，每层偏移 5%，形成斜塔。
- **数量**：20 body。
- **期望**：稳定（与现实一致）。
- **挑战**：接触流形稳定性。
- **关联**：`QA-I-02` 接触流形。

### 3.2 Bullet Through Wall（子弹穿薄板）

- **描述**：高速 sphere 撞向薄 box，验证 CCD 触发。
- **数量**：2 body。
- **期望**：CCD 触发，碰撞检测到，无穿透。
- **关联**：`D-18` CCD，`QA-I-04`。

### 3.3 Rope Bridge（绳桥）

- **描述**：100 段 rope 模拟绳桥，重物放中间。
- **数量**：100+ body。
- **期望**：绳子下垂符合物理；性能可接受。
- **关联**：`D-19` XPBD 软体（Gen 2）。

### 3.4 Character Ragdoll（角色布娃娃）

- **描述**：人形布娃娃 ~15 段关节，跌落。
- **数量**：15 body + ~14 joint。
- **期望**：关节不脱开；布娃娃跌落自然。
- **关联**：`R-FR-002` 关节。

### 3.5 Newton Cradle（牛顿摆）

- **描述**：5 个 sphere 排列成牛顿摆，1 个拉起释放。
- **数量**：5 body。
- **期望**：动量守恒（近似），左右交替摆动。
- **关联**：求解器正确性。

### 3.6 Domino Chain（多米诺）

- **描述**：100 个 box 等距排列，1 个推倒触发链式。
- **数量**：100 body。
- **期望**：链式倒伏；性能稳定。
- **挑战**：长时间仿真稳定性。

### 3.7 Stress Test 10K Bodies（1 万 body 压力）

- **描述**：10000 个 sphere 紧密堆叠。
- **数量**：10000 body。
- **期望**：不 OOM，broad phase 剪枝有效。
- **性能目标**：≥ 30 Hz（高端 PC）。
- **关联**：`QA-P-01` MVP 性能基线。

### 3.8 Mass Range Test（质量范围）

- **描述**：1 个极重 body（1e6 kg）+ 1 个极轻 body（1e-3 kg）相邻。
- **数量**：2 body。
- **期望**：数值稳定，无 NaN。
- **关联**：`QA-I-02` 数值稳定性。

## 4. L3 扩展场景

### 4.1 Destruction（破坏）

- **描述**：1 个 box 网格在冲击下碎成 ~50 个 fragment。
- **关联**：post-MVP（XPBD + 复杂形状）。

### 4.2 Soft Body Deformation（软体形变）

- **描述**：软体球落地形变恢复。
- **关联**：`D-19` XPBD 软体（Gen 2）。

### 4.3 Vehicle（载具）

- **描述**：4 轮车 + 悬挂关节。
- **关联**：raycast vehicle（post-MVP）。

### 4.4 Cloth Simulation（布料）

- **描述**：布料飘动。
- **关联**：`D-19` XPBD cloth。

### 4.5 Conveyor Belt（传送带）

- **描述**：摩擦驱动的运动平台。
- **关联**：friction model 边界。

## 5. 性能数字基线

> 与 `GVPE-DOC-14` 对齐；具体数字由集成方机器规格决定。

| 场景 | 中端 PC | 高端 PC | 备注 |
|---|---|---|---|
| Box stack 10 | 60 Hz | 60 Hz | L1 |
| Sphere pile 500 | 60 Hz | 120 Hz | L1 |
| 关节布娃娃 15 | 60 Hz | 120 Hz | L2 |
| 1 万 body 压力 | 30 Hz | 60 Hz | L2 |
| 100 万 body 极端 | N/A | 10 Hz | L3，stretch goal |

## 6. 集成方场景对齐流程

1. GVPE 提供 L1 场景库作为必跑；
2. 集成方基于自己的游戏项目添加 L2 / L3 场景；
3. 集成方签字确认场景 + 性能目标（`31_pilot_integration_agreement.md` §3）；
4. 双方共同维护场景库，集成方特化场景脱敏后进入公开版。

## 7. 关联

- `28_workflow.md` §10.8 步 79
- `31_pilot_integration_agreement.md` §1.1
- `32_system_test_spec_template.md` §3
- `34_uat_plan_template.md` §3
- `GVPE-DOC-14`（性能预算）
- `GVPE-DOC-15`（测试策略）

## 8. 审批

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 | — | | |
| 评审 | | | |
| 批准 | | | |
