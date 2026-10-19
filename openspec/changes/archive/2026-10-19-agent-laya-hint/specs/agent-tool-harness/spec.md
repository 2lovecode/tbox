## ADDED Requirements

### Requirement: Laya Hint Is Advisory
当 Laya 模式不是关闭，且对本轮决策状态的 choice 置信度达到配置阈值时，Agent SHALL 在发往模型的系统提示中加入一条参考：工具箱页面名称与置信度，并说明若与用户原文不符则按原文选择工具和参数。该页面对应的已注册 agent 工具 MUST 排在工具目录中这些工具的其余条目前面。目录中已有的其他工具 MUST 保留。系统 MUST NOT 因该参考直接派发工具，MUST NOT 改写用户原文。模式关闭、推理失败或置信度低于阈值时 MUST NOT 加入该参考。

#### Scenario: High confidence hint does not replace the user text
- **WHEN** Laya 对压短后的决策状态给出高于阈值的页面命中
- **THEN** 系统提示含该页面名和置信度，并写明按用户原文选择工具和参数；发给模型的用户消息仍是原文

#### Scenario: Disabled or low confidence leaves the prompt unchanged
- **WHEN** Laya 模式为关闭，或置信度低于阈值
- **THEN** 系统提示不含本地决策参考，工具选择仍由模型按技能目录完成

### Requirement: Decision State Drops Payloads
发给 Laya 的 state MUST 去掉 JSON 载荷，保留意图短句。用户原文不超过 40 个字符且历史中有工具结果时，state MUST 附上最近一条工具名及其结果首行。state MUST 短于完整用户粘贴。

#### Scenario: Escaped JSON plus a verb
- **WHEN** 用户消息是一大段 JSON 加上「转义」或「去转义」
- **THEN** 决策状态含该动词，且不含整段 JSON 正文
