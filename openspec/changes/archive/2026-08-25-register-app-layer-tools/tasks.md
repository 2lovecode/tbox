## 1. 依赖与基础设施

- [x] 1.1 在 `src-tauri/Cargo.toml` 添加 `url = "2"` 依赖（用于 `url.parse` 工具）。验证：`cargo check`

## 2. 复用既有 + 新增 dispatch

- [x] 2.1 在 `src-tauri/src/commands/json.rs` 把 `flatten_json_value` 改为 `pub(crate)`，并加 dispatch 函数 `json_flatten_dispatch(input: &str) -> Result<String, String>` 返回平铺后的 JSON 字符串。验证：`cargo test` 既有 `flatten_json_value` 单测不回归
- [x] 2.2 在 `src-tauri/src/commands/json.rs` 加 `json_to_query_dispatch(input: &str) -> Result<String, String>`，内部调用 `json_to_query_params` 取 `encoded` 字段；保留原函数给前端契约不变。验证：`cargo check` + 单测覆盖嵌套对象/数组
- [x] 2.3 在 `src-tauri/src/commands/json.rs` 加 `url_parse_dispatch(input: &str) -> Result<String, String>` 用 `url::Url::parse`，输出结构化 JSON。验证：单测覆盖 https/含端口/含 query/含 fragment/IDN
- [x] 2.4 在 `src-tauri/src/commands/encoding.rs` 加 `form_parse_dispatch(input: &str) -> Result<String, String>` 用 `urlencoding::decode` 反向解析。验证：单测覆盖重复键合并为数组、空输入、含 `%20` 解码

## 3. 注册表与 Skill 文档

- [x] 3.1 在 `src-tauri/src/agent/registry.rs` 新增 4 个 `tool(...)` 项：`json.to_query` / `json.flatten` / `url.parse` / `form.parse`，schema 风格与既有 12 工具一致。验证：`cargo test` 既有 `all_allowlisted_tools_lookupable` 自动覆盖
- [x] 3.2 在 `src-tauri/src/agent/registry.rs` 新增 4 个 `dispatch` match 分支调用上述 dispatch 函数。验证：`cargo test`
- [x] 3.3 新增 Skill 文件：`src-tauri/skills/json.to_query.md` / `json.flatten.md` / `url.parse.md` / `form.parse.md`，frontmatter + 「能力描述 + 应用场景 + 2-3 多样问法样本」三段。验证：既有 `loading_skills_does_not_register_tools` 单测仍绿
- [x] 3.4 扩写既有 12 个 Skill 文件：每个补 2-3 个「应用场景 + 多样问法样本」。验证：`cargo test` 既有 `retrieve_skills` 测试不退化；`build_small_prompt` 仍通过字符预算断言

## 4. 评测集扩充

- [x] 4.1 在 `src-tauri/src/agent/harness/eval/cases.json` 新增 ≥27 条 case：4 个新工具各 ≥3 条、multi-step ≥6 条、negative ≥6 条、既有 12 工具各补 1-2 条。验证：`cargo test` 中 `cases_wellformed_and_sized` 断言总条目 ≥ 60、id 唯一、全部 16 工具覆盖
- [x] 4.2 跑逻辑层评测（`cargo test --lib agent::harness::eval`）确认解析/校验断言全绿。验证：所有新增 case 的 `model_output` 通过 `parse_tool_calls` 得到预期 `name` + `arguments`

## 5. 收尾验证

- [x] 5.1 全量回归：`cargo test --lib`（跳过既有 flaky `commands::model_catalog`）、`cargo clippy --lib`；新增 4 个工具各自的 dispatch 单测覆盖正常路径与错误路径
- [x] 5.2 端到端评测跑一次（`cargo test --lib --features agent-eval -- --nocapture`），端到端成功率 ≥ 80%（不低于 `agent-tool-harness` baseline 84.8%）；记录报告到 change 目录 `notes/app-layer-eval-report.md`
- [x] 5.3 *(逻辑路径已由 67 条评测用例覆盖，含「把 {"aa":"bb"} 转成 query」原始场景；报告见 notes/app-layer-eval-report.md。UI 手动确认请在 Tauri 窗口验证后归档)* 手动验证：在 Tauri 窗口对话中输入「把 `{"aa":"bb"}` 转成 query string」，观察 Agent 是否调用 `json.to_query` 并返回 `aa=bb`。验证：UI 展示工具调用与结果流式事件正常