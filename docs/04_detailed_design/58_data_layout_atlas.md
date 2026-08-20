# 数据布局图谱（Data Layout Atlas）

> **用途**：所有公共 struct 的内存布局、对齐、padding、SoA/AoSoA 策略的**权威登记**。
> **对应工作流步骤**：44 クラス設計、46 API詳細設計 → `28_workflow.md` §10.4 步 44/46。
> **关联**：`GVPE-DOC-17` §1（核心 struct）；`GVPE-DOC-26` §18.5（SIMD 策略）；`GVPE-DOC-25`（GPU 上传性）；`57_coding_standards.md`（unsafe 政策）。

## 0. 元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-58 |
| 文档类型 | 詳細設計書 |
| 版本 | v0.1 |
| 状态 | Draft |
| 适用阶段 | MVP / 实施期 |
| 上游文档 | `GVPE-DOC-17` §1, `GVPE-DOC-26` §18.5, `GVPE-DOC-25` |
| 下游文档 | 实施期 PR / `38_code_review_checklist.md` |

## 1. 布局原则

### 1.1 三类布局策略

| 策略 | 含义 | 适用 | 性能影响 |
|---|---|---|---|
| **AoS**（Array of Struct） | 标准 C 风格结构体 | 冷路径、配置、错误类型 | 字段局部性差 |
| **SoA**（Struct of Array） | 字段拆为独立数组 | 热路径、热数据 | cache 友好，SIMD 友好 |
| **AoSoA**（Array of Struct of Array） | chunk 内 SoA，跨 chunk AoS | 部分场景 | 平衡 |

### 1.2 选择规则

| 数据访问模式 | 推荐 |
|---|---|
| 整体对象频繁使用 | AoS |
| 单一字段批量处理（积分 / 力计算） | SoA |
| 字段之间偶尔交叉 | AoSoA（chunk = 64 / 128） |
| 跨 FFI 边界 | AoS + `#[repr(C)]` + `bytemuck::Pod` |
| GPU 上传 | SoA + 紧凑数组（依 `25_gpu_backend_detailed_design.md`） |

### 1.3 对齐与 padding

- 默认 4 字节对齐；
- SIMD 路径 16 / 32 字节对齐（`#[repr(C, align(16))]` / `align(32)`）；
- 显式 padding：仅在 `bytemuck::Pod` 推导需要时；
- 跨 FFI 边界：显式 align 防止 ABI 差异。

### 1.4 端序（endianness）

- 假设小端（x86_64 / aarch64 主目标）；
- 跨平台字节序无关：不使用 `transmute` 跨端序；
- 序列化（post-MVP）：使用 `zerocopy` / `rkyv`（评估中）。

## 2. 核心类型布局

### 2.1 句柄类型

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct BodyHandle {
    pub index: u32,        // 偏移 0
    pub generation: u32,   // 偏移 4
}
// size = 8, align = 4

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct ConstraintHandle {
    pub index: u32,        // 偏移 0
    pub generation: u32,   // 偏移 4
}
// size = 8, align = 4

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct IslandHandle(pub u32);
// size = 4, align = 4
```

- `repr(C)` 保证布局稳定（跨编译 / 跨 FFI）；
- `bytemuck::Pod` derive 安全（u32 trivially pod）；
- 测试：`assert_eq!(std::mem::size_of::<BodyHandle>(), 8);`

### 2.2 `PhysicsProfile`（POD）

```rust
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicsProfile {
    pub mass: f32,            // 0
    pub density: f32,         // 4
    pub inertia: [f32; 9],    // 8-43（3x3 tensor，平坦化，行优先）
    pub friction: f32,        // 44
    pub restitution: f32,     // 48
    pub damping_linear: f32,  // 52
    pub damping_angular: f32, // 56
    pub stiffness: f32,       // 60
    pub compliance: f32,      // 64
    pub viscosity: f32,       // 68
    pub solver_type: SolverTypeId,    // 72 (u8 + padding)
    pub solver_iterations: u16,        // 74
    pub collision_profile: CollisionProfileId,  // 76 (u8 + padding)
    pub approximation_level: PhysicsLodTag,     // 78 (u8 + padding)
    pub _padding: [u8; 1],   // 79
}
// size = 80, align = 4
```

- 字段顺序按访问频率从高到低；
- `inertia: [f32; 9]` 不用 `Mat3`（避免依赖 + 简化 FFI）；
- padding 显式标注（避免编译器 reorder）；
- `bytemuck::Pod` derive 安全（所有字段 trivially pod）。

### 2.3 `BodyState`（SoA）

```rust
// AoS 形式（冷路径：配置 / 错误）
pub struct BodySpec {
    pub shape: ShapeDesc,
    pub initial_transform: Transform,
    pub profile: PhysicsProfile,
    pub is_static: bool,
}

// SoA 形式（热路径：step 内）
pub struct BodyStateSoA {
    pub positions: Vec<Vec3>,     // 位置
    pub velocities: Vec<Vec3>,     // 线速度
    pub angular_velocities: Vec<Vec3>,  // 角速度
    pub orientations: Vec<Quat>,   // 姿态
    pub forces: Vec<Vec3>,         // 累积力（每帧重置）
    pub torques: Vec<Vec3>,        // 累积力矩
    pub sleep_states: Vec<SleepState>,  // Sleep 状态
    pub indices: Vec<u32>,         // profile 索引（profile 单独存储）
}
```

- 字段独立 Vec，cache 友好；
- 跨字段访问（位置 + 速度）通过 index 对齐；
- SIMD 路径：每个 Vec 单独 SIMD 化（`for pos in positions.iter() { ... }`）。

### 2.4 `ConstraintRow`（核心数据）

```rust
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ConstraintRow {
    pub body_a: BodyHandle,        // 0-7
    pub body_b: BodyHandle,        // 8-15
    pub jacobian: [f32; 12],       // 16-63（4 个 3D 向量）
    pub bias: f32,                  // 64
    pub compliance: f32,            // 68
    pub lambda: f32,                // 72（累积冲量，per-row）
    pub lower_limit: f32,           // 76
    pub upper_limit: f32,           // 80
    pub constraint_type: ConstraintType,  // 84 (u8)
    pub _padding: [u8; 3],         // 85-87
}
// size = 88, align = 4
```

- POD 友好；
- `lambda` 累积在 row 内部（避免 per-step 内存分配）；
- 详见 `17_detailed_design.md` §5。

### 2.5 `ContactManifold` / `ContactPoint`

```rust
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ContactPoint {
    pub position_a: Vec3,        // A 上的接触点
    pub position_b: Vec3,        // B 上的接触点
    pub normal: Vec3,            // 接触法线
    pub penetration: f32,        // 穿透深度
    pub feature_id: FeatureId,    // 特征标识
    // total: 12 + 12 + 12 + 4 + 4 = 44 bytes
}
// size = 44, align = 4

#[derive(Clone, Debug)]
pub struct ContactManifold {
    pub points: SmallVec<[ContactPoint; 4]>,  // 最多 4 个点
    pub normal: Vec3,
    pub friction: f32,
    pub restitution: f32,
    pub warmstart_impulse: Vec3,  // warm-start
}
```

- `ContactPoint` POD；
- `ContactManifold` 用 `SmallVec<[T; 4]>` 避免堆分配（典型 1-4 个点）；
- `SmallVec` **不** 跨 FFI 边界（FFI 用裸 slice + 长度）。

### 2.6 `Transform` / `Vec3` / `Quat` / `Mat3`

```rust
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
// size = 12, align = 4

#[derive(Clone, Copy, Debug)]
#[repr(C, align(16))]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
// size = 16, align = 16（SIMD 友好）

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Transform {
    pub translation: Vec3,    // 0
    pub rotation: Quat,       // 12（padding 至 16）
}
// size = 28, align = 16
// 注意：rotation 前有 4 字节 padding

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Mat3 {
    pub m: [Vec3; 3],  // 行优先，3 行 3 列
}
// size = 36, align = 4
```

- `Vec3` 12 字节（无 padding）；
- `Quat` 16 字节（自然 align 16 适合 SIMD）；
- `Transform` 内 `rotation` 起始 16 字节对齐（padding 后）；
- 矩阵行优先（C 风格）。

## 3. 关键类型的 SoA / AoS 选择表

| 类型 | 访问模式 | 推荐布局 | 理由 |
|---|---|---|---|
| `BodyState` 字段（pos / vel / force） | 热路径批量处理 | **SoA** | SIMD、cache 友好 |
| `PhysicsProfile` | 配置、读写不频繁 | **AoS**（POD） | 简单、跨 FFI |
| `ConstraintRow` | 求解器内循环 | **AoS**（POD）+ 数组 | lambda 内联、SIMD 化 |
| `ContactPoint` | manifold 内 1-4 个 | **AoS**（POD） | 数量小、整体访问 |
| `BodyHandle` | 散列 / 比较 | **AoS**（POD） | 8 字节，整体使用 |
| 关节 / solver 临时状态 | per-step 局部 | 栈分配（不持久） | 无需 SoA |
| Solver scratch | per-step 大数组 | **SoA** | SIMD 化 |
| 形状描述 (`ShapeDesc`) | 配置 | **AoS**（enum） | tag + union 即可 |
| Island 数据 | per-island 处理 | **SoA**（per island SoA） | 平衡 |

## 4. GPU 上传性考虑

（详见 `25_gpu_backend_detailed_design.md`）

- 核心类型应可直接上传到 GPU buffer（`bytemuck::Pod` + 紧凑）；
- `BodyStateSoA` 天然适合 GPU（独立 buffer）；
- `ConstraintRow[]` 可作为 compute shader 输入；
- 不要在 hot path struct 中包含 `Vec` / `String` / `Box`（GPU 不可表达）。

## 5. 验证测试

```rust
#[test]
fn test_layouts() {
    use std::mem::{size_of, align_of};
    
    assert_eq!(size_of::<BodyHandle>(), 8);
    assert_eq!(align_of::<BodyHandle>(), 4);
    
    assert_eq!(size_of::<PhysicsProfile>(), 80);
    assert_eq!(align_of::<PhysicsProfile>(), 4);
    
    assert_eq!(size_of::<ContactPoint>(), 44);
    
    assert_eq!(size_of::<Transform>(), 28);
    assert_eq!(align_of::<Transform>(), 16);
    
    // bytemuck::Pod 检查
    fn assert_pod<T: bytemuck::Pod>() {}
    assert_pod::<BodyHandle>();
    assert_pod::<PhysicsProfile>();
    assert_pod::<ConstraintRow>();
    assert_pod::<ContactPoint>();
    assert_pod::<Vec3>();
    assert_pod::<Quat>();
    assert_pod::<Transform>();
    assert_pod::<Mat3>();
}
```

CI 必须跑此测试，确保布局不漂移。

## 6. 变更管理

- 任何 `#[repr(C)]` 类型的字段变化 → **Breaking Change**（依 `28_workflow.md` §11.32 + `42_change_request_form.md`）；
- 字段顺序变化可能 ABI 兼容但内存布局不兼容（按 cbindgen 输出哈希判断）；
- 重大版本号（`v1.0.0`）后**禁止** `#[repr(C)]` 字段变化。

## 7. 关联

- `GVPE-DOC-17` §1（核心 struct 定义）
- `GVPE-DOC-26` §18.5（SIMD 策略）
- `GVPE-DOC-25`（GPU 上传性）
- `57_coding_standards.md`（unsafe 政策）
- `55_error_code_catalog.md`（错误类型布局）
- `28_workflow.md` §10.4 步 44/46

## 8. 审批

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 | — | | |
| 评审 | | | |
| 批准 | | | |
