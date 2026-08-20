# GVPE — Advanced Shapes & GJK/EPA Detailed Design（詳細設計書）

## 0. 文档元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-20 |
| 文档类型 | 詳細設計書 |
| 版本 | v0.2（标准化重写版） |
| 状态 | Draft |
| 适用阶段 | MVP（设计就绪，运行时不在 MVP 范围） |
| 关联系统 | GVPE / gvpe-shape, gvpe-collision, gvpe-memory |
| 上游文档（输入基线） | `06_collision_design.md` §6.2/§6.3, `17_detailed_design.md` §3, `08_memory_design.md` §8.1, `15_testing_strategy.md`, `00_vision.md` §0.5 |
| 下游文档（被消费于） | `10_ffi_design.md` |
| 编写者 | — |
| 审批者 | — |

## 1. 修订历史

| 版本 | 日期 | 修改者 | 修改内容 |
|---|---|---|---|
| v0.1 | — | — | 初稿（原始版本） |
| v0.2 | 2026-08-19 | 标准化 pass | 转为 IPA 詳細設計書 格式；叙事中文化；保留全部技术内容；补充文档元数据、修订历史、术语定义、关联文档、审批签字等标准章节 |

## 2. 文档目的

声明完备的物理引擎需要完整的形状集——游戏或仿真真正使用的不仅是 MVP 的三种原语，还包括 Capsule、Convex Hull、Triangle Mesh、Heightfield、Compound。本文给出这些高级形状的详细设计：形状描述扩展、GJK/EPA 算法、窄阶段派发表、宽阶段隐含影响、测试夹具。

## 3. 适用范围

- 适用于 MVP 之后阶段的高级形状（Capsule / ConvexHull / TriangleMesh / Heightfield / Compound）及其 GJK/EPA 算法的详细设计
- 不适用于 GPGPU 端的形状查询加速（属于后续 Phase）
- 不适用于连续形状的"非凸"广义表示（如 SDF 体素）——这些属于更远期 Phases

## 4. 术语定义

| 术语 | 定义 |
|---|---|
| GJK（Gilbert-Johnson-Keerthi） | 凸-凸距离/相交测试算法 |
| EPA（Expanding Polytope Algorithm） | 在 GJK 输出的单纯形上扩展多面体以求解穿透深度 |
| 胶囊（Capsule） | 线段加半球端的形状，常用于角色碰撞体 |
| 凸包（Convex Hull） | 由点集最小凸包络定义的形状 |
| 三角网格（Triangle Mesh） | 由顶点与三角形索引构成的网格，用于静态地形/道具 |
| 高度场（Heightfield） | 二维网格上的标量高度，常用于地形 |
| 复合形状（Compound） | 多个子形状按局部变换组合的形状 |
| 支持函数（Support Function） | 给定方向返回凸体最远点的函数 |
| 单纯形（Simplex） | GJK 中维护的至多 4 点的凸包 |
| 多面体（Polytope） | EPA 中由三角形面构成的凸多面体 |
| 静态 AABB | 永不重排序的 AABB（SAP 优化机会） |
| 接触流形（Contact Manifold） | 接触点集合（见 `17_detailed_design.md` §5.2） |
| 包围球（Bounding Sphere） | 形状的最小球形包络，CCD 与保守估计常用 |
| 资产引用计数 | 通过 `Arc` 共享不可变资产（顶点/索引等） |
| 派发表（Dispatch Table） | 形状组合到具体碰撞算法的选择表 |

## 5. 模块详细设计

### 5.1 形状描述扩展模块

`ShapeDesc` 枚举扩展新增 5 个变体（Capsule / ConvexHull / TriangleMesh / Heightfield / Compound），使用 `Arc` 共享不可变顶点/索引资产。详见正文 §20.1。

### 5.2 GJK 模块

凸-凸距离/相交测试算法，作用于所有凸 `ShapeDesc` 变体（Sphere/Box/Capsule/ConvexHull）。详见正文 §20.2。

### 5.3 EPA 模块

在 GJK 输出单纯形上扩展多面体，求解穿透深度并构建 `ContactManifold`。详见正文 §20.3。

### 5.4 窄阶段派发表模块

按形状组合选择 SAT / 解析 / GJK+EPA / 网格 / 高度场 / 复合派生的派发表。详见正文 §20.4。

### 5.5 宽阶段隐含模块

静态 AABB 的 SAP 优化机会说明。详见正文 §20.5。

## 6. 类与数据结构

### 6.1 形状描述扩展

```rust
enum ShapeDesc {
    Sphere { radius: f32 },
    Box3 { half_extents: [f32; 3] },
    Plane { normal: [f32; 3], offset: f32 },
    Capsule { radius: f32, half_height: f32 },                       // 新增
    ConvexHull { points: Arc<[[f32; 3]]> },                           // 新增, 共享/引用计数（资产式）
    TriangleMesh { vertices: Arc<[[f32; 3]]>, indices: Arc<[u32]>,    // 新增, 仅静态 body
                    bvh: Arc<MeshBvh> },
    Heightfield { heights: Arc<[f32]>, width: u32, depth: u32,        // 新增, 仅静态 body
                   cell_size: f32 },
    Compound { children: Vec<(Transform, Box<ShapeDesc>)> },          // 新增, 递归
}
```

较重的变体上使用 `Arc` 避免多 body 共享资产时的复制——这是与 `08_memory_design.md` §8.1 热路径纪律一致的分配策略：`Arc` 的 clone 本身不在热路径上（形状不会逐帧变化），热路径上仅 *针对* 它们的碰撞查询。

### 6.2 支持函数抽象

```rust
trait SupportFn { fn support(&self, direction: [f32; 3]) -> [f32; 3]; }
```

所有凸 `ShapeDesc` 变体（Sphere/Box/Capsule/ConvexHull）都实现 `SupportFn`——GJK 本身与形状无关，这正是新增凸原语时仅需新增 `SupportFn` 实现而不修改本函数的原因（与 `18_joints_ccd_design.md` §18.1 中"关节行 vs 求解器"相同的"算法不针对形状种类特殊化"纪律）。

## 7. 算法详解

### 7.1 GJK（Gilbert-Johnson-Keerthi）—— 凸-凸距离/相交

```rust
fn gjk_intersect(a: &SupportFn, b: &SupportFn) -> GjkResult {
    let mut simplex = Simplex::new();
    let mut direction = initial_direction(a, b);
    loop {
        let point = minkowski_support(a, b, direction);
        if dot(point, direction) < 0.0 { return GjkResult::NoOverlap; }
        simplex.push(point);
        match simplex.do_simplex(&mut direction) {
            SimplexResult::ContainsOrigin => return GjkResult::Overlap(simplex),
            SimplexResult::Continue => continue,
        }
        if simplex.iterations() > GJK_MAX_ITERATIONS { return GjkResult::NoOverlap; }  // 保守性 bail-out
    }
}
```

所有凸 `ShapeDesc` 变体（Sphere/Box/Capsule/ConvexHull）都实现 `SupportFn`——GJK 本身与形状无关，这正是新增凸原语时仅需新增 `SupportFn` 实现而不修改本函数的原因。

### 7.2 EPA（Expanding Polytope Algorithm）—— 穿透深度（基于 GJK 重叠单纯形）

```rust
fn epa_penetration(simplex: Simplex, a: &SupportFn, b: &SupportFn) -> ContactManifold {
    let mut polytope = Polytope::from_simplex(simplex);
    loop {
        let (closest_face, distance) = polytope.closest_face_to_origin();
        let support_point = minkowski_support(a, b, closest_face.normal);
        let expansion = dot(support_point, closest_face.normal) - distance;
        if expansion < EPA_TOLERANCE { return build_manifold_from_face(closest_face, distance); }
        polytope.expand(support_point);   // 重三角化, 移除新点可见的面
    }
}
```

GJK+EPA 联合替代 SAT（`17号文書` §3.3）专门用于 SAT 无法处理的形状对（任何涉及 Capsule 或 ConvexHull 的组合）——SAT 在 Box-Box/Box-Plane/Sphere-Box 路径上保持不变（这些情况下更廉价），依 §20.4 的派发表。

### 7.3 窄阶段派发（扩展 `17号文書` §3.3，非替换）

```rust
fn narrow_phase(a: &ShapeDesc, xf_a: &Transform, b: &ShapeDesc, xf_b: &Transform) -> Option<ContactManifold> {
    match (a, b) {
        (Box3{..}|Sphere{..}|Plane{..}, Box3{..}|Sphere{..}|Plane{..}) => narrow_phase_sat(a, xf_a, b, xf_b),
        (Sphere{..}, Sphere{..}) => sphere_sphere_analytic(a, xf_a, b, xf_b),   // 最廉价的精确情形, 无需 SAT
        (TriangleMesh{..}, _) | (_, TriangleMesh{..}) => mesh_vs_convex(a, xf_a, b, xf_b),   // BVH 加速, 按三角形 GJK/EPA
        (Heightfield{..}, _) | (_, Heightfield{..}) => heightfield_vs_convex(a, xf_a, b, xf_b),   // 网格单元查找, 按单元 GJK/EPA
        (Compound{children: ca, ..}, _) => compound_vs_other(ca, xf_a, b, xf_b),   // 按子形状递归, 联合流形
        (_, Compound{..}) => narrow_phase(b, xf_b, a, xf_a).map(flip_manifold),
        _ => gjk_epa_convex_pair(a, xf_a, b, xf_b),   // Capsule/ConvexHull 组合
    }
}
```

SAT 仍是 MVP 形状三件套的快速路径（更廉价且已实现，`17号文書` §3.3）——GJK/EPA 是新增而非替代，正如 `06_collision_design.md` §6.3 已承诺的（「GJK/EPA：为凸包支持保留（post-MVP）」）。

### 7.4 宽阶段隐含影响

`Arc<[[f32;3]]>` 后端的形状（mesh/heightfield）几乎都是 **静态** body（`06号文書` 不强制要求，但绝大多数情况如此）——宽阶段（`17号文書` §3.2 SAP）将静态 AABB 视为永不重排序，这正是 SAP 的 insertion sort 在零速度条目上的行为（无需设计变更，此处仅作显式说明以使交互明确而非偶然）。

## 8. 错误处理

- GJK 达到 `GJK_MAX_ITERATIONS`：保守性 bail-out，返回 `NoOverlap`（不视作错误，宁可漏检也不误报穿透）
- EPA 达到最大扩展次数：返回退化流形（法线为最近面的法线，穿透深度取下界）
- `MeshBvh` 构建失败（退化网格）：返回 `InitError` 子类型
- 静态 vs 动态误用：将 `TriangleMesh`/`Heightfield` 附给动态 body 在 `GvpeContext::new` 阶段通过 `is_static` 校验拦截
- 资产 `Arc` 共享：所有权模型保证不会出现 use-after-free

## 9. 性能考量

- SAT 仍是 MVP 形状的快路径
- 球-球使用解析公式，避免任何迭代
- GJK 收敛性：典型 3–7 次迭代，使用 `GJK_MAX_ITERATIONS` 兜底
- EPA 终止条件：`EPA_TOLERANCE` 控制精度
- Mesh/Heightfield 通过 BVH/网格单元降低常数因子
- Compound 按子形状独立查询后合并流形
- `Arc` 后端的资产不在热路径上 clone
- 性能预算：见 `14_performance_budget.md`

## 10. 测试考量

- GJK/EPA 与 SAT 在 Box-Box case 上交叉验证（法线、深度在容差内一致）
- Capsule 在高度场上稳定静止，无抖动、无隧道
- Compound 形状（如两 box 组成 L 形）与单 box 正确碰撞
- 网格 vs 凸形状的法线方向正确性
- 高度场 vs 球的静摩擦行为符合解析预期
- 详细测试夹具见正文 §20.6

## 11. 关联需求

| 需求 ID | 中文描述 | 满足位置 |
|---|---|---|
| GVPE-FR-002 | 刚体动力学求解（完整形状集） | §20.1–§20.5 |
| `00_vision.md` §0.5 | 物理引擎完备性需完整形状 | 全文 |
| `06_collision_design.md` §6.2 | MVP 子集与预留项 | §20.1 |
| `06_collision_design.md` §6.3 | GJK/EPA 预留承诺 | §20.2–§20.4 |
| `08号文書` §8.1 | 热路径纪律（Arc 分配策略） | §20.1 |
| `15号文書` §15.6 | 测试夹具纪律 | §20.6 |

## 12. 关联文档

- 上游：`06_collision_design.md` §6.2/§6.3（MVP 形状子集与 GJK/EPA 预留）、`17_detailed_design.md` §3（碰撞基础）、`08_memory_design.md` §8.1（热路径纪律）、`15_testing_strategy.md`（测试纪律）、`00_vision.md` §0.5
- 下游：`10_ffi_design.md`（高级形状经 FFI 暴露）
- 平行：`18_joints_ccd_design.md`（共享"算法不特殊化形状种类"纪律）、`19_softbody_xpbd_design.md`（共享 ShapeDesc 扩展位置）

## 13. 审批签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 | — | — | — |
| 校对 | — | — | — |
| 审批 | — | — | — |

---

## 14. 正文

> Input baseline: `06_collision_design.md` §6.2 (MVP subset: Sphere/Box/Plane, rest reserved),
> `17_detailed_design.md` §3. A physics engine claiming completeness needs the full shape set a game
> or simulation actually uses — Capsule, Convex Hull, Triangle Mesh, Heightfield, Compound — not just
> the three MVP primitives.

## 20.1 形状描述扩展

```rust
enum ShapeDesc {
    Sphere { radius: f32 },
    Box3 { half_extents: [f32; 3] },
    Plane { normal: [f32; 3], offset: f32 },
    Capsule { radius: f32, half_height: f32 },                       // 新增
    ConvexHull { points: Arc<[[f32; 3]]> },                           // 新增, 共享/引用计数（资产式）
    TriangleMesh { vertices: Arc<[[f32; 3]]>, indices: Arc<[u32]>,    // 新增, 仅静态 body
                    bvh: Arc<MeshBvh> },
    Heightfield { heights: Arc<[f32]>, width: u32, depth: u32,        // 新增, 仅静态 body
                   cell_size: f32 },
    Compound { children: Vec<(Transform, Box<ShapeDesc>)> },          // 新增, 递归
}
```

较重的变体上使用 `Arc` 避免多 body 共享资产时的复制——这是与 `08_memory_design.md` §8.1 热路径纪律一致的分配策略：`Arc` 的 clone 本身不在热路径上（形状不会逐帧变化），热路径上仅 *针对* 它们的碰撞查询。

## 20.2 GJK（Gilbert-Johnson-Keerthi）—— 凸-凸距离/相交

```rust
fn gjk_intersect(a: &SupportFn, b: &SupportFn) -> GjkResult {
    let mut simplex = Simplex::new();
    let mut direction = initial_direction(a, b);
    loop {
        let point = minkowski_support(a, b, direction);
        if dot(point, direction) < 0.0 { return GjkResult::NoOverlap; }
        simplex.push(point);
        match simplex.do_simplex(&mut direction) {
            SimplexResult::ContainsOrigin => return GjkResult::Overlap(simplex),
            SimplexResult::Continue => continue,
        }
        if simplex.iterations() > GJK_MAX_ITERATIONS { return GjkResult::NoOverlap; }  // 保守性 bail-out
    }
}

trait SupportFn { fn support(&self, direction: [f32; 3]) -> [f32; 3]; }
```

所有凸 `ShapeDesc` 变体（Sphere/Box/Capsule/ConvexHull）都实现 `SupportFn`——GJK 本身与形状无关，这正是新增凸原语时仅需新增 `SupportFn` 实现而不修改本函数的原因（与 `18_joints_ccd_design.md` §18.1 中"关节行 vs 求解器"相同的"算法不针对形状种类特殊化"纪律）。

## 20.3 EPA（Expanding Polytope Algorithm）—— 穿透深度（基于 GJK 重叠单纯形）

```rust
fn epa_penetration(simplex: Simplex, a: &SupportFn, b: &SupportFn) -> ContactManifold {
    let mut polytope = Polytope::from_simplex(simplex);
    loop {
        let (closest_face, distance) = polytope.closest_face_to_origin();
        let support_point = minkowski_support(a, b, closest_face.normal);
        let expansion = dot(support_point, closest_face.normal) - distance;
        if expansion < EPA_TOLERANCE { return build_manifold_from_face(closest_face, distance); }
        polytope.expand(support_point);   // 重三角化, 移除新点可见的面
    }
}
```

GJK+EPA 联合替代 SAT（`17号文書` §3.3）专门用于 SAT 无法处理的形状对（任何涉及 Capsule 或 ConvexHull 的组合）——SAT 在 Box-Box/Box-Plane/Sphere-Box 路径上保持不变（这些情况下更廉价），依 §20.4 的派发表。

## 20.4 窄阶段派发（扩展 `17号文書` §3.3，非替换）

```rust
fn narrow_phase(a: &ShapeDesc, xf_a: &Transform, b: &ShapeDesc, xf_b: &Transform) -> Option<ContactManifold> {
    match (a, b) {
        (Box3{..}|Sphere{..}|Plane{..}, Box3{..}|Sphere{..}|Plane{..}) => narrow_phase_sat(a, xf_a, b, xf_b),
        (Sphere{..}, Sphere{..}) => sphere_sphere_analytic(a, xf_a, b, xf_b),   // 最廉价的精确情形, 无需 SAT
        (TriangleMesh{..}, _) | (_, TriangleMesh{..}) => mesh_vs_convex(a, xf_a, b, xf_b),   // BVH 加速, 按三角形 GJK/EPA
        (Heightfield{..}, _) | (_, Heightfield{..}) => heightfield_vs_convex(a, xf_a, b, xf_b),   // 网格单元查找, 按单元 GJK/EPA
        (Compound{children: ca, ..}, _) => compound_vs_other(ca, xf_a, b, xf_b),   // 按子形状递归, 联合流形
        (_, Compound{..}) => narrow_phase(b, xf_b, a, xf_a).map(flip_manifold),
        _ => gjk_epa_convex_pair(a, xf_a, b, xf_b),   // Capsule/ConvexHull 组合
    }
}
```

SAT 仍是 MVP 形状三件套的快速路径（更廉价且已实现，`17号文書` §3.3）——GJK/EPA 是新增而非替代，正如 `06_collision_design.md` §6.3 已承诺的（「GJK/EPA：为凸包支持保留（post-MVP）」）。

## 20.5 宽阶段隐含影响

`Arc<[[f32;3]]>` 后端的形状（mesh/heightfield）几乎都是 **静态** body（`06号文書` 不强制要求，但绝大多数情况如此）——宽阶段（`17号文書` §3.2 SAP）将静态 AABB 视为永不重排序，这正是 SAP 的 insertion sort 在零速度条目上的行为（无需设计变更，此处仅作显式说明以使交互明确而非偶然）。

## 20.6 欠 `15_testing_strategy.md` 的测试夹具

- GJK/EPA 与 SAT 在 Box-Box case 上交叉验证——法线与穿透深度在容差内一致（两个路径相互交叉验证）。
- 一个胶囊在高度场上稳定静止 N 步，无抖动、无隧道。
- 一个复合形状（如两 box 组成 L 形）与单 box 正确碰撞——验证 §20.4 中 `compound_vs_other` 的子流形联合。

Requirements satisfied: `01_requirements.md` GVPE-FR-002 (full shape set), `06_collision_design.md`
§6.2/§6.3 (both "reserved" lines now have a design), `00_vision.md` §0.5.
