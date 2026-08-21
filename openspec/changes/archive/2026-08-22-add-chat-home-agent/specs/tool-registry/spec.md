## MODIFIED Requirements

### Requirement: Frontend Route Mapping
每个工具页面 SHALL 可通过应用路由访问，且工具箱 / 搜索入口的工具 id 能导航到对应页面。

#### Scenario: Navigate from home by tool id
- **WHEN** 用户在工具箱（承接原首页网格）点击已注册工具卡片
- **THEN** 路由切换到该工具对应页面并可交互

#### Scenario: Navigate from search by tool id
- **WHEN** 用户在 Spotlight 中确认某个已注册工具
- **THEN** 路由切换到该工具对应页面并可交互
