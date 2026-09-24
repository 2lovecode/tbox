# Proposal

## Why

内置 Skill 质量参差、不可编辑，且缺少版本回滚；Agent 也无法在受控前提下调用本机只读/检索类命令。需要统一 Skill 生命周期，并引入需用户确认的受限 OS shell，让本地开发者工作流闭环。

## What Changes

- 按标准三段式结构重写全部内置 Skill，并新增 `os.shell` Skill。
- 内置与用户/导入 Skill 均可在设置页编辑；每次保存记录版本；可恢复默认（仅内置）或回滚到任一历史版本。
- 注册表新增 `os.shell` 工具（`SideEffect::Process`）：受限 shell 执行、超时与输出截断、危险命令黑名单、跨平台命令可用性探测与替代提示。
- `Process` 类工具执行前必须用户确认；支持「本会话允许同类命令」；默认 cwd 可配置，单次调用可覆盖。
- **BREAKING（契约扩展）**：Agent 工具副作用从仅 `None` 扩展为 `None | Process`；`Process` 必须经审批闸门。

## Capabilities

### New Capabilities

- `os-shell`: 受限 OS 命令执行、命令探测与替代、用户审批与会话级同类放行、可配默认 cwd。
- `settings-skills`: 设置页 Skill 管理的编辑、版本历史、恢复默认/恢复版本（从已归档 change 提升为长期 capability；本 change 补齐版本化需求）。

### Modified Capabilities

- `skill-content-enrichment`: 内置 Skill 覆盖全部注册工具（含 `os.shell`），统一文档结构与检索约束。
- `agent-tool-harness`: 允许注册 `Process` 副作用工具；dispatch 前审批；事件与中断语义。
- `app-layer-tools-registered`: 注册表工具集合增加 `os.shell`。

## Impact

- 前端：`SkillSettingsPanel`（编辑内置、版本历史）、`HomePage`/Agent 事件（审批条）、通用设置（Shell 默认 cwd）。
- Rust：`agent::skills`（override/versions）、`agent::registry`（`SideEffect::Process` + `os.shell`）、`agent::loop`（审批等待）、新 `os_shell` 执行模块、Tauri commands。
- Non-goals：外部 Skill 不注册新工具；不做容器级沙箱；不做跨会话永久信任；不新增工具箱独立页面。
