# Agent Tool Harness

## Purpose

面向内置小型本地模型（Qwen2.5-0.5B/1.5B Instruct）的统一 Agent harness：可插拔策略层、提示工程、约束解码（可选）、容错解析、参数校验与修复重试（reask）、离线评测集。Harness 接入所有后端（embedded、ollama、云端）但按模型/后端可插拔策略档位，使 0.5B 模型也能稳定完成「理解意图 → 调对工具 → 给出预期输出」的闭环。

## Requirements

### Requirement: Pluggable Harness Strategy Layer
系统 SHALL 在 Agent 循环与模型之间提供统一的 harness 策略层：所有后端（embedded、ollama、云端）的对话回合 MUST 经由该层构建系统提示、解析模型输出、执行校验与修复重试。策略 MUST 可按后端与模型插拔：内置嵌入式小模型（0.5B/1.5B Instruct）MUST 使用小模型强化策略（结构化少样本提示、更高修复预算），其余后端 MUST 使用默认策略（简洁提示 + 相同的解析容错）。策略选择 MUST NOT 改变可调用工具集合（仍仅限注册表内 `side_effect=none` 工具）。

#### Scenario: Small model gets reinforced strategy
- **WHEN** 当前后端为 embedded 引擎且启用模型为目录内 0.5B/1.5B 小模型
- **THEN** 系统提示由小模型强化策略生成（含工具中文摘要与 `<tool_call>` 少样本示例），修复重试预算为 2

#### Scenario: Cloud backend uses default strategy
- **WHEN** 当前后端为已配置的云端提供者
- **THEN** 系统使用默认策略的简洁提示，但输出解析容错与 schema 校验与小模型策略一致

#### Scenario: Strategy does not extend tool registry
- **WHEN** 任一策略构建提示或解析输出
- **THEN** 可调用工具集合仍仅来自注册表，未注册工具 MUST 被拒绝执行

### Requirement: Reinforced Prompt For Small Models
小模型强化策略的系统提示 SHALL 为结构化四段式：角色与任务说明、检索到的 Skill 正文、注册表工具的逐工具单行中文摘要（名称、用途、参数名与必填性）、以及至少 2 个「用户请求 → `<tool_call>` → 工具结果 → 简短回答」少样本示例和 1 个无需工具直接回答的负例。提示 MUST NOT 原样倾倒全部工具的 OpenAI JSON Schema。渲染后的系统提示 MUST 控制在小模型上下文可承受范围内（目标 token 数有断言监控）。

#### Scenario: Intent-to-tool guidance present
- **WHEN** 小模型策略渲染系统提示
- **THEN** 提示包含工具中文摘要、少样本示例与「只输出一个 `<tool_call>` 块、不得编造参数名」的显式约束

#### Scenario: Prompt size guarded
- **WHEN** 渲染完成的系统提示超过设定 token 预算
- **THEN** 评测/测试产出告警断言，提示构建不得无上限膨胀

### Requirement: Constrained Decoding For Embedded Engine
嵌入式引擎 SHALL 支持回合级约束解码：harness 请求时，从注册表运行时生成 GBNF 语法（工具名枚举 + 合法 `<tool_call>` JSON 块结构），推理回合按该语法采样。约束解码 MUST 为增强项且可降级：当引擎依赖不支持、性能不可接受或小模型档默认策略决定关闭时，策略 MUST 能关闭约束解码并仅依赖解析容错与修复重试，功能不因此不可用。Ollama 与云端后端 MUST NOT 因缺少约束解码而被判不可用。

#### Scenario: Grammar-constrained tool call
- **WHEN** 小模型策略启用约束解码且模型进入工具调用回合
- **THEN** 生成的 `<tool_call>` 块结构合法：工具名属于注册表枚举，payload 为可解析 JSON

#### Scenario: Graceful degradation without grammar
- **WHEN** 约束解码被关闭或不可用
- **THEN** Agent 循环仍经解析容错与修复重试完成回合，不报「引擎不支持」类错误给用户

### Requirement: Tolerant Parsing And Schema Validation
harness 的解析层 SHALL 容错解析模型输出：剥离代码围栏、归一全角引号/冒号、容忍 `arguments` 为字符串化 JSON、剥离 `<tool_call>` 前后的引导语；无法解析为工具调用的输出 MUST 降级为纯文本回复而非报错。解析出的每个调用 MUST 经过校验：工具名不在注册表时 MUST 拒绝执行并回填含相近工具名建议的错误；参数 MUST 按该工具 JSON Schema 校验，不合格 MUST NOT 触发底层 command。

#### Scenario: Fenced and fullwidth output recovered
- **WHEN** 模型输出被 ``` 代码围栏包裹或含全角引号的 `<tool_call>`
- **THEN** 解析层归一后成功提取工具名与参数，回合并正常继续

#### Scenario: Near-miss tool name suggests alternative
- **WHEN** 模型请求了注册表中不存在的工具名（如拼写偏差）
- **THEN** 系统拒绝执行，回填错误中包含编辑距离相近的注册表工具名建议

### Requirement: Repair-and-reask Loop
当工具调用被拒绝（未注册、schema 校验失败）或工具执行失败时，系统 SHALL 把错误信息与正确参数说明回填给模型并重新请求，直至成功、模型改出合法调用、或修复预算耗尽；修复预算（小模型默认 2 次、默认策略 1 次）MUST 与既有工具回合上限合并核算，MUST NOT 叠加放大循环上限。预算耗尽时 MUST 按既有语义回填最终错误并由模型生成文字解释，应用不崩溃。

#### Scenario: Invalid arguments repaired on retry
- **WHEN** 模型首次调用参数缺必填字段，校验错误回填后模型第二次给出合法参数
- **THEN** 工具被执行，对话展示最终成功结果，总回合数仍在合并上限内

#### Scenario: Budget exhausted degrades to explanation
- **WHEN** 修复预算耗尽仍无合法调用
- **THEN** 对话展示失败说明（由模型基于错误文本生成或系统兜底），会话与消息保留，应用不崩溃

### Requirement: Offline Evaluation Suite
系统 SHALL 内置离线评测集（golden cases：用户输入、预期工具与参数要点、预期输出要点），覆盖全部已注册工具类别、中英文表达、以及无需工具的闲聊负例。评测 MUST 提供两层入口：纯逻辑层（解析/校验/容错，无网络无权重，随 `cargo test` 必跑）与端到端层（加载已安装本地模型跑完整循环，权重缺失时跳过并提示）。端到端层 MUST 输出至少四项指标：意图命中率、工具选择正确率、参数合法率、端到端成功率。评测集条目数 MUST ≥ 60 条；每个注册工具 MUST 至少被 2 条 case 命中（1 条中文问法 + 1 条英文/多样化问法为最低门槛）；评测集 MUST 包含至少 6 条多步调用 case 与至少 6 条无可用工具负例 case。

#### Scenario: Logic-layer eval runs in CI
- **WHEN** 执行 `cargo test`
- **THEN** 评测集纯逻辑层用例全部运行并对解析与校验行为断言；条目数 ≥ 60 且全部 16 工具覆盖

#### Scenario: End-to-end report with model installed
- **WHEN** 本地已安装目录内 GGUF 模型且运行端到端评测
- **THEN** 输出意图命中率、工具选择正确率、参数合法率、端到端成功率报告

#### Scenario: Skipped gracefully without weights
- **WHEN** 本地无已安装模型时运行端到端评测
- **THEN** 评测跳过并明确提示需要先下载模型，不报错失败

#### Scenario: Multi-step case covered
- **WHEN** 用户请求需要 ≥2 个工具串接（如「先 base64 解码再算哈希」）
- **THEN** 评测集中至少有 6 条 multi-step case 可经单测断言其预期工具链

#### Scenario: Negative case covered
- **WHEN** 用户请求属于「无可用工具」范畴（如天气、闲聊、计算器）
- **THEN** 评测集中至少有 6 条 negative case 期望无 Tool 调用

### Requirement: Context Budget Accounting
harness 在每次模型调用前 SHALL 估算上下文预算，分项至少包括：system、skills、tools、messages、tool_results、memory（若启用用户记忆）。上限来源：local/embedded 使用激活 profile 的 _ctx\（或引擎实际有效上下文）；云端使用 profile/提供商约定窗口或可配置上限。预算快照 MUST 经 Agent 事件或等价通道暴露给前端，并与压缩触发使用同一套数字。

#### Scenario: Local n_ctx is the limit
- **WHEN** 激活 local profile 的 n_ctx=8192
- **THEN** 预算上限按 8192（或引擎报告的有效上下文）计算，分项之和用于使用率

#### Scenario: Budget snapshot emitted each model call
- **WHEN** Agent 循环发起一次模型调用
- **THEN** 前端可收到含 used、limit（或 ratio）与分项的预算快照

### Requirement: Threshold Session Compression
当估算使用率达到配置阈值（默认 0.8）时，系统 SHALL 在两次模型调用之间对 Runtime 消息中未压缩的旧 tool results 做批量压缩或截断替换，并标记以防重复压缩。System 与核心工具定义前缀 MUST NOT 因压缩被改写。压缩 MUST 优先保留决策、约束、失败路径与可回源引用。连续全量压缩失败达到熔断次数后 MUST 停止重试并表面错误或降级，MUST NOT 死循环消耗调用。Audit/SQLite 中的原始轨迹 MUST 保持可恢复，与 Runtime 分离。

#### Scenario: Batch compress near capacity
- **WHEN** 使用率超过阈值且存在多条未压缩的大体积 tool results
- **THEN** 下一轮请求的 Runtime 中这些结果被摘要或截断替换，前缀 system/tools 保持稳定，原始结果仍可从 Audit 取回

#### Scenario: No per-turn thrashing
- **WHEN** 使用率远低于阈值
- **THEN** 系统 MUST NOT 每轮对历史做全量压缩

#### Scenario: Circuit breaker on compress failure
- **WHEN** 全量压缩连续失败达到熔断阈值
- **THEN** 停止继续压缩重试，并向用户或日志给出明确失败，不无限重试

