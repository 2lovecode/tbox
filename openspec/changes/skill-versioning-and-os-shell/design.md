# Design

## Context

See proposal.md — Why。现状：内置 Skill 经 `include_str!` 嵌入且不可改；用户 Skill 可 CRUD 无版本；Agent 注册表仅 `SideEffect::None`，循环内同步 `dispatch`，无审批通道。

## Goals / Non-Goals

**Goals:**
- 内置正文可经磁盘 overlay 覆盖，检索与设置页读同一解析路径。
- 每次保存追加版本文件，支持恢复默认 / 恢复指定 seq。
- `os.shell` 受限执行 + 审批闸门 + 会话同类放行 + 可配默认 cwd。
- 全部内置 Skill（含 `os.shell`）统一文档结构。

**Non-Goals:**
- 容器/seccomp 级沙箱；跨会话永久信任；外部 Skill 注册工具；工具箱独立 Shell 页面。

## Decisions

### 1. Skill overlay + 文件版本

- **选择**：`~/.toolbox/skills/overrides/{id}.md` 存内置当前覆盖；`skills/versions/{id}/{seq}.md` 追加历史；用户 Skill 仍在 `custom/`，保存前快照进 versions。
- **理由**：与现有 prefs/custom 文件模型一致，无需 SQLite 迁移。
- **替代**：SQLite 存版本 → 过重，拒绝。

### 2. Process 副作用 + 循环内 oneshot 审批

- **选择**：`SideEffect::{None, Process}`；`Process` 在 `dispatch` 前 emit `ToolApprovalRequired`，经 pending map + oneshot 等待 `resolve_tool_approval`；中断时 deny。
- **理由**：前端已有 `agent-event` 通道；阻塞单次工具调用即可，不必重构整条 async 流水线。
- **同类键**：命令首 token 的 basename。

### 3. 受限 shell 执行

- **选择**：Unix `sh -c` / Windows `cmd /C`；15s 超时；64KiB 截断；危险 basename 黑名单；不做路径 jail。
- **理由**：B+C 已锁定；审批展示完整 command+cwd。
- **命令探测**：对 `rg`/`grep`/`cat`/`head`/`find`/`ls` 等 `which`/`where`，缺省时在结果或 Skill 中提示替代。

### 4. 默认 cwd

- **选择**：prefs 字段 `shellDefaultCwd`（可空=用户 home）；`os.shell` 参数 `cwd` 可选覆盖。

## Risks / Trade-offs

- [受限 shell 仍可读写用户目录] → 审批 UI 明示 command/cwd；黑名单挡最高危；会话放行仅限 basename。
- [审批阻塞卡死循环] → 中断/超时 deny；前端必须监听 `ToolApprovalRequired`。
- [应用升级后内置 Skill 与 override 漂移] → 「恢复默认」一键回嵌入原文；版本历史保留用户改动。

## Migration Plan

- 无 DB 迁移；首次无 override 行为与现网一致。
- 新工具 `os.shell` 默认启用；禁用 Skill 仍不影响注册表存在性。
