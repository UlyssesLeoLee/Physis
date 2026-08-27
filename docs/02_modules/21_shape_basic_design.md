# GVPE — Basic Shapes Detailed Design（基礎形狀詳細設計）

> **草案,待 DDD Review**
>
> 本文档由 Mavis worker agent（per DEC-008）**新建**——基础 shape 模块（`gvpe-shape`）在仓库中**无既有 design doc**。
> `docs/02_modules/20_shape_advanced_design.md` 是**高级 shape**（Capsule / ConvexHull / TriangleMesh / Heightfield / Compound）及其 GJK/EPA 设计，
> **不**覆盖 MVP 基础形状（Sphere / Box3 / Plane）的细节。本文填补该基线缺位。
>
> **本文件状态**：草案（draft）。未经 DDD Review,实施后可能根据 DDD 反馈调整。

## 0. 文档元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-21 |
| 文档类型 | 詳細設計書 |
| 版本 | v0.1（草案） |
| 状态 | **Draft,待 DDD Review** |
| 适用阶段 | MVP |
| 关联系统 | GVPE / gvpe-shape, gvpe-collision, gvpe-ffi |
| 上游文档（输入基线） | `06_collision_design.md` §6.2（MVP 形状子集）, `17_detailed_design.md` §3 |
| 下游文档（被消费于） | `20_shape_advanced_design.md` §6.1（GJK 资产接口）, `10_ffi_design.md`（FFI 暴露） |
| 编写者 | Mavis 接手 agent per DEC-008（worker session mvs_927da1d9618a48158f89cdc0a809eeaf） |
| 审批者 | —（草案阶段,DDD Review 后填入） |

## 1. 修订历史

| 版本 | 日期 | 修改者 | 修改内容 |
|---|---|---|---|
| v0.1 | 2026-08-27 | Mavis 接手 agent per DEC-008 | 初稿（草案）— 填补仓库基线缺位 |

## 2. 文档目的

声明 `gvpe-shape` crate 的**基础形状**详细设计：Sphere / Box3 / Capsule / Plane / ConvexHull 5 种形状的
数据结构、`Shape` trait + `ShapeHandle`（`Arc<dyn Shape>`）共享机制、局部坐标 AABB 计算、与 gvpe-core /
gvpe-collision / gvpe-ffi 的接口边界。

**与 20 号文書的关系**：20 号文書 §6.1 把这些形状包含在 `ShapeDesc` enum 扩展里,但仅**作为 GJK 消费的资产**
描述；本文填补"这些形状作为**基础资产**的存储、共享、AABB 缓存"的细节,两者覆盖范围不同:

- 20 号文書：GJK/EPA 派发、资产（顶点/索引）的 Arc 共享策略、热路径纪律
- 21 号文書（本文）：形状数据布局、`Shape` trait 设计、`ShapeHandle` 句柄语义、局部 AABB 计算、构造校验

## 3. 适用范围

- 适用于 MVP 阶段 `gvpe-shape` crate 的 5 种基础形状
- **不适用于** GJK/EPA 算法（20 号文書 §20.2/§20.3 范围）
- **不适用于** TriangleMesh / Heightfield / Compound（20 号文書 §20.1 范围）
- **不适用于** 凸包构造算法（quickhull / incremental 等,见 §已知缺口）

## 4. 术语定义

| 术语 | 定义 |
|---|---|
| 形状（Shape） | 一个碰撞体的几何描述（如球、盒、胶囊、平面、凸包） |
| 形状资产 | 形状的不可变数据（半径 / 半尺寸 / 顶点表等）,通过 `Arc` 共享 |
| 局部 AABB | 形状在**自身局部坐标系**下的轴对齐包围盒（不施加 transform） |
| 世界 AABB | 局部 AABB 经 `Transform` 平移+旋转后的结果（gvpe-collision 关注） |
| `ShapeHandle` | `Arc<dyn Shape>` 共享句柄,克隆廉价 |
| MVP 形状子集 | Sphere / Box3 / Capsule / Plane / ConvexHull（本文范围） |
| 高级形状 | TriangleMesh / Heightfield / Compound（20 号文書范围） |

## 5. 模块详细设计

### 5.1 形状数据结构

`gvpe-shape` 暴露 5 个具名结构体（无 enum 包装——MVP 阶段不与 `gvpe-core::ShapeDesc` 合并）：

```rust
pub struct Sphere    { pub radius: f32 }
pub struct Box3      { pub half_extents: [f32; 3] }                 // 名字带 3 后缀避开 `box` 关键字
pub struct Capsule   { pub radius: f32, pub half_height: f32 }     // 轴向 = +Y
pub struct Plane     { pub normal: Vec3, pub offset: f32 }         // 无限半空间
pub struct ConvexHull { pub points: Arc<[Vec3]> }                  // 共享顶点资产
```

**为什么不用 enum 包装？**
- `Shape` trait 已经是 vtable 分派——再加一层 enum 是冗余
- enum tag 8 字节 vs `Arc<dyn Shape>` 8 字节——同样大小,但 trait object 已含 vtable
- 未来高级形状（20 号文書）直接新增结构体 + impl `Shape`,无需改 enum

### 5.2 `Shape` trait

```rust
pub trait Shape: Debug + Send + Sync {
    fn shape_type(&self) -> ShapeType;
    fn local_aabb(&self) -> Aabb;
}
```

**设计取舍**：

- **只暴露局部 AABB**（不暴露"世界 AABB"）——后者需要 `Transform`,这是 gvpe-core / gvpe-collision 的关注,
  本 crate 故意不依赖 gvpe-core。
- **要求 `Send + Sync`**——多线程 broad phase 共享同一 `ShapeHandle`。
- **AABB 实现应缓存**——构造时算一次（boxed slice 一次遍历）,后续调用零成本。
  ConvexHull 实际是惰性（每次 `local_aabb()` 都调 `Aabb::from_points`）——已知缺口。

### 5.3 `ShapeType` enum

```rust
#[repr(u8)]
pub enum ShapeType {
    Sphere = 0,
    Box3 = 1,
    Capsule = 2,
    Plane = 3,
    ConvexHull = 4,
}
```

- `#[repr(u8)]` 便于 FFI 边界直接 `transmute`
- 派发表用：`gvpe-collision::narrow_phase` 按 `(a.shape_type(), b.shape_type())` 选择算法
- `Copy + Eq + Hash` 便于作 `HashMap` key

### 5.4 `ShapeHandle`

```rust
pub struct ShapeHandle(pub(crate) Arc<dyn Shape>);
```

- **克隆廉价**：原子 refcount bump,不复制形状数据
- **多 body 共享同一资产**：assimp / glTF 加载器解析一次 `Arc<ConvexHull>`,所有引用 body 共享
- **资源释放自动**：最后一个引用 drop 时,资产自动释放
- `Debug` 输出包含 `shape_type` + `strong_count`,便于 assert "资产共享"
- `strong_count()` 公开,主要用于**测试**断言多 body 共享——不应作为业务逻辑（weak ref / 显式 drop 影响计数）

### 5.5 局部 AABB 计算

| 形状 | 局部 AABB | 备注 |
|---|---|---|
| `Sphere { radius: r }` | `[-r,-r,-r] × [+r,+r,+r]` | 球对称,旋转无影响 |
| `Box3 { half_extents: [hx,hy,hz] }` | `[-hx,-hy,-hz] × [+hx,+hy,+hz]` | OBB 需 gvpe-collision 算 8 顶点 |
| `Capsule { radius, half_height }` | `[-r,-(h+r),-r] × [+r,+(h+r),+r]` | 轴向 = `+Y` |
| `Plane { .. }` | `[-1e6,-1e6,-1e6] × [+1e6,+1e6,+1e6]` | **保守立方体**——见 §5.6 |
| `ConvexHull { points }` | `Aabb::from_points(points)` | len >= 4（构造校验保证） |

### 5.6 Plane 的特殊处理

`Plane` 是无限半空间,**没有有限 AABB**。MVP 工程做法：

- 返回 `[-1e6, +1e6]` 立方体作为保守 AABB
- 优点：broad phase 不会因为 AABB 退化而漏检（保守 = 不漏报）
- 缺点：broad phase 会引入虚假候选对；narrow phase 用 SAT 派发过滤（gvpe-collision 职责）
- 未来改进：用 `Option<Aabb>` 或单独的 `InfiniteShape` 接口——见 §已知缺口

### 5.7 ConvexHull 构造校验

`ConvexHull::new(Arc<[Vec3]>) -> Result<Self, ConvexError>`：

- 点数 < 4 → `Err(ConvexError::TooFewPoints(n))`（凸包最少 4 个非共面点）
- **MVP 不做完整凸性校验**（半空间交集 / QHull 算法）——超出范围,见 §已知缺口
- 点数 >= 4 → `Ok(Self { points })`

## 6. 关联需求

| 需求 ID | 中文描述 | 满足位置 |
|---|---|---|
| GVPE-FR-002 | 刚体动力学求解（完整形状集） | 基础 5 种形状实现 |
| `06_collision_design.md` §6.2 | MVP 子集：Sphere / Box3 / Plane | §5.1 数据结构 |
| `17_detailed_design.md` §3 | 碰撞基础 | §5.2 trait 设计 |
| `00_vision.md` §0.5 | 物理引擎完备性 | §5.5 AABB 计算 |
| 20 号文書 §6.1 | GJK 消费资产 | §5.4 `ShapeHandle` Arc 共享 |

## 7. 关联文档

- **上游**：`06_collision_design.md` §6.2 / `17_detailed_design.md` §3
- **下游**：`20_shape_advanced_design.md`（GJK 消费本文档的资产）、`10_ffi_design.md`（FFI 暴露 `ShapeHandle`）
- **平行**：`gvpe-core::ShapeDesc`（占位 enum,待 gvpe-shape 稳定后反向依赖）

## 8. 性能考量

- AABB 缓存：构造时一次计算（box/box/sphere/capsule/plane 是 O(1),convex 是 O(n)）
- `ShapeHandle::clone` 是原子 refcount bump,非热路径（形状不逐帧重建）
- `ShapeHandle::as_shape` 是 `&*self.0` 解引用,零成本
- 凸包 AABB 是惰性计算（每次 `local_aabb()` 重新调用 `Aabb::from_points`）——见 §已知缺口
- ConvexHull 资产 `Arc<[Vec3]>` 的 clone 本身不在热路径上（形状不逐帧变化）

## 9. 错误处理

- `ConvexHull::new` 点数 < 4 → `ConvexError::TooFewPoints(usize)`
- 其它构造（`Sphere::new` / `Box3::new` / `Capsule::new` / `Plane::new`）永不失败
- `Plane` 不归一化 `normal`——调用方保证（与 gvpe-core `ShapeDesc::Plane` 一致）
- `Shape::local_aabb` 对所有形状保证有限（非 NaN/Inf）——测试覆盖

## 10. 测试考量

测试数量 ≥ 9,覆盖：
- 5 种形状的局部 AABB 正确性（1 个/形状,共 5 个）
- `ConvexHull` 点数校验（< 4 拒收,>= 4 通过）
- `ShapeHandle` Arc 共享语义（refcount + 资产共享）
- `ShapeType` tag 一致性
- 全部形状 AABB 有限 + 含原点

## 11. 审批签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 | Mavis 接手 agent per DEC-008 | （代签） | 2026-08-27 |
| 校对 | — | — | — |
| 审批 | — | — | — |

---

## 12. 已知缺口（DDD Review 必查）

1. **完整凸包构造算法未实现**：MVP `ConvexHull::new` 仅校验点数 ≥ 4,**不**做完整凸性校验（半空间交集 / QHull / quickhull 等）。
   实际工程中应使用 `quickhull3` / `chull` crate 或自研 — 但本 crate 故意不引入依赖,留给后续工作。
2. **Plane 局部 AABB 是保守立方体**：可能引入 broad phase 虚假候选对。未来可改用 `Option<Aabb>` 或单独接口。
3. **ConvexHull AABB 惰性计算**：`local_aabb()` 每次都调 `Aabb::from_points(points)` 重新算。
   可优化为构造时一次性缓存（`OnceCell<Aabb>`）——见 v0.8+ TODO。
4. **凸包算法选择未定**：quickhull vs incremental（2D 增量到 3D）vs gift wrapping —— 见 §13 选型。
5. **形状集是否完整**：sphere/box/capsule/plane/convex 之外是否需要 cylinder / heightfield——
   cylinder 在 20 号文書**未列出**,heightfield 已划入 20 号文書的高级形状（不在本 crate 范围）。
6. **Cylinder 缺位**：`20_shape_advanced_design.md` 6.1 列举的 advanced 形状**不含** cylinder。
   一些引擎（Bullet / PhysX）有 cylinder——本 MVP 不实现,后续按游戏需求补。

## 13. 凸包算法选型（DDD Review 议题）

| 算法 | 时间复杂度 | 数值稳定性 | 依赖 |
|---|---|---|---|
| quickhull3 | 平均 O(n log n),最坏 O(n²) | 中等（依赖浮点精度） | 第三方 crate |
| incremental | 平均 O(n²) | 高 | 无（自研） |
| gift wrapping | O(nf)（f = 面数） | 高 | 无（自研） |

**MVP 决策**：**不**实现凸包算法（只校验 + 存储）。上游负责解析+构造凸包,本 crate 假定输入已凸。
后续如需自研:incremental（数值稳定性最优,代价是 O(n²) 平均）作为第一选项。

## 14. 21 vs 20 文档覆盖关系 / 重叠点

| 主题 | 21（本文） | 20 |
|---|---|---|
| Sphere / Box3 / Plane 数据结构 | ✅ §5.1 | 提及（`ShapeDesc` 扩展） |
| Capsule 数据结构 | ✅ §5.1 | ✅ §6.1 |
| ConvexHull 数据结构 | ✅ §5.1 | ✅ §6.1 |
| `Shape` trait | ✅ §5.2 | 提及（`SupportFn` 是不同抽象） |
| `ShapeHandle` Arc 共享 | ✅ §5.4 | ✅ §6.1 资产式分配策略 |
| 局部 AABB 计算 | ✅ §5.5 | **不**涉及 |
| ConvexHull 构造校验 | ✅ §5.7 | **不**涉及 |
| GJK/EPA 算法 | **不**涉及 | ✅ §20.2/§20.3 |
| TriangleMesh / Heightfield / Compound | **不**涉及 | ✅ §20.1 |
| 派发表（narrow phase dispatch） | **不**涉及 | ✅ §20.4 |
| `SupportFn` trait | **不**涉及 | ✅ §6.2 |

**重叠点**：Capsule / ConvexHull 数据结构、`ShapeDesc` 扩展位置、Arc 共享策略。
**无重叠**：AABB 计算、构造校验（21 独有）；GJK/EPA、派发表（20 独有）。

## 15. 与 gvpe-core::ShapeDesc 的关系

`gvpe-core::ShapeDesc`（`crates/gvpe-core/src/descriptor.rs:15`）当前是**占位 enum**（仅 MVP 三种）：

```rust
pub enum ShapeDesc {
    Sphere { radius: f32 },
    Box3 { half_extents: [f32; 3] },
    Plane { normal: [f32; 3], offset: f32 },
}
```

后续工作（不在本文档范围）：

1. gvpe-core 反向依赖 gvpe-shape,把 `ShapeDesc` 替换为 `ShapeHandle`（per-body 资产引用）
2. `BodySpec::shape` 字段从 `ShapeDesc`（per-body 拷贝）改为 `ShapeHandle`（共享）
3. `RuntimeDescriptor::bodies` 中的 `shape` 字段也跟随改
4. `RuntimeDescriptor::validate` 校验每个 body 持有有效 `ShapeHandle`

**本 crate 不做此替换**——属于跨 crate 重构,超出单次 worktree 范围。
