## Why

真实使用中用户问「把 `{"aa":"bb"}` 转成 query」时，Agent 反问「请问您想用哪种方式」——根因不是模型不行，而是 **Agent 注册表里没有 `json.to_query` 这种应用层工具**，模型识别不到可执行路径。Rust 侧 `commands/json.rs::json_to_query_params` 已经实现且前端 `JsonToQuery.vue` 已在用，只是没暴露给 Agent。`form.parse` / `url.parse` / `json.flatten` 同样缺失。

本 change 把已有的应用层能力注册到 Agent 工具表，并补充对应 Skill 与评测用例，让 0.5B 在 Agent 对话里就能完成「JSON→query」「表单解析」「URL 拆解」「嵌套平铺」等高频任务。

## What Changes

- **复用现有实现**：将 `commands::json::json_to_query_params` 包成 Agent 工具 `json.to_query`（同一进程复用，不写新逻辑），同时改返回值为简单字符串（`aa=bb&x=hi`），去掉前端专用的双结果结构。
- **新增 3 个应用层工具**（注册表 + dispatch + Skill + eval case）：
  - `json.flatten`：复用 `flatten_json_value` 辅助，输出平铺后的 JSON（键路径 `a.b[0].c`）。
  - `url.parse`：URL 字符串 → 结构化 JSON（scheme/host/port/path/query_map/fragment），依赖 `urlencoding` crate。
  - `form.parse`：`application/x-www-form-urlencoded` 字符串 → JSON 对象（重复键合并为数组），依赖 `urlencoding::decode`。
- **Skill 文档厚化**：为全部 16 个注册工具（12 既有 + 4 新增）的预置 Skill 各补「典型应用场景 + 2-3 个多样问法 → 工具调用」样本。
- **评测集扩充**：从 33 条扩到 ≥60 条，覆盖 16 工具 × 中英文问法 + 多步调用 + 「无可用工具」负例。

### Non-goals
- 不改 Agent 循环、harness 策略层、解析容错、reask 行为（已归档 `agent-tool-harness` 负责）。
- 不为新工具加前端页面（Agent 对话内调用；前端页面已有/可后续单开 change）。
- 不改 local-llm-runtime。
- 不引入新 crate（`urlencoding`、`percent_encoding`、`serde_json` 已就绪）。

## Capabilities

### New Capabilities
- `app-layer-tools-registered`：4 个应用层工具（`json.to_query` / `json.flatten` / `url.parse` / `form.parse`）在 Agent 注册表中可用，dispatch 实现完整、Skill 完整、eval 覆盖。
- `skill-content-enrichment`：全部 16 个注册工具的 Skill 文档包含「典型应用场景 + 多样问法样本」。
- `app-layer-eval-suite`：评测集 ≥60 条，逻辑层 `cargo test` 必跑、端到端 `--features agent-eval`。

### Modified Capabilities
- `agent-tool-harness`：评测集门槛 ≥60 条（MODIFIED Requirement：Offline Evaluation Suite 的「端到端层 MUST 输出至少四项指标」条款补「数据门槛」约束。

## Impact

- **Rust**：`src-tauri/src/agent/registry.rs` 新增 4 个 `tool(...)` 项 + 4 个 dispatch 分支；`src-tauri/src/commands/json.rs` 中 `json_to_query_params` 返回值轻量化（去掉 `JsonToQueryResult` 包装，改 `Result<String, String>`）——但前端 `JsonToQuery.vue` 仍调用原入口，因此需保留旧函数或保留双返回兼容前端。
- **Skills**：12 个 `src-tauri/skills/*.md` 文档扩写 + 4 个新增（`json.to_query.md` / `json.flatten.md` / `url.parse.md` / `form.parse.md`）。
- **评测**：`src-tauri/src/agent/harness/eval/cases.json` 33 → ≥60 条。
- **依赖**：无新增。
- **前端**：无（Agent 内调用；前端 `JsonToQuery.vue` 仍走原 command）。
- **规格**：3 个新主规格 + 1 个修订主规格。