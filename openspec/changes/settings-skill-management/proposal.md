## Why

内置工具会对应生成预置 Skill 说明书，并参与 Agent 上下文检索；当前用户无法查看这些 Skill，也无法禁用不希望被模型检索到的项。Skill 数量继续增长后，缺少管理入口会让上下文暴露和工具路由难以控制。

知识库依据：Skills 适合以「薄目录 + 按需查阅」组织，只向模型暴露需要的说明书，避免全量注入；工具数量多时应分层管理并缩小候选（《深入理解 AI Agent》ch04「Skills：把工具发现变成按需查阅」「层次化组织与按需加载」）。

## What Changes

- 设置页新增「Skill 管理」分区，列出全部内置 Skill 的名称、关联工具、用途和启用状态。
- 为全部 Agent 注册工具提供内置 Skill；当前 16 个工具由 16 个内置 Skill 覆盖。
- 支持在设置页创建和修改用户 Skill，用户 Skill 参与检索与说明注入，但 MUST NOT 注册新的可执行工具。
- 支持导入外部 Markdown Skill 文件；导入后成为用户 Skill，仍受同一安全边界约束。
- 用户可启用或禁用单个内置 Skill；状态持久化到本机配置，重启后保留。
- Agent Skill 检索 MUST 跳过禁用项；禁用只影响 Agent 说明书注入，MUST NOT 删除或注销工具箱工具。
- 设置页保留现有 LLM、记忆、通用、关于分区，并新增 Skill 分区导航。

## Capabilities

### New Capabilities

- `settings-skills`: 设置页 Skill 管理分区的展示、启停和持久化行为。

### Modified Capabilities

- `agent-tool-harness`: 检索与注入内置 Skill 和用户 Skill 时遵守统一启停配置；外部 Skill 不扩展工具注册表。

## Impact

- 前端：设置页导航、`SettingsSkillsPanel`、Skill 编辑对话框、设置 store。
- Rust：`agent::skills` 元数据、用户 Skill 存储、外部导入校验、检索过滤、Tauri command 注册。
- Non-goals：外部 Skill 不注册新工具；不支持执行 Skill 内脚本；不支持目录批量导入；不禁用工具箱页面中的工具本身；不改内置工具注册表和白名单。
