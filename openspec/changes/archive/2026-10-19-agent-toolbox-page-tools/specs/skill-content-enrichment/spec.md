# Spec Delta

## ADDED Requirements

### Requirement: Skills Route Page Operations To Tool Ids
每个已注册页面操作的 Skill MUST 给出该操作对应的工具 id，并至少包含 1 条用户问法到 `<tool_call>` 的样本。Skill MUST NOT 再把这些操作写成「Agent 不能」或「请打开页面」。

未注册的界面或演示能力（屏幕标尺、网速模拟、视频模拟、未接通的 PDF、演示文件恢复）MUST 仍在 Skill 中说明其边界，且 MUST NOT 编造 tool_call。

#### Scenario: Escape sample calls json.escape
- **WHEN** JSON Skill 描述「把 JSON 转义」
- **THEN** 样本调用 `json.escape`，而不是 `json.format`

#### Scenario: Demo pages stay without a fake tool
- **WHEN** 用户询问网速测试或文件恢复
- **THEN** 对应 Skill 说明该能力在页面中的真实边界，且不包含 tool_call
