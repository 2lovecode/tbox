## MODIFIED Requirements

### Requirement: Reinforced Prompt For Small Models
小模型强化策略的系统提示 SHALL 为结构化四段式：角色与任务说明、检索到的 Skill 正文、工具的逐工具单行中文摘要（名称、用途、参数名与必填性）、以及至少 2 个「用户请求 → `<tool_call>` → 工具结果 → 简短回答」少样本示例和 1 个无需工具直接回答的负例。当本轮用户文本检索到技能且这些技能登记了工具 id 时，摘要 MUST 只包含这些已注册工具；未检索到技能时 MUST 列出全部已注册工具。提示 MUST NOT 原样倾倒全部工具的 OpenAI JSON Schema。渲染后的系统提示 MUST 控制在小模型上下文可承受范围内（目标 token 数有断言监控）。`json.format` 的参数说明 MUST 只描述美化，MUST NOT 把反转义写成它的用途。

#### Scenario: Intent-to-tool guidance present
- **WHEN** 小模型策略渲染系统提示
- **THEN** 提示包含工具中文摘要、少样本示例与「只输出一个 `<tool_call>` 块、不得编造参数名」的显式约束

#### Scenario: Prompt size guarded
- **WHEN** 渲染完成的系统提示超过设定 token 预算
- **THEN** 评测/测试产出告警断言，提示构建不得无上限膨胀

#### Scenario: Retrieved skills narrow the tool directory
- **WHEN** 用户文本检索命中登记了 `json.escape` 的 JSON 技能
- **THEN** 小模型系统提示的工具目录包含 `json.escape`，且不包含与该次检索无关的工具（例如未同时命中编码技能时不包含 `base64.encode`）

### Requirement: Threshold Session Compression
当估算使用率达到配置阈值（默认 0.8）时，系统 SHALL 在两次模型调用之间对 Runtime 消息中未压缩的旧 tool results 做批量压缩或截断替换，并标记以防重复压缩。System 与核心工具定义前缀 MUST NOT 因压缩被改写。压缩 MUST 优先保留决策、约束、失败路径与可回源引用。当策略落到重置时，Runtime MUST 保留本轮用户原文，以及最近一对助手工具调用与其工具结果；单条结果过长时 MAY 只保留开头并注明不要用相同参数再次调用。连续全量压缩失败达到熔断次数后 MUST 停止重试并表面错误或降级，MUST NOT 死循环消耗调用。Audit/SQLite 中的原始轨迹 MUST 保持可恢复，与 Runtime 分离。

#### Scenario: Batch compress near capacity
- **WHEN** 使用率超过阈值且存在多条未压缩的大体积 tool results
- **THEN** 下一轮请求的 Runtime 中这些结果被摘要或截断替换，前缀 system/tools 保持稳定，原始结果仍可从 Audit 取回

#### Scenario: No per-turn thrashing
- **WHEN** 使用率远低于阈值
- **THEN** 系统 MUST NOT 每轮对历史做全量压缩

#### Scenario: Circuit breaker on compress failure
- **WHEN** 全量压缩连续失败达到熔断阈值
- **THEN** 停止继续压缩重试，并向用户或日志给出明确失败，不无限重试

#### Scenario: Reset keeps the latest tool exchange
- **WHEN** 使用率超过阈值且重置策略生效，而消息尾部已有一次工具调用及其结果
- **THEN** 重置后的 Runtime 仍包含本轮用户原文、该次工具调用和该次结果（或结果开头），模型不必在看不到结果的情况下重发同一调用

## ADDED Requirements

### Requirement: Identical Tool Calls Deduped Per Turn
同一模型回合解析出的多个工具调用中，工具名与参数 JSON 完全相同的调用 MUST 只执行一次。保留第一次出现的调用标识。

#### Scenario: Duplicate query conversion runs once
- **WHEN** 模型在一次回复里给出两个参数相同的 `json.to_query`
- **THEN** 该工具只执行一次，对话里只有一条对应结果

### Requirement: JSON Escape Rejects Extra Slashes Clearly
`json.escape` SHALL 在输入已是合法 JSON 时按该原文转义。输入不是合法 JSON 但剥去多余反斜杠或弯引号后可解析时，SHALL 对解析出的值转义并成功返回。仍然无法解析时，返回的错误文本 MUST 包含「不要再多写反斜杠」，以便模型用用户原文重试。

#### Scenario: Extra slash layer still escapes
- **WHEN** 调用 `json.escape`，input 为已带一层反斜杠的对象文本（形如 `{\"a\":1}`）
- **THEN** 工具成功返回对 `{"a":1}` 的一层转义，而不是 `key must be a string`

#### Scenario: Unparseable input names the retry
- **WHEN** 调用 `json.escape`，input 在剥层和弯引号归一后仍不是 JSON
- **THEN** 错误文本包含「不要再多写反斜杠」
