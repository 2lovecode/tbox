# Retrospective: llm-multi-provider-protocols

> Written: 2026-08-24 (after verify passed; PASS 门槛在收尾时达成)
> Commit range: `3f0a9c6..90bae44` (3 commits, feature branch `b59a273` merged via merge commit)
> Worktree: merged to main (`.worktrees/llm-multi-provider-protocols` 已合并)

---

## 0. Evidence

- **Commit range**: `3f0a9c6..90bae44` (3 commits: `b59a273` 实现 + `3f0a9c6` 优化 + `90bae44` merge)
- **Diff size**: +3,186 / -4,442 lines across 59 files（负行数主要来自 `llm_presets_data.json` 由脚本再生成与 `Cargo.lock` 变动）
- **Tasks done**: 14/14（`grep -cE '^\s*- \[x\]' tasks.md` = 14；5.1 于归档收尾时由用户本机手测确认后勾选）
- **Active hours**: 估算 ~4–6 小时（brainstorm 至 merge，同日 17:17 实现 commit → 18:09 merge；含前序规划）
- **Subagent dispatches**: n/a（由 Cursor Agent apply 执行；无独立 dispatch 记录）
- **New external dependencies**: `genai = "0.6"`（MIT/Apache-2.0 双许可，Rust 多协议 LLM 客户端库）
- **Bugs encountered post-merge**: none（merge 后 main `cargo test` 35 passed / 0 failed）
- **OpenSpec validate state at archive**: pass（`openspec validate --all --json` 9/9 valid，仅 2 条非阻塞 WARNING）
- **Test coverage signal**: `cargo test` 31 (lib) + 4 (integration) = 35 passed；`npm run build`（vue-tsc + vite）通过；无覆盖率工具

Commit chain (時序):

```
3f0a9c6 feat:opt（基线，main 侧）
b59a273 feat(llm): multi-provider presets, protocols, and genai adapter（feature 分支全部实现，单 commit）
90bae44 Merge branch 'feature/llm-multi-provider-protocols'
```

---

## 1. Wins

- [evidence: `src-tauri/src/agent/genai_model.rs` +246 行, `Cargo.toml` L96] 用 `genai` 0.6 一次打通 openai_chat / anthropic_messages / gemini_native / ollama_native 多协议，避免了逐协议自写 HTTP 客户端——proposal 的核心决策成立。
- [evidence: `scripts/gen_llm_presets.py` + `llm_presets_data.json` 610 行] CC Switch 76 项预设用脚本从上游快照生成而非手抄，可重复再生，上游对齐成本降为一次脚本运行。
- [evidence: `agent/llm.rs` 单测 `local_without_model_does_not_hit_cloud` 等] 「无模型不打云端」这一安全不变量被固化为自动化测试，不依赖手测。
- [evidence: tasks.md 14/14 + merge 后 cargo test 35/0] 全部任务完成且自动化验证干净，verify 的 3 个警告项（5.1 手测、commit、PASS 判定）在收尾时全部闭环。

## 2. Misses

- 🟡 [painful | evidence: verify.md §5「feature 分支相对 main 0 commits」初始状态] 实现完成时整个 feature 分支是一个未提交的工作区快照——worktree 流程中 commit 步骤被推迟到了 verify 之后，导致 verify 首跑拿不到 commit range、retro 一度无法量化。最终以单 commit `b59a273` 补齐。
- 🟡 [painful | evidence: verify.md §7 表格] 5.1 手动回归（UI 横幅、进度条渲染、真实 HTTP 对话）没有自动化等价物，长期靠人工 checklist；本 cycle 靠用户口头确认闭环。
- 📌 [nit | evidence: verify.md §4 漂移警告] `agent/llm.rs` 仍保留 reqwest 版 `resolve_backend` / `OpenAiCompatModel` 供单测（proposal Non-goals 明确允许），但「保留至多一个版本」的清理时点未定义。

## 3. Plan deviations

| Plan task | What changed | Why |
|-----------|--------------|-----|
| 1.2 预设快照「抽样比对上游」 | 增加了 `scripts/gen_llm_presets.py` 生成脚本而非纯手写快照 | 76 项手抄不可维护；脚本化让上游对齐可重复验证 |
| 4.3 「旧 OpenAI 客户端可编译或测试旁路」 | 保留了 reqwest 客户端作为单测旁路 | genai 适配器不便于在无网络单测中直接构造；符合 Non-goals 的最小保留 |
| 5.1 手动 checklist | 执行时点从 apply 期推迟到归档收尾期 | 手动 UI 回归需要用户本机环境（Ollama / 云端 Key），agent 无法代跑；由用户确认后补勾 |

## 4. Skill / workflow compliance

| Skill                                            | Used |
|--------------------------------------------------|------|
| superpowers:brainstorming                        | ✓（brainstorm.md 产物在位） |
| superpowers:writing-plans                        | ✓（plan.md 含 TDD Step 序列） |
| superpowers:using-git-worktrees                  | ✓（`.worktrees/llm-multi-provider-protocols`，已合并） |
| superpowers:subagent-driven-development          | ✗（见下） |
| (transitive) superpowers:test-driven-development | ✓（protocol 推断、ollama pull 解析等均有先测后实现记录） |
| (transitive) superpowers:requesting-code-review  | ✗（见下） |
| superpowers:finishing-a-development-branch       | ✓（merge commit `90bae44` 回 main） |

### Deliberately Skipped Skills

- **superpowers:subagent-driven-development**
  - **What was skipped**: 未以 subagent 逐 task 派发实现，apply 由单一 Cursor Agent 会话完成。
  - **Why this cycle**: 实现 commit `b59a273` 为单个原子 commit，无按 task 分批的中间产物；任务间共享 `LlmConfig`/`llm.rs` 类型演进，拆分派发的交接成本高于收益。
  - **How to prevent recurrence**: `scope-judgment rule` — 当 change 的 tasks 高度共享同一组核心类型（本例 `LlmConfig`/`resolve_backend`）且总 diff <~3k 行时，允许单会话实现；超过该规模或 task 边界清晰时应回到 subagent 派发。
- **superpowers:requesting-code-review**
  - **What was skipped**: merge 前无独立 code-review 会话记录。
  - **Why this cycle**: 验证以 verify.md 的 9 项检查 + cargo test/build 代替；观察到的触发条件是 verify §4 抽查全部对齐、无阻塞项，评审被默认吸收进 verify。
  - **How to prevent recurrence**: `schema graph fix` — 在 schema.yaml 的 verify artifact instruction 中，当 Overall Decision 首判为 PASS WITH WARNINGS 时（本例即如此），强制附带一次 code-review 记录或显式豁免理由，防止「verify 代替评审」成为默认。

## 5. Surprises

- 预设快照（610 行 JSON）让 diff 呈现净负行数（-4,442），初看像删代码，实为脚本再生成 + Cargo.lock 收缩——量化 retro 时不能只看 +/- 总量。
- verify 的 PASS 门槛被「实现未 commit」这类非代码质量因素阻塞：workflow 状态（commit/分支）而非实现本身成为收尾关键路径，这在事前规划中未预料。
- 引入 `genai` 后旧 reqwest 代码不是「删掉即可」——部分单测依赖其可离线构造的形态，保留旁路反而让测试矩阵更稳。

## 6. Promote candidates → long-term learning

- [ ] 🟡 **UI 层手动回归项应有归档前的显式闭环记录** → **Promote to schema**（verify.md 模板：凡 §7 标记「UI 层待手测」的行，归档前须附用户确认时间戳）
  > **Why**: 本 cycle 5.1 依赖用户口头确认，verify 首跑因此只能给 PASS WITH WARNINGS，收尾被拖到归档期。
  > **How to apply**: 写 verify.md §7 时即预约手测确认人与时点；归档命令执行前检查该项已填。
- [ ] 🟡 **feature worktree 的 commit 时点应在 verify 之前而非之后** → **Promote to project CLAUDE.md**（AGENTS.md「OpenSpec × Superpowers」段补充）
  > **Why**: 实现未 commit 导致 verify 无法记录 commit range，retro 量化与审计链断裂，需事后补齐。
  > **How to apply**: apply 阶段每完成一个 plan task 组即 commit；verify 运行前 worktree 必须无未 staged 实现（spec 文件除外）。
- [ ] 📌 **大生成物（预设快照/lock 文件）使 diff stat 失真，retro §0 应排除或单列** → **One-off**（本 retro 已单列说明）
  > **Why**: 仅影响本仓库这类「脚本生成静态快照」的模式，不构成通用规则。
  > **How to apply**: 下次涉及生成物快照的 change，在 §0 Evidence 直接采用本篇的「单列说明」写法。
- [ ] 📌 **引入新依赖后旧客户端代码可作离线测试旁路保留一个版本** → **Promote to memory**（type: feedback）
  > **Why**: `agent/llm.rs` reqwest 旁路让无网络单测可构造后端，直接删除反而破坏测试矩阵。
  > **How to apply**: 替换 HTTP 客户端类依赖时，先检查现有单测是否依赖旧实现的可构造性，再决定删除节奏（本例约定「至多一个版本」）。
