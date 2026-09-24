## MODIFIED Requirements

### Requirement: Pluggable Harness Strategy Layer
系统 SHALL 在 Agent 循环与模型之间提供统一的 harness 策略层：所有后端（embedded、ollama、云端）的对话回合 MUST 经由该层构建系统提示、解析模型输出、执行校验与修复重试。策略 MUST 可按后端与模型插拔：内置嵌入式小模型（0.5B/1.5B Instruct）MUST 使用小模型强化策略（结构化少样本提示、更高修复预算），其余后端 MUST 使用默认策略（简洁提示 + 相同的解析容错）。可调用工具集合仍仅来自注册表；未注册工具 MUST 被拒绝。注册表可包含 `side_effect=None` 与 `side_effect=Process` 工具；`Process` 工具 MUST 在执行前通过用户审批闸门。

#### Scenario: Small model gets reinforced strategy
- **WHEN** 当前后端为 embedded 引擎且启用模型为目录内 0.5B/1.5B 小模型
- **THEN** 系统提示由小模型强化策略生成（含工具中文摘要与 `<tool_call>` 少样本示例），修复重试预算为 2

#### Scenario: Cloud backend uses default strategy
- **WHEN** 当前后端为已配置的云端提供者
- **THEN** 系统使用默认策略的简洁提示，但输出解析容错与 schema 校验与小模型策略一致

#### Scenario: Process tool requires approval
- **WHEN** 模型请求注册表中 `side_effect=Process` 的工具
- **THEN** 系统在执行前发出审批请求；未获允许 MUST NOT 启动进程

#### Scenario: Strategy does not extend tool registry
- **WHEN** 任一策略构建提示或解析输出
- **THEN** 可调用工具集合仍仅来自注册表，未注册工具 MUST 被拒绝执行

#### Scenario: Strategy does not invent tools
- **WHEN** 任一策略构建提示或解析输出
- **THEN** 可调用工具集合仍仅来自注册表，未注册工具 MUST 被拒绝执行

## ADDED Requirements

### Requirement: Tool Approval Events
系统 SHALL 通过 Agent 事件通道发出工具审批请求（含 requestId、toolId、command、cwd、similarKey），并接受前端回传的决策（deny / allow / allow_similar）。会话中断 MUST 使挂起审批按拒绝处理。

#### Scenario: Interrupted run denies pending approval
- **WHEN** 存在等待中的工具审批且用户中断 Agent 运行
- **THEN** 该审批按拒绝结算，不执行命令
