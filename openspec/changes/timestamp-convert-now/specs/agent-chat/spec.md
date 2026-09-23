## ADDED Requirements

### Requirement: Timestamp Convert Supports Current Instant
系统 SHALL 在 Agent 工具 `timestamp.convert` 中支持将 `input` 识别为当前时刻别名：至少包括 `now`（大小写不敏感）、`当前`、`现在`。此时 MUST 返回本机当前 UTC 时刻，JSON 字段与既有成功响应一致：`iso`、`unix_seconds`、`unix_millis`。MUST NOT 将别名解析为字面时间字符串或臆造固定日期。

#### Scenario: now returns current unix and iso
- **WHEN** Agent 调用 `timestamp.convert` 且 `input` 为 `now`（或 `NOW`）
- **THEN** 返回的 `unix_seconds` 接近调用时刻的真实 Unix 秒，且 `iso` 为对应 RFC3339 字符串

#### Scenario: Chinese alias 当前
- **WHEN** Agent 调用 `timestamp.convert` 且 `input` 为 `当前` 或 `现在`
- **THEN** 行为与 `now` 相同，返回当前 UTC 时刻快照

#### Scenario: Existing numeric and iso inputs unchanged
- **WHEN** `input` 为合法 Unix 数字或 ISO-8601 字符串
- **THEN** 转换行为与扩展前一致
