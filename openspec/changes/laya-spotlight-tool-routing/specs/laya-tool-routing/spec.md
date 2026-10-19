# Spec Delta

## Purpose

本地 Laya 决策路由能力：通过用户本机运行的 laya-serve HTTP 服务，将自然语言输入映射到已注册工具 id，并提供超时、置信度阈值与降级契约，使上层入口（首先是 Spotlight）可在不破坏既有行为的前提下获得智能路由。

## ADDED Requirements

### Requirement: Decision Service Invocation
系统 SHALL 通过配置的 laya-serve HTTP endpoint 发起决策请求，请求包含用户输入文本与「工具 id + 名称 + 描述 + 标签」候选列表，并解析返回的 `choice` 结果与置信度。

#### Scenario: Route input to a tool
- **WHEN** 决策服务可用且用户输入「把 JSON 转成 YAML」
- **THEN** 系统向 laya-serve 发起 choice 决策请求，候选列表包含所有已注册工具，并解析出工具 id 与置信度

#### Scenario: Service unreachable
- **WHEN** laya-serve 未启动或连接被拒绝
- **THEN** 系统不抛出用户可见错误，决策结果视为未命中

### Requirement: Timeout and Degradation
决策请求 SHALL 执行配置的超时时间（默认 300ms）内完成；超时或失败时系统 SHALL 静默降级为未命中，不阻塞调用方主流程。

#### Scenario: Slow decision service
- **WHEN** laya-serve 超过配置超时时间未返回
- **THEN** 请求被取消，结果为未命中，调用方立即继续执行回退路径

### Requirement: Confidence Threshold
系统 SHALL 仅当返回置信度大于等于配置阈值（默认 0.7）时视为命中；低于阈值的 SHALL 返回未命中。

#### Scenario: Low confidence result
- **WHEN** laya-serve 返回置信度 0.4 且阈值为 0.7
- **THEN** 决策结果为未命中，不向调用方推荐任何工具

### Requirement: Configuration
系统 SHALL 提供配置项：启用开关（默认关闭）、endpoint（默认 `http://127.0.0.1:8000`）、置信度阈值（默认 0.5，zero-shot 阈值扫描实测 0.5 覆盖 31/50、精度 0.839 优于 0.7）、超时毫秒数（默认 300）；用户可通过设置界面修改。

#### Scenario: Feature disabled by default
- **WHEN** 用户未启用决策路由
- **THEN** 系统不发起任何对 laya-serve 的网络请求，行为与未集成时一致

### Requirement: Health Check
系统 SHALL 提供决策服务健康检查能力，设置界面 SHALL 展示服务可达状态。

#### Scenario: Check service health
- **WHEN** 用户在设置中点击「检测服务」且 laya-serve 正在运行
- **THEN** 界面显示服务可达；未运行时显示不可达及安装/启动指引
