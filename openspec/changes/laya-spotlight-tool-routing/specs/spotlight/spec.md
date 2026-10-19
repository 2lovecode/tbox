# Spec Delta

## MODIFIED Requirements

### Requirement: Tool Search
Spotlight SHALL 能按名称/标签等检索已注册工具并导航到目标工具。当决策路由启用且服务命中时，Spotlight SHALL 将路由到的工具置顶展示并标注「智能匹配」；未启用、未命中或服务不可用时，Spotlight SHALL 保持现有关键词检索行为不变。

#### Scenario: Search finds a tool
- **WHEN** 用户输入与某工具名称匹配的关键词
- **THEN** 结果列表包含该工具，确认后进入对应页面

#### Scenario: Smart route pins a tool
- **WHEN** 决策路由已启用、服务可用，且用户输入自然语言「我想把 JSON 转成 YAML」并被判定命中某工具（置信度 ≥ 阈值）
- **THEN** 结果列表将该工具置顶并显示「智能匹配」标注，确认后进入对应页面

#### Scenario: Fallback on no route
- **WHEN** 决策路由未启用、未命中、超时或服务不可达
- **THEN** Spotlight 仅展示关键词检索结果，无「智能匹配」置顶项，且不出现错误提示
