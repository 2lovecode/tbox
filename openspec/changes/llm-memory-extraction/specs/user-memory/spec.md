## MODIFIED Requirements

### Requirement: Conversation-End Memory Extraction
系统 SHALL 在会话满足触发条件时启动记忆提取：助手回合正常结束且本会话自上次抽取后至少新增一轮用户消息（实现上可与「本回合 user + 最终 assistant 文本」增量等价）；或会话被关闭/切换且存在未抽取增量。

提取 MUST **优先使用当前可用 LLM**（与主对话同一 LLM 配置），从本会话增量对话中抽取对未来有用的候选事实，并遵循选择性、抽象化、结构化。候选 MUST 为可解析的结构化条目（至少含 `text`；MAY 含稳定 `key`）。提取 MUST NOT 把工具原始大输出或整段轨迹原文写入记忆库。

当 LLM 不可用、调用失败、超时、或返回无法解析的内容时，系统 SHALL **回退到启发式抽取**（覆盖语言/emoji 等已知偏好短语），再进入既有写入消歧流水线。启发式 MUST NOT 替代「有可用 LLM 时」的优先路径。

提取与回退失败 MUST NOT 影响已完成对话的展示与持久化。写入仍 MUST 经写时 ADD/UPDATE/DELETE/NOOP 与密钥拒绝。

#### Scenario: Extract after useful preference stated
- **WHEN** 用户在会话中明确表示「请始终用中文简洁回答」且该回合正常结束触发抽取，且当前 LLM 可用
- **THEN** 记忆库出现对应偏好类条目（或对已有同类条目执行 UPDATE），且不把该轮工具结果原文存为记忆

#### Scenario: LLM extract yields structured candidates
- **WHEN** 用户陈述非启发式覆盖的稳定事实（如「我是后端工程师，回复请偏简洁」）且 LLM 返回合法候选 JSON
- **THEN** 系统将该候选交给写入消歧并入库（或 UPDATE），不依赖启发式短语表

#### Scenario: LLM failure falls back to heuristic
- **WHEN** 提取所用 LLM 调用失败或返回无法解析内容，且增量文本含启发式可识别偏好（如「用英文回答」）
- **THEN** 系统使用启发式候选完成 ingest；对话与轨迹保持不变

#### Scenario: Extraction failure is non-fatal
- **WHEN** LLM 与启发式均未产出候选，或 ingest 过程出错
- **THEN** 对话与轨迹保持不变；系统可记录失败日志或轻提示，MUST NOT 回滚已写入的消息

### Requirement: Memory Retrieval Into Context
新会话或新回合构建模型上下文时，系统 SHALL 检索最多 N 条（可配置，默认 ≤8）相关生效记忆并注入上下文（状态栏尾部或短 system 段）。注入占用的 token MUST 计入上下文预算的 `memory` 分项，且 MUST 受硬上限约束（默认不超过上下文窗口的 5% 或固定字符上限，以更严者为准）。

用户关闭「自动记忆」（`auto_memory_enabled=false`）时，系统 MUST NOT 自动抽取；**仍 SHALL 对已有生效记忆执行检索注入**（预算内）。用户 MUST 仍可通过管理 UI 删除/撤销记忆。

#### Scenario: Cross-session continuity
- **WHEN** 会话 A 已写入「中文简洁」偏好且用户新建会话 B 提问
- **THEN** 发往模型的上下文包含该偏好（在预算内），无需用户再次声明

#### Scenario: Memory budget capped
- **WHEN** 生效记忆很多且检索命中超过注入上限
- **THEN** 只注入预算内的高相关子集，不得撑破 memory 硬上限

#### Scenario: Auto memory disabled stops extract only
- **WHEN** 用户关闭自动记忆开关后完成新会话，且库中已有生效记忆
- **THEN** 系统 MUST NOT 对该会话执行自动抽取；发往模型的上下文仍包含预算内已有相关记忆

#### Scenario: Auto memory disabled with empty store
- **WHEN** 用户关闭自动记忆且无生效记忆
- **THEN** 不抽取、不注入记忆块
