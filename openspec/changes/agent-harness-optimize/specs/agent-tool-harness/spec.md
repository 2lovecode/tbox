## ADDED Requirements

### Requirement: Eval Failure Attribution
离线评测（逻辑层与端到端层）SHALL 对失败（及可判定偏离的）轨迹标注**首错类别**，类别枚举至少包含：`intent`（该调未调 / 不该调却调）、`tool_choice`（选错工具）、`args`（参数非法）、`format`（可被容错解析恢复或未恢复的格式问题）、`loop`（同签名重复调用）、`answer_mismatch`（工具结果正确但最终自然语言与结果矛盾）。端到端报告 MUST 在既有四项指标之外输出各类别计数。golden case MAY 带可选字段声明期望首错类别（用于负例与前缀回归）；未声明时仅在失败路径上归类。

#### Scenario: E2E report includes attribution counts
- **WHEN** 本地已安装模型并运行端到端评测
- **THEN** 报告除意图命中率、工具选择正确率、参数合法率、端到端成功率外，还打印各类首错计数（可为 0）

#### Scenario: Logic layer classifies parse failure as format or args
- **WHEN** golden case 的 `model_output` 经解析后工具名合法但参数不符合 schema
- **THEN** 该 case 的首错类别为 `args`（而非笼统失败）

### Requirement: Trajectory Prefix Regression Cases
评测集 SHALL 包含按首错类别覆盖的轨迹前缀回归用例：每个已支持的首错类别 MUST 至少有 2 条 case（可用合成 `model_output` / 预期字段在逻辑层断言，不必整链 e2e）。前缀回归 MUST 可在无权重环境下随 `cargo test` 运行。

#### Scenario: Prefix cases cover all attribution classes
- **WHEN** 执行逻辑层评测相关测试
- **THEN** 断言每个首错类别至少被 2 条 case 覆盖

### Requirement: Model-Tier Harness Strategy Within Embedded
在 embedded 后端内，系统 SHALL 按启用模型的档位选择策略强度：目录标注为 Agent 推荐（≥1.5B Instruct 一类）的模型 MUST 使用完整小模型强化策略（四段式提示、修复预算 2）；标注为轻量（如 0.5B）的模型 MUST 仍走强化策略，但 MAY 使用更短的少样本与更紧的提示预算，且 MUST NOT 默认开启约束解码。云端 / Ollama 仍使用默认策略。策略选择 MUST NOT 改变可调用工具集合。

#### Scenario: Recommended embedded model gets full reinforced prompt
- **WHEN** 当前后端为 embedded 且启用模型为目录内 Agent 推荐项
- **THEN** 系统提示含完整工具摘要、检索 Skill 与少样本；修复预算为 2；约束解码默认关闭

#### Scenario: Lite embedded model keeps grammar off
- **WHEN** 当前后端为 embedded 且启用模型为目录内轻量项（0.5B）
- **THEN** 约束解码默认关闭；Agent 循环仍可用解析容错与修复重试完成回合

### Requirement: Catalog-Plus-Skill Prompt Layout
小模型强化策略的系统提示 SHALL 将「注册表工具逐工具单行中文摘要」作为常驻目录，将「检索到的 Skill 正文」按需注入（最多与既有检索 limit 一致）；MUST NOT 因 Skill 扩写而无上限膨胀。提示仍 MUST NOT 倾倒全部工具的 OpenAI JSON Schema。

#### Scenario: Tool directory always present
- **WHEN** 小模型策略渲染系统提示且 Skill 检索命中 0 条
- **THEN** 提示仍包含全部注册工具的单行摘要与少样本约束

#### Scenario: Skills injected only when retrieved
- **WHEN** 用户问题命中至多 3 条 Skill
- **THEN** 系统提示仅包含这些命中 Skill 的正文，不含未命中 Skill 全文

## MODIFIED Requirements

### Requirement: Pluggable Harness Strategy Layer
系统 SHALL 在 Agent 循环与模型之间提供统一的 harness 策略层：所有后端（embedded、ollama、云端）的对话回合 MUST 经由该层构建系统提示、解析模型输出、执行校验与修复重试。策略 MUST 可按后端与模型插拔：内置嵌入式模型 MUST 使用小模型强化策略族（含按模型档位区分的强化强度，见 Model-Tier Harness Strategy Within Embedded），其余后端 MUST 使用默认策略（简洁提示 + 相同的解析容错）。策略选择 MUST NOT 改变可调用工具集合（仍仅限注册表内 `side_effect=none` 工具）。

#### Scenario: Small model gets reinforced strategy
- **WHEN** 当前后端为 embedded 引擎且启用目录内本地 Instruct 模型
- **THEN** 系统提示由小模型强化策略族生成（含工具中文摘要与 `<tool_call>` 少样本示例），修复重试预算至少为 2（轻量档不得低于默认策略预算）

#### Scenario: Cloud backend uses default strategy
- **WHEN** 当前后端为已配置的云端提供者
- **THEN** 系统使用默认策略的简洁提示，但输出解析容错与 schema 校验与小模型策略一致

#### Scenario: Strategy does not extend tool registry
- **WHEN** 任一策略构建提示或解析输出
- **THEN** 可调用工具集合仍仅来自注册表，未注册工具 MUST 被拒绝执行

### Requirement: Reinforced Prompt For Small Models
小模型强化策略的系统提示 SHALL 为结构化多段式：角色与任务说明、注册表工具的逐工具单行中文摘要（常驻目录）、检索到的 Skill 正文（按需）、以及至少 2 个「用户请求 → `<tool_call>` → 工具结果 → 简短回答」少样本示例和 1 个无需工具直接回答的负例（轻量档 MAY 减少少样本条数但 MUST 保留至少 1 个正例与 1 个负例）。提示 MUST NOT 原样倾倒全部工具的 OpenAI JSON Schema。渲染后的系统提示 MUST 控制在当前档位上下文可承受范围内（目标字符/token 预算有断言监控）。

#### Scenario: Intent-to-tool guidance present
- **WHEN** 小模型策略渲染系统提示
- **THEN** 提示包含工具中文摘要、少样本示例与「只输出一个 `<tool_call>` 块、不得编造参数名」的显式约束

#### Scenario: Prompt size guarded
- **WHEN** 渲染完成的系统提示超过设定字符/token 预算
- **THEN** 评测/测试产出告警断言，提示构建不得无上限膨胀

### Requirement: Offline Evaluation Suite
系统 SHALL 内置离线评测集（golden cases：用户输入、预期工具与参数要点、预期输出要点，以及可选首错类别），覆盖全部已注册工具类别、中英文表达、以及无需工具的闲聊负例。评测 MUST 提供两层入口：纯逻辑层（解析/校验/容错/归因，无网络无权重，随 `cargo test` 必跑）与端到端层（加载已安装本地模型跑完整循环，权重缺失时跳过并提示）。端到端层 MUST 输出至少四项指标：意图命中率、工具选择正确率、参数合法率、端到端成功率，以及首错类别计数。评测集条目数 MUST ≥ 60 条；每个注册工具 MUST 至少被 2 条 case 命中（1 条中文问法 + 1 条英文/多样化问法为最低门槛）；评测集 MUST 包含至少 6 条多步调用 case 与至少 6 条无可用工具负例 case。

#### Scenario: Logic-layer eval runs in CI
- **WHEN** 执行 `cargo test`
- **THEN** 评测集纯逻辑层用例全部运行并对解析与校验行为断言；条目数 ≥ 60 且全部注册工具覆盖

#### Scenario: End-to-end report with model installed
- **WHEN** 本地已安装目录内 GGUF 模型且运行端到端评测
- **THEN** 输出意图命中率、工具选择正确率、参数合法率、端到端成功率报告，以及首错类别计数

#### Scenario: Skipped gracefully without weights
- **WHEN** 本地无已安装模型时运行端到端评测
- **THEN** 评测跳过并明确提示需要先下载模型，不报错失败

#### Scenario: Multi-step case covered
- **WHEN** 用户请求需要 ≥2 个工具串接（如「先 base64 解码再算哈希」）
- **THEN** 评测集中至少有 6 条 multi-step case 可经单测断言其预期工具链

#### Scenario: Negative case covered
- **WHEN** 用户请求属于「无可用工具」范畴（如天气、闲聊、计算器）
- **THEN** 评测集中至少有 6 条 negative case 期望无 Tool 调用
