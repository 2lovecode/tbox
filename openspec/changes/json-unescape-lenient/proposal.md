# Proposal

## Why

JSON 处理工具的「去转义」对带 `\\n`、`\\uXXXX` 等双重转义的粘贴串（常见于 ES query / 日志摘录）会解坏或判定无效；「美化」已具备正确解层，去转义应对齐同一能力。

## What Changes

- 增强 `unescape_json`：按 JSON 字符串字面量正确解一层（含 `\uXXXX`），必要时多层剥壳，直到得到稳定文本。
- 输出仍为**未美化**的还原字符串；`is_valid` 表示还原结果能否解析为 JSON。
- 补充单测覆盖 ES query 类样例。
- 前端契约不变（`JsonToQuery` 无关；`JsonTool.vue` 仍读 `unescaped` / `is_valid`）。

## Non-goals

- 不去转义后自动美化 / 双结果输出。
- 不改 `escape_json` 语义。
- 不改 Agent `json.format` 路径（已用 lenient 解析）。

## Capabilities

### New Capabilities

- `json-tools`: JSON 处理工具（美化/压缩/转义/去转义等）的对外行为契约

### Modified Capabilities

- （无）

## Impact

- Rust: `src-tauri/src/commands/json.rs`（`unescape_json`）
- 前端: `src/views/JsonTool.vue`（无契约变更，仅受益于后端增强）
- 工具 id: 9（JSON处理工具）
