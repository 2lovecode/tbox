## Context

既有实现与 archived `timestamp-convert-now` 将输出锁在 UTC；工具箱页默认 local。

## Decision

1. `iso` = `Local` 时区 RFC3339；`iso_utc` = 同一瞬间的 UTC
2. `unix_*` 不变（绝对秒/毫秒）
3. 朴素 datetime 按 Local 解析；带 offset 的 ISO 按 offset 解析后再转到 Local 展示

## Risks

- 夏令时模糊时刻：`from_local_datetime` 取 `single`/`earliest`
