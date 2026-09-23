## ADDED Requirements

### Requirement: Sidebar Conversation Context Menu
系统 SHALL 在侧栏历史会话项上提供右键上下文菜单，至少包含「改名」「打开轨迹」「删除」三项；MUST NOT 在会话行上展示悬停删除按钮作为删除主入口。

#### Scenario: Open context menu
- **WHEN** 用户在侧栏某条已持久化会话上右键
- **THEN** 出现上下文菜单，含改名、打开轨迹、删除；浏览器默认右键菜单不出现

#### Scenario: Rename from menu
- **WHEN** 用户从上下文菜单选择改名并提交非空标题
- **THEN** 该会话标题被更新并锁定（与既有手动改名行为一致），侧栏展示新标题

#### Scenario: Open trajectory from menu
- **WHEN** 用户从上下文菜单选择打开轨迹
- **THEN** 应用导航至该会话的轨迹详情页（`/agent-runs/:id`）

#### Scenario: Delete from menu
- **WHEN** 用户从上下文菜单选择删除并确认
- **THEN** 该会话及其消息从存储与历史列表中移除

#### Scenario: Delete while viewing conversation or trajectory
- **WHEN** 用户删除当前正打开的会话（首页聊天或 `/agent-runs/:id` 轨迹详情）并确认
- **THEN** 应用导航至首页（新对话空态）；MUST NOT 停留在已删除会话的轨迹页

#### Scenario: No hover delete control
- **WHEN** 用户将指针悬停在侧栏会话行上
- **THEN** 不出现独立的悬停删除按钮
