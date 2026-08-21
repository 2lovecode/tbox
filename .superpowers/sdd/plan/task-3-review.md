# Task 2.1 Review — Agent 工具注册表

**Reviewer:** Code review gate (task-scoped)  
**Base:** `ad0a18f4f139fcb59d0142e2cc1f157e3f6ac736`  
**Head:** `96a37e222c3393348afce0f38952e4e6e2e90eac`  
**Sources:** `task-3-brief.md`, `task-3-report.md`, `task-3-review-pkg.txt`（未重跑完整测试套件；仅静态 diff / 源码核对）

---

## 结论摘要

实现与 Task 2.1 brief、`Allowlisted Pure-compute Tools` 及全局门禁对齐：13 个 allowlisted id 均可 `lookup`，全部 `side_effect=None`；`dispatch` 拒绝未知 id（含 `http.request`）；手写 schema 校验，无 `jsonschema`；`jwt.parse` 仅 Base64URL 解 header/payload 并固定 `verified: false`，无验签；未接入 network/db；提交仅 4 个允许路径，未夹带未跟踪 command 模块。三个强制 TDD 测试名齐全，RED 证据为缺失 `dispatch` 的编译失败。

| 维度 | 裁决 |
|------|------|
| **Spec** | ✅ |
| **Quality** | **Approved** |

---

## Spec 符合性（对照 brief + 全局约束）

| 要求 | 结论 | 证据（diff / 源码） |
|------|------|---------------------|
| 接口 `SideEffect` / `ToolSpec` / `lookup` / `dispatch` | ✅ | `registry.rs` 与 brief verbatim 一致；`dispatch` → `Result<String, String>` |
| Allowlisted ids 全集（13，Base64 拆 encode/decode） | ✅ | `TOOLS` + `all_allowlisted_tools_lookupable` |
| 全部 `side_effect=None` | ✅ | `tool()` 固定 `SideEffect::None`；lookup 测试断言 |
| `jwt.parse` 不验签 / 不解密 | ✅ | 只解 parts[0]/[1]；输出 `verified: false`；未读签名段做 HMAC/RSA |
| 禁止 http / db 工具 | ✅ | 表中无此类 id；`dispatch`/`lookup` 不引用 `network`/`db_tools`；测试拒绝 `http.request` |
| 手写必填校验，无 jsonschema crate | ✅ | `validate_args`；`Cargo.toml` 无 `jsonschema` |
| TDD：`unknown_tool_is_rejected` / `invalid_args_do_not_run` / `base64_roundtrip` | ✅ | `#[cfg(test)]` 三测存在；另加 allowlist lookup 测 |
| RED = 生产符号缺失类编译失败 | ✅（依 report） | 报告 `cannot find function dispatch`；本 gate 未重放历史 |
| GREEN：`--no-run` / `cargo check`（Windows ENTRYPOINT） | ✅（依 report） | brief 预见 `STATUS_ENTRYPOINT_NOT_FOUND`；本 gate 未重跑 |
| `mod agent`；不必挂 generate_handler | ✅ | `lib.rs` 仅 `pub mod agent;` |
| 不提交未跟踪 command 堆 | ✅ | commit 仅 `agent/**`、`lib.rs`、`tasks.md` |
| tasks.md 勾选 2.1 | ✅ | `- [x] 2.1 ...` |
| commit message | ✅ | `feat: add allowlisted agent tool registry` |

### 与 implementer report 的核对

| 声称 | 验证 |
|------|------|
| 13 id + 全 None | ✅ |
| Base64 / hash / jwt 等在 registry 薄实现 | ✅；`json.format` / `encoding.convert` 复用已跟踪 `commands::json` / `encoding` |
| 未改 encoding.rs / 未 add 未跟踪 commands | ✅ commit 文件列表 |
| 三强制测试 + 额外 lookup 测 | ✅ |
| 无 jsonschema | ✅ |
| Windows 无法跑通 lib test 进程 | ✅ 与 Task 1.x 环境一致；brief 允许 `--no-run` |

---

## 质量评估

### 做得好的地方

- **门禁清晰**：lookup 表与 dispatch match 一一对应；未知 id 在校验前即拒绝。
- **依赖纪律**：优先已跟踪 command + Cargo.toml 已有 crate，避免污染 commit。
- **JWT 安全边界明确**：名称与返回值都标明不验签，符合规格「不含验签」。
- **校验在执行前**：`validate_args` 后再 match；空参 `base64.encode` 不会进入 encode。
- **测试对准契约**：未知工具用 `http.request`；非法参用缺字段；roundtrip 断言精确。

### 发现项

#### Critical

*无*

#### Important

1. **Lib 测试在本机 Windows 未实际执行**  
   Report 与既有任务一致：`STATUS_ENTRYPOINT_NOT_FOUND`。本 gate 依 `--no-run` / `cargo check` 与静态审查通过，但 **运行时 GREEN 证据仍缺**。建议 CI / 可跑 lib test 环境补跑  
   `cargo test --lib agent::registry -- --nocapture`。

#### Minor

1. **`additionalProperties: false` 未在 `validate_args` 中强制**  
   Schema 声明了禁止额外字段，手写校验只查 required / type / enum。多余键仍会进入部分工具。若要与「不符合 JSON Schema 则不调用」严格对齐，应拒绝未知属性。

2. **`xml.format` 解析失败时空白折叠回退**  
   非法 XML 可能返回「成功」压缩串而非错误；对 Agent 反馈偏弱。可接受为 brief「最小可用」，后续可改为硬失败。

3. **`charset.convert` 非 UTF-8 路径语义偏弱**  
   对已是 UTF-8 的 `String` 再按标签 `decode(as_bytes)`，非真正跨编码转码。Brief 允许最小实现；后续若要对齐真实 charset 工具，需字节/源编码模型。

4. **`invalid_args_do_not_run` 未用 spy/计数证明「未调用」**  
   仅断言校验错误文案；当前实现顺序正确，但测试对「不跑底层」是间接证据。

---

## Gate 裁决

| 检查项 | 结果 |
|--------|------|
| Spec 全部 MUST / 全局约束 | ✅ |
| Critical | 0 |
| Important | 1（环境级测试未跑通，非逻辑缺口） |
| **Quality** | **Approved** |

**Task 2.1 通过。** 可继续 Task 2.2（预置 Skill）或后续 Agent 循环任务；建议在能跑 `cargo test --lib` 的环境补跑 registry 三测。
