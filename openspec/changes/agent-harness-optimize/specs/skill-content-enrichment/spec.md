## ADDED Requirements

### Requirement: Skill Routing Conditions And Anti-Examples
每个预置 Skill 的正文 SHALL 在「能力描述」或专用小节中写明**何时使用**与**何时不使用**（至少 1 条反例/易混淆边界，例如近邻工具对比）。frontmatter `keywords` MUST 优先服务路由（典型问法与边界词），而非仅罗列功能同义词。

#### Scenario: Near-neighbor anti-example present
- **WHEN** 读取 `json.to_query` 与 `json.flatten` 的 Skill 正文
- **THEN** 各自含至少 1 条说明「何种问法应选本工具而非另一工具」的边界或反例

### Requirement: Retrieval Downranks Wrong Neighbor
`retrieve_skills` SHALL 在简单关键词匹配基础上，对命中「明确指向其他工具」的问法避免把易混淆 Skill 排在第一位（例如「平铺嵌套 JSON」不得把 `json.to_query` 排在 `json.flatten` 之前）。检索 limit 与「不扩展可执行工具集」约束保持不变。

#### Scenario: Flatten query ranks flatten skill first
- **WHEN** `retrieve_skills("把嵌套 JSON 平铺开", 3)`
- **THEN** 返回列表首位为 `json.flatten`（若命中非空）

## MODIFIED Requirements

### Requirement: Skill Document Structure
全部注册工具的预置 Skill 文档 SHALL 保持 frontmatter（`tool_id` / `keywords`），body 由「能力描述（含何时用/何时不用）+ 典型应用场景 + 用户问法 → 工具调用样本」组成。每个 Skill MUST 至少包含 2-3 个多样问法样本，覆盖中英文、不同意图表述与至少 1 个边界用例（特殊字符、空输入、嵌套对象或近邻工具反例等）。

#### Scenario: JSON Skill documents multiple phrasings
- **WHEN** `retrieve_skills` 按用户问题「把 JSON 转成 query string」检索
- **THEN** 命中 `json.to_query` Skill，其 body 包含至少 2 个不同问法样本与对应的 `<tool_call>` 块

#### Scenario: JWT Skill includes boundary case
- **WHEN** 用户提问包含空 payload、过期 exp、签名错误等边界
- **THEN** `jwt.parse` Skill body 含相应问法样本并说明预期错误文本
