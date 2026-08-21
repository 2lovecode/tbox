---
name: "OPSX: Init Rules"
description: "Install OpenSpec app-level agent rules (AGENTS.md / CLAUDE.md) into the current project"
category: "Workflow"
tags: ["workflow", "agents", "openspec"]
---

Install **应用级 Agent 规则** into this repo. Follow the skill **install-openspec-agent-rules** exactly.

**Do not** commit `.cursor/`. **Do not** add these as Cursor User Rules.

**Steps**

1. Read `.agents/skills/install-openspec-agent-rules/SKILL.md` (or `.claude/skills/install-openspec-agent-rules/SKILL.md`).
2. Collect `PRODUCT_NAME` / `PRODUCT_BLURB` / `STACK` / `NEW_FEATURE_PATHS` / optional living-specs table (ask if missing).
3. Run the install script from the skill, with `--force` only if the user confirms overwriting existing files.
4. Report which files were written.

Example:

```bash
node .agents/skills/install-openspec-agent-rules/scripts/install.mjs \
  --product "MyApp" \
  --blurb "一句话产品定位" \
  --stack "技术栈一句" \
  --paths "新能力常见落点一句"
```
