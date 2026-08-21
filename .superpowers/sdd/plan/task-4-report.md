# Task 2.2 Report — 预置 Skill 检索

**Status:** DONE  
**Date:** 2026-08-22  
**Change:** `openspec/changes/add-chat-home-agent`

## Summary

为 12 个注册表工具（Base64 encode/decode 合并一份）添加 `src-tauri/skills/*.md`，front matter 含 `tool_id` 与 `keywords`；`include_str!` 打进二进制。实现 `SkillDoc` + `retrieve_skills(query, limit)` 关键词/工具名/中文别名匹配，尊重 `limit`；JWT Skill 正文写明只解析不验签。未改 `registry.rs` 工具集。

## Files

| Path | Action |
|------|--------|
| `src-tauri/skills/*.md` (12) | Created |
| `src-tauri/src/agent/skills.rs` | Created |
| `src-tauri/src/agent/mod.rs` | Modified (`pub mod skills;`) |
| `openspec/changes/add-chat-home-agent/tasks.md` | 勾选 2.2 |

## TDD Evidence

### RED

先提交 stub `retrieve_skills` 返回空 vec + 两则测试；编译通过，运行时 `jwt_query_hits_jwt_skill` 会断言失败（空 hits）。

### GREEN

实现 front matter 解析 + 打分检索后：

```text
cargo test --lib agent::skills --no-run   # OK
cargo check                                 # OK
```

`cargo test --lib agent::skills` 仍因 Windows `STATUS_ENTRYPOINT_NOT_FOUND` 无法启动 test 进程（同 Task 2.1）。

测试用例：

- `jwt_query_hits_jwt_skill`
- `loading_skills_does_not_register_tools`（`lookup("http.request")` 前后均为 None）

## Commit

```text
feat: retrieve preset tool skills for agent context
```

仅 add：`src-tauri/skills/**`、`skills.rs`、`mod.rs`、`tasks.md`。未 `git add .`。

## Concerns

1. Windows 无法实际跑通 `--lib` 测试进程；GREEN 证据为 `--no-run` + `cargo check`。
2. 检索为简单 substring 打分，非 embedding；多关键词并列时排序可能需后续调优。
