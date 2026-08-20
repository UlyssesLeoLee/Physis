# 算法伪代码图谱（Algorithm Pseudocode Atlas）

> **用途**：GVPE 关键算法的权威伪代码登记——broad phase、narrow phase、SI 求解、island、scheduler 等。
> **对应工作流步骤**：45 ロジック設計、43 モジュール設計 → `28_workflow.md` §10.4 步 43/45。
> **关联**：`GVPE-DOC-17` §3-§8（详细算法）；`GVPE-DOC-06`（碰撞）；`GVPE-DOC-07`（求解器）；`GVPE-DOC-09`（并行）；`58_data_layout_atlas.md`（数据结构）。

## 0. 元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-59 |
| 文档类型 | 詳細設計書 |
| 版本 | v0.1 |
| 状态 | Draft |
| 适用阶段 | MVP / 实施期 |
| 上游文档 | `GVPE-DOC-17` §3-§8, `GVPE-DOC-06/07/09` |
| 下游文档 | 实施期 PR / `38_code_review_checklist.md` |

## 1. 总体原则

- **伪代码 vs 真实代码**：本文件用类 Python / 类 Rust 伪代码，**不**直接可执行；真实代码在 `17` 和 06/07/09 号详细设计文档 + crate 实现中；
- **数学符号优先**：尽量用数学表达而非代码；
- **复杂度标注**：每个关键算法标注时间 / 空间复杂度；
- **测试用例关联**：每个算法附主要测试场景。

## 2. Broad Phase — SAP（Sweep and Prune）

### 2.1 目的

给定 N 个 AABB，输出所有可能重叠的 body pair（candidate pairs）。

### 2.2 算法

```python
def broad_phase_sap(aabbs: list[Aabb], scratch: Arena) -> list[Pair]:
    # 1. 选轴：选 AABB 中心点方差最大的轴
    best_axis = argmax([variance([aabb.center[i] for aabb in aabbs]) for i in [0, 1, 2]])
    
    # 2. 索引列表：按 AABB 在 best_axis 的 min 值排序
    # 前帧顺序近似 → 用 insertion sort（O(n) amortized for coherent motion）
    indices = scratch.alloc_slice(range(len(aabbs)))
    insertion_sort_by_key(indices, key=lambda i: aabbs[i].min[best_axis])
    
    # 3. 扫描 + 收集 candidate pairs
    pairs = []
    active = []  # active AABB 索引
    for i in indices:
        # 移除 active 中 max[axis] < aabbs[i].min[axis] 的（不再重叠）
        active = [j for j in active if aabbs[j].max[best_axis] >= aabbs[i].min[best_axis]]
        # 对 active 中每个 j，若完整 AABB 重叠，加入 pairs
        for j in active:
            if aabbs_overlap(aabbs[i], aabbs[j]):
                pairs.append(Pair(min(i, j), max(i, j)))
        active.append(i)
    return pairs
```

### 2.3 复杂度

- **最好情况**（coherent motion）：O(n + k)，k = 输出对数；
- **最坏情况**（乱序 / 大量退化）：O(n²)；
- **空间**：O(n) for indices + active list。

### 2.4 退化策略

- 完全乱序场景：fallback 到完整 sort（O(n log n)）；
- 极端集中（所有 body 紧密）：退化为 O(n²)，需要上层报警。

## 3. Narrow Phase — SAT（Separating Axis Theorem）

### 3.1 目的

判定两 shape 是否相交，输出接触流形（normal + 穿透深度 + 接触点）。

### 3.2 算法（Box-Box）

```python
def narrow_phase_sat_box_box(a: OBB, b: OBB) -> Optional[Manifold]:
    # 1. 收集分离轴：面法线（6 个）+ 边外积（9 个）= 15
    axes = a.axes + b.axes  # 6 个面法线
    for ea in a.axes:        # 3 边
        for eb in b.axes:    # 3 边
            axes.append(cross(ea, eb))
    
    # 2. 找最小重叠轴
    min_overlap = +inf
    best_axis = None
    for axis in axes:
        if abs(dot(axis, axis)) < epsilon:  # 退化轴（平行边）
            continue
        proj_a = project(a, axis)
        proj_b = project(b, axis)
        overlap = min(proj_a.max, proj_b.max) - max(proj_a.min, proj_b.min)
        if overlap < 0:  # 分离轴
            return None
        if overlap < min_overlap:
            min_overlap = overlap
            best_axis = axis
    
    # 3. 沿 best_axis 构造流形
    normal = sign(dot(b.center - a.center, best_axis)) * best_axis
    return build_manifold_from_axis(a, b, normal, min_overlap)
```

### 3.3 复杂度

- **box-box**：O(1)（固定 15 轴 + 投影）；
- **sphere-box**：O(1)（3 轴）；
- **sphere-sphere**：O(1)（1 轴 = 中心连线）；
- **sphere-plane**：O(1)（1 轴 = 平面法线）。

## 4. Sequential Impulse 求解

### 4.1 目的

迭代求解接触 / 摩擦 / 关节约束，输出累积冲量。

### 4.2 主循环

```python
def solve_si(constraints: list[ConstraintRow], bodies: BodyStateSoA, iter: int):
    # 1. Warm-start（应用上一帧 lambda 累积）
    for row in constraints:
        apply_impulse(bodies, row, row.lambda)
    
    # 2. 主迭代
    for k in range(iter):
        for row in constraints:
            # 计算当前 lambda 增量
            delta_lambda = solve_single_constraint(row, bodies)
            # 累积到 row.lambda
            row.lambda += delta_lambda
            # 应用冲量
            apply_impulse(bodies, row, delta_lambda)
    
    # 3. 记录 warm-start lambda（供下一帧）
    # （已存储在 row.lambda）
```

### 4.3 单约束求解

```python
def solve_single_constraint(row: ConstraintRow, bodies: BodyStateSoA) -> f32:
    # 1. 计算相对速度（含角速度）
    v_rel = compute_relative_velocity(bodies, row)
    
    # 2. 含 bias 的目标速度（含 Baumgarte 稳定化）
    target_velocity = row.bias - v_rel
    
    # 3. 有效质量（Effective Mass）
    inv_m_a = 1.0 / bodies.mass[row.body_a]
    inv_m_b = 1.0 / bodies.mass[row.body_b]
    k = dot(row.jacobian, row.jacobian * inv_m_a) + dot(row.jacobian, row.jacobian * inv_m_b)
    # 简化为：k = J^T * M^-1 * J
    effective_mass = 1.0 / k
    
    # 4. 含 compliance 的 lambda 增量
    delta_lambda = (target_velocity - row.compliance * row.lambda) * effective_mass
    delta_lambda = clamp_to_limits(delta_lambda, row.lower_limit, row.upper_limit)
    
    return delta_lambda
```

### 4.4 复杂度

- **每约束 / 每迭代**：O(1)；
- **整体**：O(K * M)，K = 迭代次数，M = 约束数；
- 典型 K = 10, M = 10000 → 100K 次操作 / step。

## 5. 接触流形构建

```python
def build_manifold_from_axis(a: Shape, b: Shape, normal: Vec3, depth: f32) -> Manifold:
    # 1. 找 A 的支持点（沿 -normal 最远点）
    supp_a = support_point(a, -normal)
    # 2. 找 B 的支持点（沿 +normal 最远点）
    supp_b = support_point(b, normal)
    # 3. 构造接触点
    pos_a = supp_a - normal * (depth * 0.5)
    pos_b = supp_b + normal * (depth * 0.5)
    point = ContactPoint(pos_a, pos_b, normal, depth)
    return Manifold(points=[point], normal=normal, ...)
```

## 6. Island 构建（Union-Find）

```python
class IslandUF:
    parent: list[int]
    rank: list[int]
    
    def find(self, x: int) -> int:
        while self.parent[x] != x:
            self.parent[x] = self.parent[self.parent[x]]  # path compression
            x = self.parent[x]
        return x
    
    def union(self, x: int, y: int) -> bool:
        rx, ry = self.find(x), self.find(y)
        if rx == ry:
            return False
        if self.rank[rx] < self.rank[ry]:
            rx, ry = ry, rx
        self.parent[ry] = rx
        if self.rank[rx] == self.rank[ry]:
            self.rank[rx] += 1
        return True
```

### 6.1 流程

```python
def build_islands(bodies: list[Body], contact_pairs: list[Pair]) -> list[Island]:
    n = len(bodies)
    uf = IslandUF(n)
    # 1. 通过接触 / 关节 union
    for pair in contact_pairs:
        uf.union(pair.a, pair.b)
    # 2. 收集 group
    groups = {}  # root -> [body indices]
    for i in range(n):
        root = uf.find(i)
        groups.setdefault(root, []).append(i)
    # 3. 过滤小岛（1 个 body 且 sleeping）— 可选
    return [Island(members) for members in groups.values() if len(members) > 1 or not is_sleeping(members[0])]
```

### 6.2 复杂度

- **Union-Find with path compression + rank**：amortized O(α(n)) per op，α = inverse Ackermann，几乎常数；
- **总**：O(n + k * α(n))，k = 接触对数。

## 7. Sleep 状态机

```python
def update_sleep_state(body: Body, dt: f32, sleep_threshold: SleepThreshold) -> SleepState:
    if body.is_static:
        return SleepState::Static
    
    if body.sleep_state == SleepState::Active:
        # 计算动能
        ke = 0.5 * body.mass * dot(body.velocity, body.velocity) + \
             0.5 * dot(body.angular_velocity, body.inertia * body.angular_velocity)
        if ke < sleep_threshold.kinetic_energy_threshold and body.time_below > sleep_threshold.time_threshold:
            return SleepState::Sleeping
        body.time_below += dt if ke < sleep_threshold.kinetic_energy_threshold else 0
        return SleepState::Active
    
    elif body.sleep_state == SleepState::Sleeping:
        # 邻居唤醒检查
        if any_neighbor_active(body):
            return SleepState::Active
        return SleepState::Sleeping
```

## 8. Work-Stealing Scheduler

```python
class Scheduler:
    queues: list[deque[Job]]  # per-thread
    global_queue: deque[Job]
    
    def submit(self, job: Job):
        self.global_queue.append(job)
    
    def worker_loop(self, thread_id: int):
        while not self.shutdown:
            job = self.find_work(thread_id)
            if job is None:
                idle()
                continue
            job.execute()
    
    def find_work(self, thread_id: int) -> Optional[Job]:
        # 1. 自己的队列
        if self.queues[thread_id]:
            return self.queues[thread_id].pop()
        # 2. 全局队列
        if self.global_queue:
            return self.global_queue.popleft()
        # 3. 偷其他线程
        for other in permutation(range(len(self.queues))):
            if other == thread_id: continue
            if self.queues[other]:
                return self.queues[other].pop()  # steal from end
        return None
```

## 9. 帧主循环

```python
def step(runtime: Runtime, dt: f32) -> Result<(), RuntimeError>:
    # 1. 输入验证
    if dt <= 0 or dt > MAX_DT:
        return Err(RuntimeError::InvalidDt)
    
    # 2. Broad phase
    pairs = broad_phase_sap(runtime.aabbs, runtime.scratch)
    
    # 3. Narrow phase
    manifolds = narrow_phase_batch(pairs, runtime.shapes, runtime.scratch)
    
    # 4. 构建 ConstraintRow 列表
    constraints = build_constraint_rows(manifolds, runtime.profiles)
    
    # 5. 构建 island
    islands = build_islands(runtime.bodies, manifolds)
    
    # 6. 调度（per island 一个 job）
    for island in islands:
        scheduler.submit(solve_island_job(island, constraints, dt))
    
    # 7. 等待完成
    scheduler.wait_all()
    
    # 8. 积分
    for body in runtime.bodies:
        integrate(body, dt)
    
    # 9. Update sleep state
    update_sleep_states(runtime.bodies, dt)
    
    Ok(())
```

## 10. 性能关键路径总结

| 路径 | 复杂度 | 优化目标 |
|---|---|---|
| Broad phase | O(n) ~ O(n²) | 1000 body: < 1ms |
| Narrow phase | O(k) (k = pair 数) | 100 pair: < 1ms |
| Constraint build | O(k) | 100 constraint: < 100μs |
| SI solve | O(K * M) | 10000 constraint × 10 iter: < 8ms |
| Integrate | O(n) | 1000 body: < 100μs |
| Island build | O(n + k) | 1000 body: < 100μs |
| Sleep update | O(n) | 1000 body: < 50μs |
| **单 step 总计** | — | **1000 body: < 16ms（60Hz）** |

## 11. 关联

- `GVPE-DOC-17` §3-§8（详细算法）
- `GVPE-DOC-06`（碰撞详细）
- `GVPE-DOC-07`（求解器详细）
- `GVPE-DOC-09`（并行详细）
- `58_data_layout_atlas.md`（数据结构）
- `28_workflow.md` §10.4 步 43/45

## 12. 审批

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 | — | | |
| 评审 | | | |
| 批准 | | | |
