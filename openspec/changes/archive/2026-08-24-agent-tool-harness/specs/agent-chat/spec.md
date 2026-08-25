## MODIFIED Requirements

### Requirement: Agent Tool Loop
系统 SHALL 在 Rust 侧执行 Agent 循环：系统提示构建、输出解析、参数校验与修复重试由统一 harness 策略层承担（见 agent-tool-harness 能力）；策略按当前后端/模型插拔，所有后端 MUST 经由该层。循环将系统提示、检索到的 Skill、会话消息发给当前 LLM；若模型返回 tool call，则仅调度注册表中的工具并把结果回填；参数校验不合格或调用被拒绝时 MUST 在修复预算内回填错误并重试（reask），而不是直接终止回合。前端 MUST 以流式事件展示 token 与工具调用（名称、参数、结果或失败）。

#### Scenario: Single tool call then answer
- **WHEN** 用户发送需要 Base64 解码的请求且本地或云端 LLM 可用
- **THEN** 对话中出现对应工具调用记录与结果，并跟有助手对结果的说明

#### Scenario: Sequential tool calls
- **WHEN** 用户请求先 Base64 解码再计算哈希，且两个工具均已注册
- **THEN** 系统按循环依次执行这两个工具，并在同一轮对话中展示两次调用与最终回复

#### Scenario: Invalid tool call repaired via reask
- **WHEN** 模型返回的工具调用参数不符合 JSON Schema
- **THEN** 系统不调用底层 command，将校验错误回填给模型并重新请求；修复预算内得到合法调用则继续执行，预算耗尽则按错误语义收尾

#### Scenario: Cancel in-flight turn
- **WHEN** 用户在 Agent 循环尚未完成时取消发送
- **THEN** 循环停止；已持久化的消息保留；进行中的助手输出标记为中断而非删除整条会话
