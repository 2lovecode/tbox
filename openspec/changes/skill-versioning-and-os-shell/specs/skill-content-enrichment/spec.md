## MODIFIED Requirements

### Requirement: Skill Document Structure
预置 Skill SHALL 采用渐进式披露：L0 为精确的 name + 「何时使用」短描述（常驻目录）；L1 为完整说明书（检索命中后才注入）。frontmatter 可含多个 `tool_id` 与多个 `toolbox_id`——**不必**与工具一一对应；合并后的 Skill 数量 SHOULD 明显少于工具总数以控制 L0 上下文。body MUST 写清子工具/页面的选用边界与 ≥2 问法样本。覆盖约束：(1) 工具箱 id 1..=36 均出现在某 Skill 的 `toolbox_id`；(2) Agent 注册表每个工具 id 均出现在某 Skill 的 `tool_id`。

#### Scenario: One skill covers multiple agent tools
- **WHEN** 系统加载内置 Skill
- **THEN** 存在至少一个 Skill 的 `tool_id` 列表长度 ≥2，且 L0 目录条目数少于 Agent 工具数

#### Scenario: JSON family skill retrieved for flatten intent
- **WHEN** `retrieve_skills` 按「把嵌套 JSON 平铺」检索
- **THEN** 命中合并后的 JSON Skill，其 `tool_id` 列表含 `json.flatten`

#### Scenario: All toolbox tools have skills
- **WHEN** 系统加载全部内置 Skill
- **THEN** 工具箱 id 1 到 36 均至少出现在一个内置 Skill 的 `toolbox_id` 列表中

#### Scenario: L0 description is precise when-to-use
- **WHEN** 渲染 L0 目录
- **THEN** 每条 description 来自「何时使用」要点，而非冗长正文首段

## ADDED Requirements

### Requirement: Skill Searchability With Overrides
`retrieve_skills` MUST 对内置 Skill 优先使用本机覆盖正文（若存在），否则使用嵌入原文。扩写与覆盖 MUST NOT 引入新检索维度；`limit=3` 预算保持。

#### Scenario: Override body used in retrieval
- **WHEN** 某内置 Skill 存在覆盖且用户问题命中其关键词
- **THEN** 注入上下文的正文为覆盖内容而非嵌入原文
