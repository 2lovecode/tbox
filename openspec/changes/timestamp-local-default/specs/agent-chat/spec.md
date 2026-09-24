## MODIFIED Requirements

### Requirement: Timestamp Convert Supports Current Instant
系统 SHALL 在 Agent 工具 `timestamp.convert` 中支持将 `input` 识别为当前时刻别名：至少包括 `now`（大小写不敏感）、`当前`、`现在`、`此刻`。此时 MUST 返回本机当前时刻快照。成功响应 JSON MUST 包含：

- `iso`：本机**本地时区**的 RFC3339（offset 为本机当前 offset，MUST NOT 固定为 `+00:00`，除非本机即为 UTC）
- `iso_utc`：同一瞬间的 UTC RFC3339
- `unix_seconds` / `unix_millis`：对应 Unix 时间

MUST NOT 将别名解析为字面时间字符串或臆造固定日期。

#### Scenario: now returns local iso with real offset
- **WHEN** Agent 调用 `timestamp.convert` 且 `input` 为 `now`
- **THEN** `unix_seconds` 接近真实时刻，且 `iso` 的数值 offset 等于本机本地时区当前 offset（非一律 `+00:00`），且 `iso_utc` 以 `Z` 或 `+00:00` 表示 UTC

#### Scenario: Chinese alias 当前
- **WHEN** `input` 为 `当前` 或 `现在` 或 `此刻`
- **THEN** 行为与 `now` 相同

#### Scenario: Naive datetime interpreted as local
- **WHEN** `input` 为无时区的 `YYYY-MM-DD HH:MM:SS`
- **THEN** 按本机本地时区解释，`iso` 带本地 offset

#### Scenario: Numeric unix still absolute
- **WHEN** `input` 为合法 Unix 秒或毫秒
- **THEN** `unix_seconds`/`unix_millis` 与输入一致（毫秒经换算），`iso` 为该瞬间的本地 RFC3339
