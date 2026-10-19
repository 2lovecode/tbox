# Spec Delta

## MODIFIED Requirements

### Requirement: Configuration
系统 SHALL 提供配置项：决策模式 `mode`（`off | embedded | external`，默认 `off`）、置信度阈值（默认 0.5，zero-shot 阈值扫描实测 0.5 覆盖 31/50、精度 0.839 优于 0.7）、超时毫秒数（默认 300，仅 external 模式生效）、external 模式的 endpoint（默认 `http://127.0.0.1:8000`）、模型目录（默认应用数据目录 `laya/`）；用户可通过设置界面修改。

#### Scenario: Feature disabled by default
- **WHEN** 用户未启用决策路由（mode 为 off）
- **THEN** 系统不发起任何决策请求（网络或进程内），行为与未集成时一致

#### Scenario: Mode-specific fields
- **WHEN** 模式为 embedded
- **THEN** endpoint 与超时配置不参与决策执行
