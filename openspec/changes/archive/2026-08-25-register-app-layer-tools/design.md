## Context

真实使用中 `json.to_query` 这类高频应用层任务用户已经在前端工具页直接用 `JsonToQuery.vue` 完成，但 Agent 对话里调不到——因为这些 Rust command 没进 Agent 注册表。模型识别不到可执行路径就反问用户「想用哪种方式」。

四个应用层工具的现状：
| 工具 id | Rust command | 前端页面 | Agent 注册表 | Skill | eval case |
|---|---|---|---|---|---|
| `json.to_query` | ✅ `commands/json.rs::json_to_query_params` | ✅ `JsonToQuery.vue` | ❌ | ❌ | ❌ |
| `json.flatten` | ⚠️ 私有辅助 `flatten_json_value` | ❌ | ❌ | ❌ | ❌ |
| `url.parse` | ❌ | ❌ | ❌ | ❌ | ❌ |
| `form.parse` | ❌ | ❌ | ❌ | ❌ | ❌ |

本 change 把这 4 个工具「接入 Agent」——`json.to_query` 直接复用现有 Rust 函数（前后端共进程），其他 3 个新写短实现。

## Goals / Non-Goals

**Goals:**
- 4 个应用层工具（`json.to_query` / `json.flatten` / `url.parse` / `form.parse`）注册到 Agent 工具表，可经 harness 完整流程调用。
- 全部 16 个注册工具 Skill 各补 2-3 个「应用场景 + 多样问法」样本。
- 评测集 ≥60 条，端到端成功率基线 ≥80%（与 `agent-tool-harness` baseline 一致）。

**Non-Goals:**
- 不改 Agent 循环、harness 策略、解析、reask。
- 不改前端页面（前端 `JsonToQuery.vue` 仍调原 command）。
- 不引入新 crate。

## Decisions

### D1: `json.to_query` 复用 + 包装
不重写。Agent 注册表的 `dispatch("json.to_query", args)` 直接调用 `commands::json::json_to_query_params` 函数。前端 `JsonToQuery.vue` 也调同一个函数——**保持单源**。

返回值统一为 `String`（按既有 dispatch 协议 `Result<String, String>`）。由于前端 `JsonToQueryResult` 含 `query_string` + `encoded` 两个字段，本 change 需要：
- 保持 `json_to_query_params` 返回 `JsonToQueryResult` 给前端；
- 注册表 dispatch 时调一个**新内部函数** `json_to_query_dispatch(input) -> Result<String, String>`，内部调用原函数并返回 `encoded` 字段；
- 或：保留旧函数 + 新加 dispatch-only 函数（避免破坏前端）。

**采用第二种**——保留前端契约，dispatch 走新函数。

### D2: `json.flatten` 复用私有辅助
`flatten_json_value` 已在 `commands/json.rs:229`。把它 `pub(crate)` 出来，dispatch 调用并收集 `(path, value)` 对，返回 `serde_json::Value`（对象，路径做 key、值做 value；嵌套值用 `Value::String(json)` 保留原始 JSON 字符串以便还原）。

### D3: `url.parse` 新写
用现成 `urlencoding::decode` 不够——需要拆 URL。用 `url::Url` crate（已通过 `urlencoding` 间接可用？检查）。Cargo.toml 里 `urlencoding` 引入未必带 `url`。需要看：

**可能新增 `url = "2"`**（最小依赖，纯 host 解析），或用 `regex` 自己拆。**倾向新增 `url = "2"`**（约 200KB，业界标准，无额外传递依赖）。

输出格式：`{"scheme": "https", "host": "api.x.com", "port": 443, "path": ["/v1/x"], "query": {"k": "v"}, "fragment": null}`。路径拆为段数组（与 `path_segments()` 一致）；query 解码后作为对象（重复键合并为数组）。

### D4: `form.parse` 新写
直接用 `urlencoding::decode` + `&str::split('&')` + `split('=')`，递归解析每对 `k=v` 到 JSON 对象。重复键合并为 JSON 数组。空字符串返回空对象。

### D5: Skill 文档扩写范式
每个 Skill 文件保持 frontmatter (`tool_id` / `keywords`)，body 改为「能力描述 + 典型应用场景 + 用户问法→工具调用样本」三段。检索逻辑 `retrieve_skills` 不变（按关键词 + 工具名 + 段匹配）。

新增 4 个 Skill 文件：
- `skills/json.to_query.md`：JSON → URL query（应用层高频）
- `skills/json.flatten.md`：嵌套 JSON 路径化
- `skills/url.parse.md`：URL 拆解
- `skills/form.parse.md`：表单字符串 → JSON

### D6: 评测集扩充分层
保留原 33 条逻辑层 case，向后兼容。新增 ≥27 条：
- 4 个新工具各 2-3 条（中英文 + 边界：空对象/特殊字符/嵌套）。
- 多步 ≥6 条（先 base64 解码再哈希；先 json.to_query 再 url-encode 整个 query string 等）。
- 「无可用工具」负例 ≥6 条。
- 既有 12 工具各补 1-2 条中英文样本。

继续单文件 JSON 管理，逻辑层随 `cargo test` 必跑。

### D7: 与 `agent-tool-harness` 主规格的协调
仅修订其 `Offline Evaluation Suite` 一条：增加「数据门槛 ≥60 条」约束。不破坏既有条款。

## Risks / Trade-offs

- [`json.to_query` 双返回值同时给 Agent 与前端] → 保留原函数给前端，Agent dispatch 走专用函数；零破坏。
- [`url` crate 新增约 200KB] → 接受，业界标准；备选用 `regex` 自拆成本高且易错。
- [Skill 文档扩长后检索噪声增加] → 检索仍按关键词 + 工具名 + 段匹配（既有），扩写不引入新字段；`limit=3` 仍生效。
- [评测门槛 ≥60 条引入维护负担] → 单文件 JSON + `cases_wellformed_and_sized` 测试保底，回滚易。

## Migration Plan

1. 新增 `url` 依赖。
2. 复用 `flatten_json_value` pub(crate)，加 `json.flatten` / `json.to_query` / `url.parse` / `form.parse` 4 个 dispatch 函数（分散在 `commands/json.rs` 和 `commands/encoding.rs` 已有文件中）。
3. `registry.rs` 加 4 个工具项。
4. 4 个 Skill 文件新增，12 个既有 Skill 扩写。
5. `cases.json` 扩到 ≥60 条；逻辑层测试通过。
6. 端到端评测跑一次，确认端到端 ≥ 80%（baseline 同 `agent-tool-harness` 84.8%）。
7. 主规格 MODIFIED 同步 + 归档。

回滚：去掉 dispatch match、注册表项、Skill include、eval case 行；纯增量代码，干净可逆。

## Open Questions

- `url::Url::parse` 对 IDN host 默认 punycode，本 change 不强制改写（保留原 host）。
- `form.parse` 重复键：合并为 JSON 数组（如 `a=1&a=2` → `{"a": ["1","2"]}`）——与 `www-urlencoded` 习惯一致。