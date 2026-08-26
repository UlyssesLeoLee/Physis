//! 粗筛（broad phase）：Sweep-and-Prune（SAP）。
//!
//! 依据 `GVPE-DOC-06`（06_collision_design.md）§6.1：MVP 选 SAP（最低自研风险），
//! 接口按 §7.2 暴露 `fn broad_phase(bodies: &[Aabb]) -> Vec<(BodyIndex, BodyIndex)>`，
//! 算法选择封装在 crate 内部，未来可整体替换为 Dynamic AABB Tree。
//!
//! ## 算法（1D SAP 沿 X 轴）
//!
//! 1. 收集所有 `(idx, min_x, max_x)` 三元组；
//! 2. 按 `min_x` 升序排序；
//! 3. 维护 active list：每加入新 AABB，对 active 中所有 `max_x >= new.min_x` 的
//!    项做 Y/Z 轴二次检查；不重叠则从 active 移除；
//! 4. 把当前 AABB 加入 active；输出所有 `(i, j)` 对（`i < j`）。
//!
//! ## 复杂度
//!
//! - 排序：`O(n log n)`；
//! - sweep 主体：最坏 `O(n^2)`（全重叠场景），典型帧间运动连贯时接近 `O(n + k)`，
//!   其中 `k` = 实际重叠对数。
//!
//! ## `BodyIndex` 类型
//!
//! design doc §7.2 用 `BodyIndex` 表示 body 索引；本 crate 定义为 `u32` 的类型别名
//! 以严格匹配文档；`gvpe_core::BodyHandle` 包含 `generation` 字段，世代校验
//! 由调用方在调用 `broad_phase` 前完成。

use gvpe_math::Aabb;

/// Body 索引（对应 `06_collision_design.md §7.2` 的 `BodyIndex`）。
///
/// **注意**：与 `gvpe_core::BodyHandle`（含 `generation`）不同 —— broad phase
/// 只关心索引值；句柄解析 / 世代校验在调用方（runtime / solver）层完成。
pub type BodyIndex = u32;

/// 粗筛主入口：返回所有可能相交的 AABB 对（i, j）（`i < j`）。
///
/// 重复调用间无状态共享（无增量维护）；每帧从 `bodies` 重建索引。
/// v0.8+ 引入增量式 SAP（per-frame sort + insertion sort）时，本函数语义保持不变。
///
/// ## 边界
///
/// - `bodies.len() < 2` → 返空 `Vec`；
/// - 退化 AABB（`min > max` 任一分量）按几何规则仍参与重叠判定（保守选择）；
///   调用方应保证 `min <= max`。
pub fn broad_phase(bodies: &[Aabb]) -> Vec<(BodyIndex, BodyIndex)> {
    let n = bodies.len();
    if n < 2 {
        return Vec::new();
    }

    // 1) 收集 (idx, min_x, max_x) 并按 min_x 升序
    let mut entries: Vec<(usize, f32, f32)> = bodies
        .iter()
        .enumerate()
        .map(|(i, aabb)| (i, aabb.min.x, aabb.max.x))
        .collect();
    entries.sort_by(|x, y| x.1.partial_cmp(&y.1).unwrap_or(std::cmp::Ordering::Equal));

    // 2) Sweep
    let mut pairs: Vec<(BodyIndex, BodyIndex)> = Vec::new();
    let mut active: Vec<ActiveEntry> = Vec::with_capacity(n);
    for &(idx, min_x, max_x) in &entries {
        // 移除 active 中 max_x < 当前 min_x 的项
        active.retain(|a| a.max_x >= min_x);
        // 与 active 中所有项做 3D 重叠检查
        for a in &active {
            if aabb_overlap_3d(&bodies[idx], &bodies[a.idx]) {
                let (i, j) = if idx < a.idx { (idx, a.idx) } else { (a.idx, idx) };
                pairs.push((i as BodyIndex, j as BodyIndex));
            }
        }
        // 当前 AABB 加入 active
        active.push(ActiveEntry {
            idx,
            max_x,
        });
    }
    pairs
}

#[derive(Clone, Copy, Debug)]
struct ActiveEntry {
    idx: usize,
    max_x: f32,
}

/// 3D AABB 重叠判定（与 `gvpe_math::Aabb::overlaps` 等价；内联避免跨 crate 依赖）。
#[inline]
fn aabb_overlap_3d(a: &Aabb, b: &Aabb) -> bool {
    a.min.x <= b.max.x
        && a.max.x >= b.min.x
        && a.min.y <= b.max.y
        && a.max.y >= b.min.y
        && a.min.z <= b.max.z
        && a.max.z >= b.min.z
}
