# Task 2.2 — 预置 Skill 检索

来源：plan.md Task 2.2；spec Preset Skills Only

## Files

- Create: `src-tauri/skills/*.md` — 每个注册表工具一份（含 base64.encode/decode 可合并为一份 JWT 除外）。front matter 必须含 `tool_id` 与 `keywords`。
- Create: `src-tauri/src/agent/skills.rs`
- Modify: `src-tauri/src/agent/mod.rs` — `pub mod skills;`
- 勾选 tasks.md 2.2

## Interfaces

```rust
pub struct SkillDoc { pub tool_id: String, pub body: String }
pub fn retrieve_skills(query: &str, limit: usize) -> Vec<SkillDoc>
```

- 默认调用方传 limit=3；函数必须尊重 `limit`（hits.len() <= limit）
- Skill 只是说明书，**不得**调用 `dispatch` 注册新工具
- JWT skill 正文写明：只解析，不验签

检索：关键词/工具名/中文别名匹配即可，不要 embedding。

用 `include_str!` 打进二进制。

## TDD tests（必须先 RED）

plan.md 两个测试：`jwt_query_hits_jwt_skill`、`loading_skills_does_not_register_tools`（后者 `use crate::agent::registry::lookup`）。

## Verify / Commit

`cargo test --lib agent::skills`（若 STATUS_ENTRYPOINT_NOT_FOUND：`--no-run` + `cargo check`）

只 add: `src-tauri/skills/**`, `src-tauri/src/agent/skills.rs`, `mod.rs`, `tasks.md`

`git commit -m "feat: retrieve preset tool skills for agent context"`

Never `git add .`
