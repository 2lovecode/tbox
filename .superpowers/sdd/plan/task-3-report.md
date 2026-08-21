# Task 2.1 Report — Agent 工具注册表

**Status:** DONE  
**Date:** 2026-08-22  
**Change:** `openspec/changes/add-chat-home-agent`

## Summary

实现 allowlisted Agent 工具注册表：`lookup` / `dispatch` / `ToolSpec` / `SideEffect::None`。注册 13 个稳定字符串 id（对应规格中的 12 个纯计算工具；Base64 拆为 encode/decode）。未注册 id（含 `http.request`）拒绝；手写 schema 必填/类型校验，无 `jsonschema` 依赖。缺省 command 的工具在 `registry.rs` 内用已有 crate 薄实现；`json.format` / `encoding.convert` 复用已跟踪的 `commands::json` / `commands::encoding`。

## Files

| Path | Action |
|------|--------|
| `src-tauri/src/agent/mod.rs` | Created |
| `src-tauri/src/agent/registry.rs` | Created |
| `src-tauri/src/lib.rs` | Modified (`pub mod agent;`) |
| `openspec/changes/add-chat-home-agent/tasks.md` | 勾选 2.1 |

未改 `encoding.rs`（Base64 在 registry 内实现）。未纳入未跟踪的 `commands/*.rs`。

## Registered tool ids

全部 `side_effect=None`，均可 `lookup`：

- `json.format` → `commands::json::format_json_pretty`
- `base64.encode` / `base64.decode` → `base64` crate
- `hash.digest` → `md-5` / `sha2`（md5 | sha256）
- `jwt.parse` → Base64URL 解析 header/payload，**不验签**
- `timestamp.convert` → `chrono`
- `encoding.convert` → `commands::encoding::url_encode`
- `xml.format` → `serde-xml-rs`（失败时空白折叠回退）
- `yaml.format` → `serde_yaml`
- `uuid.generate` → `uuid` v4
- `cron.explain` → 5 字段中文说明
- `number.convert` → 进制 2..=36
- `charset.convert` → 校验 charset；UTF-8 原样；其它走 `encoding_rs` 标签

**禁止：** `http.request`、network、db_tools 均不在表中。

## TDD Evidence

### RED（必须：生产符号缺失的编译失败）

先只提交测试 + `mod agent`，未定义 `dispatch`：

```text
cargo test --lib agent::registry -- --nocapture
error[E0425]: cannot find function `dispatch` in this scope
  --> src\agent\registry.rs:12:19
  --> src\agent\registry.rs:18:19
  --> src\agent\registry.rs:24:23
  --> src\agent\registry.rs:25:23
error: could not compile `tbox` (lib test) due to 4 previous errors
```

确认失败原因是缺失 `dispatch`，而非断言写错。

### GREEN

实现 `lookup` / `validate_args` / `dispatch` 及全部工具薄实现后：

```text
cargo test --lib agent::registry -- --nocapture
   Compiling tbox ...
    Finished `test` profile ...
     Running unittests src\lib.rs (...\tbox_lib-....exe)
error: test failed ... (exit code: 0xc0000139, STATUS_ENTRYPOINT_NOT_FOUND)
```

**编译成功**；进程启动失败为 Windows 已知问题（brief 已预见）。补充验证：

```text
cargo test --lib agent::registry --no-run   # OK — Finished test profile
cargo check                                 # OK — Finished dev profile
```

测试用例已存在且可链接进 test 二进制：

- `unknown_tool_is_rejected`
- `invalid_args_do_not_run`
- `base64_roundtrip`
- `all_allowlisted_tools_lookupable`（额外：全部 id + 拒绝 http）

## Commit

```text
feat: add allowlisted agent tool registry
```

仅 add：`src-tauri/src/agent/**`、`src-tauri/src/lib.rs`、`openspec/changes/add-chat-home-agent/tasks.md`。未 `git add .`。

## Concerns

1. **Windows 无法实际跑通 `--lib` 测试进程**（`STATUS_ENTRYPOINT_NOT_FOUND`）；GREEN 证据为编译/`--no-run`/`cargo check`。建议在非沙箱或 Linux CI 再跑一遍。
2. `xml.format`：`serde_xml_rs` ↔ `serde_json::Value` 能力有限，复杂 XML 可能走空白折叠回退，非严格 pretty-print。
3. `charset.convert` 为最小可用实现，非完整编码转码管线。
4. 未把未跟踪 command 模块接入 `commands/mod.rs`（按 controller 裁定）。
