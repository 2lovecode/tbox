# User Memory

## Purpose

跨会话用户记忆：从对话自动抽取稳定偏好与事实，写入时消歧（UPDATE 优先），并在新会话按预算检索注入。

## Requirements

### Requirement: Conversation-End Memory Extraction
系统 SHALL 在会话满足触发条件时启动后台记忆提取：助手回合正常结束且本会话自上次抽取后至少新增一轮用户消息；或会话被关闭/切换且存在未抽取增量。提取 MUST 使用当前可用 LLM，从本会话增量对话中抽取对未来有用的候选事实，并遵循选择性、抽象化、结构化。提取 MUST NOT 把工具原始大输出或整段轨迹原文写入记忆库。提取失败 MUST NOT 影响已完成对话的展示与持久化。

#### Scenario: Extract after useful preference stated
- **WHEN** 用户在会话中明确表示「请始终用中文简洁回答」且该回合正常结束触发抽取
- **THEN** 记忆库出现对应偏好类条目（或对已有同类条目执行 UPDATE），且不把该轮工具结果原文存为记忆

#### Scenario: Extraction failure is non-fatal
- **WHEN** 提取所用 LLM 调用失败
- **THEN** 对话与轨迹保持不变；系统可记录失败日志或轻提示，MUST NOT 回滚已写入的消息

### Requirement: Write-Time Conflict Decision
对每条候选事实，系统 SHALL 先检索相近的已有记忆，再判定 **ADD / UPDATE / DELETE / NOOP** 之一并执行。语义冲突的稳定属性（如语言偏好、输出风格）MUST 优先 **UPDATE** 覆盖当前生效值，并写入 `updated_at` 与证据引用（来源会话 id 与短摘录）。UPDATE 前的旧值 MUST 保留在 revisions（或等价旁表）以支持撤销。密钥、token、密码类候选 MUST 被拒绝入库（NOOP 或硬拒绝）。

#### Scenario: Preference update replaces prior value
- **WHEN** 记忆库已有「偏好中文」，用户在新会话中说「以后请用英文回答」并完成抽取
- **THEN** 生效记忆更新为英文偏好（UPDATE）；旧中文值仅存在于 revisions，新会话注入 MUST NOT 再以中文偏好为主

#### Scenario: Secrets never stored
- **WHEN** 用户消息包含 API Key 或明显密钥形态文本并触发抽取
- **THEN** 该候选不得写入生效记忆表

#### Scenario: Unrelated fact is ADD
- **WHEN** 候选与现有记忆均不相似且通过敏感检查
- **THEN** 系统执行 ADD 创建新条目

### Requirement: Memory Retrieval Into Context
新会话或新回合构建模型上下文时，系统 SHALL 检索最多 N 条（可配置，默认 ≤8）相关生效记忆并注入上下文（状态栏尾部或短 system 段）。注入占用的 token MUST 计入上下文预算的 `memory` 分项，且 MUST 受硬上限约束（默认不超过上下文窗口的 5% 或固定字符上限，以更严者为准）。用户关闭「自动记忆」时 MUST NOT 自动抽取，MAY 仍允许手动管理已有记忆。

#### Scenario: Cross-session continuity
- **WHEN** 会话 A 已写入「中文简洁」偏好且用户新建会话 B 提问
- **THEN** 发往模型的上下文包含该偏好（在预算内），无需用户再次声明

#### Scenario: Memory budget capped
- **WHEN** 生效记忆很多且检索命中超过注入上限
- **THEN** 只注入预算内的高相关子集，不得撑破 memory 硬上限

#### Scenario: Auto memory disabled
- **WHEN** 用户关闭自动记忆开关后完成新会话
- **THEN** 系统 MUST NOT 对该会话执行自动抽取

### Requirement: Memory Management UI
设置（或等价管理入口）SHALL 列出生效记忆，支持删除与撤销最近一次 UPDATE（从 revisions 恢复）。列表 MUST 展示更新时间；MAY 展示来源会话。删除 MUST 为软删或等价不可再注入状态。

#### Scenario: User deletes a memory
- **WHEN** 用户在管理界面删除一条记忆
- **THEN** 后续对话不再注入该条

#### Scenario: Undo last update
- **WHEN** 用户对一条曾被 UPDATE 的记忆执行撤销
- **THEN** 生效值恢复为 revisions 中的上一版
