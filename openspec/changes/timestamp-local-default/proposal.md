## Why

问「当前时间」时 `timestamp.convert` 的 `iso` 固定 `+00:00`（UTC），与工具箱时间戳页默认「本地时间」不一致，对中国用户不直观。

## What Changes

- `timestamp.convert` 成功响应的 `iso` 改为**本机本地时区** RFC3339（带实际 offset，如 `+08:00`）
- 额外返回 `iso_utc`（UTC RFC3339），便于对照
- 无时区的朴素时间字符串（如 `2026-09-24 12:00:00`）按**本地**解释，不再当 UTC
- Unix 时间戳仍为绝对时刻；`iso` 以本地展示
- 更新工具 schema 文案、系统少样本、`datetime-id` Skill

## Capabilities

- `agent-chat`: 修改 `timestamp.convert` 时区默认

## Impact

- Agent 回复「当前时间」将显示本地 offset
- 依赖 `iso` 恒为 `+00:00` 的外部脚本需改读 `iso_utc`
