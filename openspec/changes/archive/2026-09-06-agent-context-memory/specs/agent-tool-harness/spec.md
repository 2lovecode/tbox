## ADDED Requirements

### Requirement: Context Budget Accounting
harness 在每次模型调用前 SHALL 估算上下文预算，分项至少包括：system、skills、tools、messages、tool_results、memory（若启用用户记忆）。上限来源：local/embedded 使用激活 profile 的 `n_ctx`（或引擎实际有效上下文）；云端使用 profile/提供商约定窗口或可配置上限。预算快照 MUST 经 Agent 事件或等价通道暴露给前端，并与压缩触发使用同一套数字。

#### Scenario: Local n_ctx is the limit
- **WHEN** 激活 local profile 的 n_ctx=8192
- **THEN** 预算上限按 8192（或引擎报告的有效上下文）计算，分项之和用于使用率

#### Scenario: Budget snapshot emitted each model call
- **WHEN** Agent 循环发起一次模型调用
- **THEN** 前端可收到含 used、limit（或 ratio）与分项的预算快照

### Requirement: Threshold Session Compression
当估算使用率达到配置阈值（默认 0.8）时，系统 SHALL 在两次模型调用之间对 Runtime 消息中未压缩的旧 tool results 做批量压缩或截断替换，并标记以防重复压缩。System 与核心工具定义前缀 MUST NOT 因压缩被改写。压缩 MUST 优先保留决策、约束、失败路径与可回源引用。连续全量压缩失败达到熔断次数后 MUST 停止重试并表面错误或降级，MUST NOT 死循环消耗调用。Audit/SQLite 中的原始轨迹 MUST 保持可恢复，与 Runtime 分离。

#### Scenario: Batch compress near capacity
- **WHEN** 使用率超过阈值且存在多条未压缩的大体积 tool results
- **THEN** 下一轮请求的 Runtime 中这些结果被摘要或截断替换，前缀 system/tools 保持稳定，原始结果仍可从 Audit 取回

#### Scenario: No per-turn thrashing
- **WHEN** 使用率远低于阈值
- **THEN** 系统 MUST NOT 每轮对历史做全量压缩

#### Scenario: Circuit breaker on compress failure
- **WHEN** 全量压缩连续失败达到熔断阈值
- **THEN** 停止继续压缩重试，并向用户或日志给出明确失败，不无限重试
