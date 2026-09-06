## Why

长会话多轮工具调用后，用户看不到上下文占用，也容易因窗口顶满或「上下文焦虑」导致追问失败；会话结束后稳定偏好无法跨会话保留。现在 Agent 循环与轨迹持久化已就绪，补齐用量可见、会话内压缩与自动记忆，是长对话可用性的下一层 Harness。

## What Changes

- **上下文用量展示**：Rust 侧估算分项预算（system / skills / tools / messages / tool_results / memory），经事件暴露；聊天 UI 展示已用/上限（或百分比），接近压缩阈值时轻提示。
- **单会话 Runtime 压缩**：使用率达到可配置阈值（默认约 80%）时，在两次模型调用之间批量压缩/截断旧 tool results；System 与核心工具定义前缀不动；Audit/SQLite 原始轨迹与 Runtime 分离，UI 可回看原文；全量压缩连续失败须熔断。
- **用户记忆模块（对话自动抽取 + 写入时 UPDATE）**：会话结束（或缓冲触发）后后台抽取候选事实；对相近旧记忆做 ADD / UPDATE / DELETE / NOOP；稳定偏好冲突优先 UPDATE，旧值进 revisions 可撤销；新会话检索 top-k 注入上下文并计入 memory 预算；设置页可列表/删除/撤销；密钥类不得入库。
- **Non-goals**：子 Agent 上下文隔离、Mem0 v3 式仅追加写入、双模型 Proposer/Reviewer PR 流水线、共享知识库 RAG、程序记忆自动挖掘、每条用户消息实时抽取、默认开启约束解码变更。

## Capabilities

### New Capabilities

- `user-memory`: 跨会话用户记忆的抽取、冲突决策（写入时 UPDATE）、持久化、检索注入与管理 UI

### Modified Capabilities

- `agent-chat`: 聊天 UI 展示上下文用量与压缩提示；Agent 循环可注入检索到的用户记忆；压缩不抹除 Audit 时间线
- `agent-tool-harness`: 上下文预算核算、阈值触发的会话内 Runtime 压缩与熔断

## Impact

- Rust：`agent/loop.rs`、`agent/harness/*`、`agent/trajectory.rs`（或等价）、新建 memory 模块与 SQLite migration、`commands/agent.rs` / 新 `commands/memory.rs`、Agent 事件载荷扩展
- 前端：聊天页用量条与压缩提示、设置中记忆管理入口（沿用现有设置壳）、类型与 store
- 配置：压缩阈值、memory 注入上限、自动记忆开关（可挂 LLM/通用设置）
- 验证：相关 `cargo test`、逻辑层用例（偏好 UPDATE）；前端 `vue-tsc`
