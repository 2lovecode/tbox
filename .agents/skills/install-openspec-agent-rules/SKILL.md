---
name: install-openspec-agent-rules
description: Use when scaffolding or refreshing OpenSpec app-level agent rules (AGENTS.md / CLAUDE.md), installing agent policy into a repo, or when the user runs /opsx:init-rules or asks to generate project agent rules without committing .cursor/.
---

# Install OpenSpec Agent Rules

把 **应用级 Agent 规则**写入当前项目根目录（进 git）。`.cursor/` 永不提交；可选生成本机压缩规则。

## 产出（进 git）

| 文件 | 作用 |
|------|------|
| `AGENTS.md` | 跨 Agent 应用级规则（真相） |
| `CLAUDE.md` | Claude Code 薄入口 → AGENTS.md |

## 产出（本机，不进 git）

| 文件 | 作用 |
|------|------|
| `.cursor/rules/openspec.mdc` | Cursor alwaysApply 压缩版 |
| `.gitignore` 中的 `.cursor/` | 确保整目录忽略 |

## 何时用

- 新仓库要装 OpenSpec 协作规则
- 刷新/对齐已有 `AGENTS.md` 模板
- 用户说：安装应用级规则、生成 AGENTS.md、`/opsx:init-rules`

## 步骤

1. **确认项目根**  
   在含（或将含）`openspec/` 的仓库根执行。若无 OpenSpec，先提醒用户跑 `openspec init`（本 skill 不替代 OpenSpec CLI 初始化）。

2. **收集变量**（缺一则问用户；已有 AGENTS.md 时可从中推断）  
   - `PRODUCT_NAME` — 产品名（如 TBox）  
   - `PRODUCT_BLURB` — 一句定位  
   - `STACK` — 技术栈一句  
   - `NEW_FEATURE_PATHS` — 新能力常见落点一句  
   - `LIVING_SPECS_TABLE` — Living specs 表体（Markdown 行；没有则用占位一行）

3. **执行安装脚本**（优先，确定性更高）

```bash
node .agents/skills/install-openspec-agent-rules/scripts/install.mjs \
  --product "PRODUCT_NAME" \
  --blurb "PRODUCT_BLURB" \
  --stack "STACK" \
  --paths "NEW_FEATURE_PATHS" \
  --specs-table "LIVING_SPECS_TABLE"
```

可选：

- `--force` — 覆盖已有 `AGENTS.md` / `CLAUDE.md`
- `--no-cursor-rule` — 不写 `.cursor/rules/openspec.mdc`
- `--root <path>` — 指定项目根（默认 cwd）

若脚本路径不存在（skill 装在 `~/.cursor/skills/...`），改用 skill 目录内同名脚本，并把 `--root` 指到目标仓库。

4. **校验**  
   - 根目录存在 `AGENTS.md`、`CLAUDE.md`  
   - `.gitignore` 含独立一行 `.cursor/`（或等价忽略）  
   - 未把 `.cursor/` 加入 git

5. **向用户汇报** 写了哪些文件；提醒：改政策只改 `AGENTS.md`，再重跑本 skill（或手改本机 `.cursor/rules/openspec.mdc`）。

## 安装本 skill 到项目

把本目录复制为：

```text
<repo>/.agents/skills/install-openspec-agent-rules/
```

Claude Code 可再复制一份到 `.claude/skills/install-openspec-agent-rules/`，并使用 `.claude/commands/opsx/init-rules.md`。

## 不要做

- 不要把 `.cursor/` 提交进 git  
- 不要写成 Cursor **User Rules**（跨项目误伤）  
- 不要在 `docs/superpowers/` 另建长期规格
