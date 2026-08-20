# 测试总结报告模板（Test Summary Report Template）

> **用途**：单元测试 / 集成测试 / 系统测试 阶段结束后的总结报告。
> **对应工作流步骤**：
> - 65 単体試験完了承認（UT 完）→ `28_workflow.md` §11.11
> - 75 結合試験（IT 回归）→ §11.13-§11.15
> - 89 システム試験完了承認（ST 完）→ §11.17
> - 126 リグレッションテスト（回归测试）→ §11.37 / `28_workflow.md` §10.13
> **关联**：`29_unit_test_spec_template.md`（UT 规格）；`30_integration_test_spec_template.md`（IT 规格）；`32_system_test_spec_template.md`（ST 规格）；`35_uat_spec_template.md`（UAT 规格）；`39_release_checklist.md`（release gate）；`27_qa_register.md`（QA 项关闭）。

## 0. 报告元数据

| 字段 | 取值 |
|---|---|
| 报告类型 | ☐ UT 总结 ☐ IT 总结 ☐ ST 总结 ☐ UAT 总结 ☐ 回归测试总结 |
| 报告编号 | `TSR-XXX` |
| 关联测试规格 | `29` / `30` / `32` / `35` / 其他 |
| 关联 release / 周期 | `<vX.Y.Z>` / `<YYYY-MM-DD ~ YYYY-MM-DD>` |
| 测试负责人 | — |
| 报告日期 | `<YYYY-MM-DD>` |

## 1. 测试范围

| 维度 | 范围 |
|---|---|
| 涉及 crate | ... |
| 涉及 feature | ... |
| 平台 / OS | ... |
| Rust toolchain | ... |

## 2. 测试用例统计

| 类别 | 总数 | 通过 | 失败 | 跳过 | 通过率 |
|---|---|---|---|---|---|
| 功能 | | | | | |
| 错误路径 | | | | | |
| 边界 / 数值 | | | | | |
| 性能 | | | | | |
| 障碍 | | | | | |
| FFI | | | | | |
| 回归 | | | | | |
| **合计** | | | | | |

## 3. 关键测试用例结果

> 摘录最重要的 5-10 个测试用例（依报告类型选）。

| 测试 ID | 名称 | 期望 | 实际 | 通过 |
|---|---|---|---|---|
| | | | | ☐ |
| | | | | ☐ |
| | | | | ☐ |

## 4. 失败用例详情

| 测试 ID | 名称 | 失败原因 | 严重度 | 状态 | 关联 |
|---|---|---|---|---|---|
| | | | ☐ Blocker ☐ High ☐ M ☐ L | ☐ Open ☐ In-progress ☐ Closed | `27_qa_register.md` `QA-X-NN` |

## 5. 性能数字

| 场景 | 目标 | 实测 | 通过 | 关联 |
|---|---|---|---|---|
| | | | ☐ | `GVPE-DOC-14` |
| | | | ☐ | |

## 6. 覆盖率

| 指标 | 目标 | 实测 | 通过 |
|---|---|---|---|
| 行覆盖率 | ≥ 80% | ___% | ☐ |
| 分支覆盖率 | ≥ 70% | ___% | ☐ |
| 公共 API 覆盖率 | 100% | ___% | ☐ |
| `unsafe` 块 miri 覆盖 | 100% | ___% | ☐ |

## 7. Feature-gate 验证

| 命令 | 期望 | 实测 | 通过 |
|---|---|---|---|
| `cargo test --all-features` | 通过 | | ☐ |
| `cargo test --no-default-features` | 通过 | | ☐ |
| `cargo test --no-default-features --features simd-only` | 通过 | | ☐ |
| `cargo tree -p gvpe-core ...` | 不含 Graph / Vector / AI / 3DGS | | ☐ |

## 8. 工具链验证

| 检查 | 期望 | 实测 | 通过 |
|---|---|---|---|
| `cargo fmt --all -- --check` | 0 差异 | | ☐ |
| `cargo clippy --all-targets --all-features -- -D warnings` | 0 warning | | ☐ |
| `cargo +nightly miri test` | 0 UB | | ☐ |
| `cargo audit` | 0 High / Critical | | ☐ |
| `cargo deny check` | 0 violation | | ☐ |

## 9. 已知问题

> 不构成 Blocker 但应在下 release 处理。

| ID | 描述 | 严重度 | 触发条件 | 关联 |
|---|---|---|---|---|
| | | ☐ M ☐ L | | |

## 10. 结论

- [ ] ☐ **通过**：可进入下一阶段 / release；
- [ ] ☐ **有条件通过**：§4 失败用例中的 Blocker / High 全部 Closed 后可进入；
- [ ] ☐ **不通过**：必须修复后重测。

## 11. 关联

- `28_workflow.md` §11.11 / §11.13 / §11.17
- `29_unit_test_spec_template.md`
- `30_integration_test_spec_template.md`
- `32_system_test_spec_template.md`
- `35_uat_spec_template.md`
- `39_release_checklist.md`（release gate）
- `27_qa_register.md`（QA 项关闭 / 新增）

## 12. 签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 测试负责人 | | | |
| QA 负责人 | | | |
| 架构师 | | | |
| Release Manager（如适用） | | | |
