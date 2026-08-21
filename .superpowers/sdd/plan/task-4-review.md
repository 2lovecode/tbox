# Task 2.2 Review — 预置 Skill 检索

**Reviewer:** Code review gate (task-scoped)  
**Commit:** `3a7d810` — `feat: retrieve preset tool skills for agent context`  
**Sources:** `task-4-brief.md`, `task-4-report.md`, `task-4-review-pkg.txt`（静态 diff / 源码核对；本 gate 重跑 `cargo test --lib agent::skills --no-run` 与一次 lib test 执行）

---

## 结论摘要

实现与 Task 2.2 brief 及 `Preset Skills Only` 规格对齐：12 份 `include_str!` 预置 Skill（Base64 encode/decode 合并）均含 `tool_id` / `keywords` front matter；公开接口 `SkillDoc` + `retrieve_skills(query, limit)` 存在且 `truncate(limit)` 尊重上限；JWT Skill 正文与标题明确「只解析、不验签」；`skills.rs` 不调用 `dispatch`，加载/检索不扩展注册表；`registry.rs` 未出现在 diff 中。两则 TDD 测试名与 plan 一致；Windows 上 lib test 进程仍 `STATUS_ENTRYPOINT_NOT_FOUND`，与 Task 2.1 环境一致，编译 GREEN 可接受。

| 维度 | 裁决 |
|------|------|
| **Spec** | ✅ |
| **Quality** | **Approved** |

---

## Spec 符合性（对照 brief + 全局约束）

| 要求 | 结论 | 证据（diff / 源码） |
|------|------|---------------------|
| 接口 `SkillDoc { tool_id, body }` + `retrieve_skills(query, limit)` | ✅ | `skills.rs` 公开 struct 与函数签名与 brief verbatim 一致 |
| `include_str!` 打进二进制 | ✅ | `static SKILLS` 中 12 个 `include_str!("../../skills/*.md")` |
| 每注册表工具一份 Skill（Base64 合并，JWT 独立） | ✅ | 12 个 `.md`；`base64.md` 的 `tool_id: base64.encode,base64.decode`；覆盖 registry 全部 13 个 id |
| front matter 含 `tool_id` 与 `keywords` | ✅ | 12 文件 grep 均有两行；`parse_skill` 解析失败则跳过 |
| 关键词/工具名/中文别名 substring 匹配，无 embedding | ✅ | `contains_match` + `score_skill`（keywords + tool_id + segment） |
| 尊重 `limit`（`hits.len() <= limit`） | ✅ | `limit == 0` 早退空 vec；`ranked.truncate(limit)`；测试 `assert!(hits.len() <= 3)` |
| JWT Skill 写明只解析、不验签 | ✅ | `jwt.parse.md` 标题「不验签」；正文「**只解析结构，不验证签名。**」「`verified` 恒为 `false`」 |
| Skill 不得调用 `dispatch` 注册新工具 | ✅ | `skills.rs` 无 `dispatch` / 无注册副作用；仅测试 `use registry::lookup` |
| 加载 Skill 不扩大注册表（`http.request` 仍为 None） | ✅ | `loading_skills_does_not_register_tools`：`lookup("http.request")` 前后均为 `None` |
| 注册表工具集不变 | ✅ | diff 无 `registry.rs`；13 个 allowlisted id 仍在 Task 2.1 提交中 |
| TDD：`jwt_query_hits_jwt_skill` / `loading_skills_does_not_register_tools` | ✅ | `#[cfg(test)]` 两测存在；后者使用 `lookup` |
| `pub mod skills` in `agent/mod.rs` | ✅ | diff 仅增一行 |
| tasks.md 勾选 2.2 | ✅ | `- [x] 2.2 ...` |
| commit 范围 | ✅ | 15 文件：12 skills + `skills.rs` + `mod.rs` + `tasks.md`；未 `git add .` |
| commit message | ✅ | `feat: retrieve preset tool skills for agent context` |

### 与 implementer report 的核对

| 声称 | 验证 |
|------|------|
| 12 份 Skill + 检索实现 | ✅ |
| RED stub 后 GREEN 实现 | ✅（依 report；本 gate 见完整实现与测试） |
| Windows `--no-run` + `cargo check` | ✅ 本 gate：`cargo test --lib agent::skills --no-run` exit 0 |
| Windows 无法跑通 lib test 进程 | ✅ 本 gate：`STATUS_ENTRYPOINT_NOT_FOUND` (0xc0000139) |
| 未改 `registry.rs` | ✅ |

---

## 质量评估

### 做得好的地方

- **边界清晰**：Skill 层纯只读检索，与 registry/dispatch 解耦，符合「说明书不注册执行器」。
- **嵌入策略简单可靠**：`include_str!` 无运行时 IO，与 brief 一致。
- **JWT 安全表述到位**：标题、正文、输出语义（`verified: false`）三层强调不验签，满足规格与安全预期。
- **limit 双保险**：空 query / `limit==0` 早退 + `truncate`，默认调用方 `limit=3` 可直接用。
- **测试对准门禁**：JWT 命中 + 注册表不扩张两测与 plan/brief 一致；`http.request` 与 Task 2.1 同一负例 id。

### 发现项

#### Critical

*无*

#### Important

1. **Lib 测试在本机 Windows 未实际执行通过**  
   与 Task 2.1/1.x 相同：`cargo test --lib agent::skills` → `STATUS_ENTRYPOINT_NOT_FOUND`。编译与测试二进制生成成功（`--no-run` exit 0），逻辑审查通过；**运行时 GREEN 证据仍缺**。建议在可跑 lib test 的环境补跑  
   `cargo test --lib agent::skills -- --nocapture`。

#### Minor

1. **未单独断言 `limit=1` 或「不全量注入」**  
   JWT 测试仅检查 `len <= 3`；实现已 `truncate`，但缺少「多命中时只返回 1 条」的显式用例。对 brief 足够，后续 Agent 集成时可补。

2. **同分排序不稳定**  
   `sort_by` 仅比 score，tie 时顺序依赖 `SKILLS` 静态数组顺序。brief 未要求稳定排序；若 UX 敏感可 secondary key（如 tool_id）。

3. **JWT keywords 含「验签」**  
   用于检索匹配可接受，且正文立即纠正为不验签；若担心误导，可改为「不解密」类词，非本 task 阻塞项。

4. **`loading_skills_does_not_register_tools` 未断言检索本身有命中**  
   查询 `"http"` 可能命中 URL 编码 Skill 或零命中；测试目标仍是 registry 不变，已满足 brief，但未证明「加载」与「检索」路径分离的更强不变量。

---

## Gate 裁决

| 检查项 | 结果 |
|--------|------|
| Spec 全部 MUST（含用户门禁五项） | ✅ |
| Critical | 0 |
| Important | 1（环境级测试未跑通，非逻辑缺口） |
| **Quality** | **Approved** |

**Task 2.2 通过。** 可继续 Task 2.3（mock LLM Agent 循环）；建议在能跑 `cargo test --lib` 的环境补跑 `agent::skills` 两测。
