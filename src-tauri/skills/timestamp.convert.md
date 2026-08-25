---
tool_id: timestamp.convert
keywords: timestamp, 时间戳, unix, epoch, 秒, 毫秒, ISO, RFC3339, 转换, 日期
---

# 时间戳转换

`timestamp.convert` 在 Unix 秒/毫秒时间戳与 ISO-8601 字符串之间互转。
参数：
- `input` — Unix 秒/毫秒时间戳数字，或 ISO/RFC3339 时间字符串
- `unit`（可选）— `seconds` / `millis` / `iso`，缺省自动推断（13 位左右当毫秒）

## 典型应用场景

- 日志时间戳 → 可读时间
- 给前端 / DB 写入前转 ISO
- 排查接口跨时区问题

## 用户问法 → 工具调用样本

**用户**：`1700000000` 是什么时间
**工具调用**：`<tool_call>{"name": "timestamp.convert", "arguments": {"input": "1700000000"}}</tool_call>`
**预期输出**：`{"iso":"2023-11-14T22:13:20Z","unix_seconds":1700000000,"unix_millis":1700000000000}`

**用户**：convert `2023-11-14T22:13:20Z` to unix timestamp
**工具调用**：`<tool_call>{"name": "timestamp.convert", "arguments": {"input": "2023-11-14T22:13:20Z", "unit": "iso"}}</tool_call>`
**预期输出**：`{"iso":"2023-11-14T22:13:20Z","unix_seconds":1699998800,"unix_millis":1699998800000}`

边界：非法时间字符串 / 数字越界报错；无 `unit` 时按长度自动推断。