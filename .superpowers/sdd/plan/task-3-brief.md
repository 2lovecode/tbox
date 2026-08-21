# Task 2.1 — Agent 工具注册表

来源：plan.md Task 2.1；spec Allowlisted Pure-compute Tools

## Files

- Create: `src-tauri/src/agent/mod.rs`
- Create: `src-tauri/src/agent/registry.rs`
- Modify: `src-tauri/src/lib.rs`（`mod agent;` 即可，本任务不必把 registry 挂到 generate_handler）
- 若 Base64 尚无 Rust command：在 `src-tauri/src/commands/encoding.rs` 增加 `base64_encode` / `base64_decode`（可用已有 `base64` crate）
- 勾选 `openspec/changes/add-chat-home-agent/tasks.md` 2.1

## Interfaces（verbatim）

```rust
pub enum SideEffect { None }
pub struct ToolSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub schema: serde_json::Value,
    pub side_effect: SideEffect,
}
pub fn lookup(tool_id: &str) -> Option<&'static ToolSpec>
pub fn dispatch(tool_id: &str, args: &serde_json::Value) -> Result<String, String>
```

注册 id（必须全部 lookup 得到，全部 side_effect=None）：

- `json.format` — 对应工具 id 9；调度已有 `commands::json::format_json_pretty`（或最小包装，结果 `to_string`）
- `base64.encode` / `base64.decode` — 工具 id 10；参数必填 `input: string`
- `hash.digest` — 工具 id 11；参数 `input` + `algorithm`（至少 md5 或 sha256）
- `jwt.parse` — 工具 id 14；**只解析** header/payload，不验签、不解密
- `timestamp.convert` — 工具 id 16
- `encoding.convert` — 工具 id 19（例如 url encode 或 unicode；选一个已有 encoding command 即可）
- `xml.format` — 工具 id 20
- `yaml.format` — 工具 id 21
- `uuid.generate` — 工具 id 30（v4 即可）
- `cron.explain` — 工具 id 31
- `number.convert` — 工具 id 32（进制转换，最小可用）
- `charset.convert` — 工具 id 33（最小可用，例如 UTF-8 标签原样返回也可，但必须校验参数）

**禁止** `lookup("http.request")` 或任何 network/db_tools。

## TDD（必须看到 RED）

plan.md 中的三个测试必须存在：`unknown_tool_is_rejected`、`invalid_args_do_not_run`、`base64_roundtrip`。

Schema 手写必填字段检查，不要为了 schema 新加 jsonschema 依赖。

## 关于未跟踪的 command 文件

工作区有很多 **未提交** 的 `commands/*.rs`（crypto、datetime、uuid_tools 等）。本任务 **不要** 把整批未跟踪工具文件塞进 commit。

优先：
1. 已在 git 中的模块（`json.rs`、`encoding.rs`）直接调用
2. 其余工具在 `registry.rs` 内用 **已在 Cargo.toml 的 crate**（`base64`, `sha2`, `md-5`, `uuid`, `serde_json`, `chrono`）写薄实现
3. xml/yaml 可用已有 `serde_yaml` / `serde-xml-rs` 做 format/pretty

`dispatch` 返回 `String`（JSON 或纯文本均可，测试只对 base64 roundtrip 要求精确）。

## Verify

`cd src-tauri && cargo test --lib agent::registry -- --nocapture`
若运行时 `STATUS_ENTRYPOINT_NOT_FOUND`：至少 `--no-run` 编译成功 + `cargo check`，并在报告里写明。RED 必须是「函数不存在」类编译失败，不能跳过。

## Commit

只 add：`src-tauri/src/agent/**`、`src-tauri/src/lib.rs`、如有则 `encoding.rs`、`tasks.md`。禁止 `git add .`。

`git commit -m "feat: add allowlisted agent tool registry"`
