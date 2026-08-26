# 构建验证清单（Build Verification Checklist）

> 本文件记录**在干净环境下**应运行的命令，用于验证工程基线（v0.5）可用。
> 由于 GVPE 第一次落地的 crate 骨架在此 commit 中，请按本清单逐项验证。

## 1. 前置条件

- Rust toolchain ≥ 1.75（推荐 stable 最新，**不**需要 nightly）
  - `rustup default stable`
  - 或通过 `rust-toolchain.toml` 自动选择
- Cargo 1.75+
- Git

## 2. 验证步骤

### 2.1 工具链确认

```bash
rustc --version   # 应 ≥ rustc 1.75.0
cargo --version
```

### 2.2 工程基线（root 配置）

```bash
# 格式检查
cargo fmt --all -- --check

# Workspace check
cargo check --workspace
cargo check --workspace --all-features
cargo check --workspace --no-default-features

# MSRV 验证（可选，需要 rustup 安装 1.75）
rustup install 1.75.0
cargo +1.75.0 check --workspace
```

### 2.3 3 个核心 crate 单元测试

```bash
cargo test -p gvpe-math
cargo test -p gvpe-core
cargo test -p gvpe-memory
```

或一次性：

```bash
cargo test --workspace
```

### 2.4 关键布局断言

`gvpe-math` 和 `gvpe-core` 的测试覆盖了 `GVPE-DOC-58` 数据布局图谱中的关键尺寸：

- `Vec3` size = 12 bytes, align = 4
- `Quat` size = 16 bytes, align = 16（SIMD 友好）
- `Mat3` size = 36 bytes, align = 4
- `Aabb` size = 24 bytes, align = 4
- `Transform` size = 32 bytes, align = 16（Vec3(12) + 4 字节 padding + Quat(16)）
- `BodyHandle` size = 8 bytes, align = 4
- `ConstraintHandle` size = 8 bytes, align = 4
- `IslandHandle` size = 4 bytes, align = 4
- `PhysicsProfile` size ∈ [78, 84] bytes, align = 4

### 2.5 unsafe 块 miri 验证

```bash
rustup +nightly component add miri
cargo +nightly miri setup

# 核心 unsafe crate 跑 miri
cargo +nightly miri test -p gvpe-memory
```

### 2.6 cargo deny（许可证 + 禁止库）

```bash
cargo install --locked cargo-deny
cargo deny check
```

期望：无 violation。

### 2.7 cargo tree（AC-02 验证）

```bash
cargo tree -p gvpe-core -p gvpe-memory -p gvpe-math
```

期望：输出**不**包含 `gvpe-graph` / `gvpe-vector` / `gvpe-compiler` / `gvpe-inference` / `gvpe-3dgs`。

## 3. 已知风险

- `cargo check` 首次会从 crates.io 下载依赖（约 1-2 分钟，依赖网络）；
- CI 会在多平台矩阵上跑（见 `.github/workflows/ci.yml`）；
- 当前 commit 的代码在 Windows + Rust 1.95 stable 已通过作者验证（开发期抽查）；最终验证需在 Linux / macOS 上补足。

## 4. 修复常见问题

### 4.1 编译错误：crate 找不到

```bash
cargo clean
cargo build --workspace
```

### 4.2 cargo deny violation

若 `cargo deny` 报告拒绝列表中的库被引入，按 `28_workflow.md` §11.32 变更管理流程处理。

### 4.3 miri 失败

若 miri 在 `gvpe-memory` 报告 UB：

1. 跑 `cargo +nightly miri test -p gvpe-memory -- --test-threads=1` 隔离；
2. 检查 `// SAFETY:` 注释与实现是否匹配；
3. 必要时加更严格的类型约束（如 `NonNull<T>` 替代 `*mut T`）；
4. 更新 `62_unsafe_inventory.md` 记录。

## 5. 完成后

- 若全部通过：✅ 工程基线就绪，可进入 M2 阶段（继续添加 `gvpe-shape` / `gvpe-collision` 等）
- 若有失败：开 issue 跟踪（GitHub Issues / 内部 `27_qa_register.md`）
