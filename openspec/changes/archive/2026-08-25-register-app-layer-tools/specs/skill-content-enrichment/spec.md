## ADDED Requirements

### Requirement: Skill Document Structure
全部 16 个注册工具的预置 Skill 文档 SHALL 保持 frontmatter（`tool_id` / `keywords`），body 由「能力描述 + 典型应用场景 + 用户问法 → 工具调用样本」三段组成。每个 Skill MUST 至少包含 2-3 个多样问法样本，覆盖中英文、不同意图表述（如「转成」「转换成」「帮我做」「how to」）与至少 1 个边界用例（特殊字符、空输入、嵌套对象等）。

#### Scenario: JSON Skill documents multiple phrasings
- **WHEN** `retrieve_skills` 按用户问题「把 JSON 转成 query string」检索
- **THEN** 命中 `json.to_query` Skill，其 body 包含至少 2 个不同问法样本与对应的 `<tool_call>` 块

#### Scenario: JWT Skill includes boundary case
- **WHEN** 用户提问包含空 payload、过期 exp、签名错误等边界
- **THEN** `jwt.parse` Skill body 含相应问法样本并说明预期错误文本

### Requirement: Skill Searchability Preserved
Skill 扩写 MUST NOT 引入新检索维度（如新的 frontmatter 字段）。`retrieve_skills` 仍按关键词 + 工具名 + 段（`.` 切分）匹配；扩写 MUST 控制在不显著拉高前 3 条结果的检索噪声。`limit=3` 的预算保持。

#### Scenario: Search still returns top-3 relevant skills
- **WHEN** 用户提问「JWT 解析」，`retrieve_skills(query, 3)`
- **THEN** 返回最多 3 条 Skill，最相关的 `jwt.parse` 在前位，未被无关工具 Skill 抢占

#### Scenario: Skill content growth does not break retrieval
- **WHEN** 同一工具的 Skill body 由 ~30 行扩到 ~80 行
- **THEN** 检索返回的 Skill 仍可被 `harness::prompt::build_small_prompt` 在预算内渲染完（不超出 `SMALL_PROMPT_CHAR_BUDGET`）