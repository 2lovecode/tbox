## Purpose

为 Agent 提供受控的操作系统命令执行能力：跨平台探测与替代、危险命令拦截、用户审批与会话级同类放行。

## ADDED Requirements

### Requirement: Restricted OS Shell Tool
系统 SHALL 注册 Agent 工具 `os.shell`，参数为必填 `command` 与可选 `cwd`。执行 MUST 使用平台 shell（Unix `sh -c` / Windows `cmd /C`），MUST 施加超时与合并输出截断，MUST 拒绝危险可执行文件 basename 黑名单中的命令。缺省 `cwd` MUST 使用设置中的 Shell 默认工作目录；若未配置则使用用户主目录。

#### Scenario: Run safe listing command after approval
- **WHEN** 用户批准 `os.shell` 且 command 为安全的 `ls`（或 Windows 等价）
- **THEN** 工具返回该 cwd 下的命令输出（或等价错误文本），且输出长度不超过截断上限

#### Scenario: Dangerous command rejected without running
- **WHEN** 模型请求 `os.shell` 且 command 的首 token basename 为 `rm`（或黑名单项）
- **THEN** 系统拒绝执行并返回错误说明，不启动进程

### Requirement: User Approval Gate
`os.shell`（及一切 `Process` 副作用工具）在真正执行前 MUST 请求用户确认。确认 UI MUST 展示 toolId、完整 command、cwd 与三类决策：拒绝、允许一次、本会话允许同类。同类 MUST 按命令首 token 的 basename 判定。用户拒绝或会话中断时 MUST 将错误结果回填给模型且 MUST NOT 执行命令。

#### Scenario: Allow once executes then asks again
- **WHEN** 用户选择「允许一次」后模型再次调用同一 basename
- **THEN** 系统再次弹出确认

#### Scenario: Allow similar skips later prompts in session
- **WHEN** 用户选择「本会话允许同类」后同会话再次调用相同 basename
- **THEN** 不再弹确认直接进入受限执行

#### Scenario: Deny returns error to model
- **WHEN** 用户拒绝某次 `os.shell`
- **THEN** 工具结果为拒绝说明，进程未启动

### Requirement: Command Availability And Alternatives
系统 SHALL 探测常用检索/阅读命令在当前 OS 上的可用性（至少覆盖 `rg`、`grep`、`cat`、`head`、`ls`/`dir` 等），并在内置 `os.shell` Skill 或工具辅助信息中说明缺失时的替代命令。探测失败 MUST NOT 阻止工具注册。

#### Scenario: Missing rg suggests grep
- **WHEN** 当前系统无 `rg` 但有 `grep`
- **THEN** Skill 或探测结果提示可用 `grep -R`（或平台等价）作为替代

### Requirement: Configurable Default Shell Cwd
设置 SHALL 提供 Shell 默认工作目录配置项。`os.shell` 未传 `cwd` 时 MUST 使用该配置；传入 `cwd` 时 MUST 覆盖默认值。

#### Scenario: Per-call cwd overrides setting
- **WHEN** 设置默认 cwd 为 A 且调用参数 cwd 为 B
- **THEN** 命令在 B 下执行
