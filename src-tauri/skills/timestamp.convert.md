---
tool_id: timestamp.convert
keywords: timestamp, 时间戳, unix, epoch, 秒, 毫秒, ISO, RFC3339, 转换, 日期, 当前时间, 现在, now
---

# 时间戳转换

## 何时使用 / 何时不用

- **用**：Unix ↔ ISO 互转；或用户问「当前时间 / 现在几点 / now」需要本机当前时刻
- **不用**：与时间无关的闲聊；勿编造固定日期字符串冒充「现在」

`timestamp.convert` 在 Unix 秒/毫秒时间戳与 ISO-8601 字符串之间互转；`input` 为 `now` / `当前` / `现在` / `此刻` 时返回本机当前 UTC 时刻。
参数：
- `input` — Unix 秒/毫秒时间戳数字、ISO/RFC3339 时间字符串，或 `now`/`当前`/`现在`/`此刻`
- `unit`（可选）— `seconds` / `millis` / `iso`，缺省自动推断（13 位左右当毫秒）；`input` 为当前时刻别名时可省略

## 典型应用场景

- 日志时间戳 → 可读时间
- 给前端 / DB 写入前转 ISO
- 排查接口跨时区问题
- 「看下当前时间」→ `input: now`

## 用户问法 → 工具调用样本

**用户**：`1700000000` 是什么时间
**工具调用**：`<tool_call>{"name": "timestamp.convert", "arguments": {"input": "1700000000"}}</tool_call>`
**预期输出**：`{"iso":"2023-11-14T22:13:20Z","unix_seconds":1700000000,"unix_millis":1700000000000}`

**用户**：convert `2023-11-14T22:13:20Z` to unix timestamp
**工具调用**：`<tool_call>{"name": "timestamp.convert", "arguments": {"input": "2023-11-14T22:13:20Z", "unit": "iso"}}</tool_call>`
**预期输出**：`{"iso":"2023-11-14T22:13:20Z","unix_seconds":1699998800,"unix_millis":1699998800000}`

**用户**：看下当前时间
**工具调用**：`<tool_call>{"name": "timestamp.convert", "arguments": {"input": "now"}}</tool_call>`
**预期输出**：当前 UTC 的 `iso` / `unix_seconds` / `unix_millis`（勿编造固定日期）

边界：非法时间字符串 / 数字越界报错；无 `unit` 时按长度自动推断；问当前时间必须用 `now`（或中文别名），禁止臆造 ISO 字符串。
