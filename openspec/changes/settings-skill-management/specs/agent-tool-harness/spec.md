## ADDED Requirements

### Requirement: Disabled Skills Are Not Retrieved
Agent harness 检索内置 Skill 和用户 Skill 时 SHALL 跳过用户已禁用的 Skill；被禁用的 Skill 正文和元数据 MUST NOT 注入当前对话上下文。过滤 MUST 在评分或候选构建阶段完成，MUST NOT 只在前端隐藏结果。

#### Scenario: Disabled skill is skipped
- **WHEN** 用户请求本可命中某个已禁用 Skill 的工具任务
- **THEN** `retrieve_skills` 不返回该 Skill，系统提示也不包含其正文

#### Scenario: User skill participates without tool escalation
- **WHEN** 用户创建或导入一个关联已有工具的知识 Skill
- **THEN** 它可被检索并注入，但 Agent 可调用工具集合仍仅来自注册表

#### Scenario: Enabled skills continue retrieval
- **WHEN** 其他相关 Skill 保持启用
- **THEN** 它们仍按现有相关性排序和注入上限参与检索
