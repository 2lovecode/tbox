## ADDED Requirements

### Requirement: OS Shell In Agent Registry
Agent 注册表 MUST 新增工具 `os.shell`：输入 `{"command":"<字符串>","cwd":"<可选字符串>"}`，`side_effect=Process`。该工具 MUST 拥有对应内置 Skill；非法参数 MUST 被 schema 校验拒绝。

#### Scenario: os.shell registered with process side effect
- **WHEN** 系统加载 Agent 注册表
- **THEN** 存在 id=`os.shell` 且副作用为 Process 的工具规格

#### Scenario: Missing command rejected
- **WHEN** Agent 调用 `os.shell` 但缺少 `command`
- **THEN** schema 校验失败，不进入审批与执行
