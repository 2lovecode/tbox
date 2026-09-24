---
name: 时间标识与数值
description: 时间戳↔日期（默认本地时区）、UUID v4/v7/v5/批量/验证、Cron 构建解释验证、进制/罗马/分数等。测网速坐标用对应 toolbox Skill。
tool_id: timestamp.convert, uuid.generate, cron.explain, number.convert
toolbox_id: 16, 30, 31, 32
keywords: 时间戳, timestamp, now, 当前时间, 时区, 本地时间, UUID, v4, v7, v5, cron, 进制, 二进制, 十六进制, 八进制, 罗马数字, 科学计数法, 分数
avoid_keywords: 网速, 坐标距离, 颜色
---

# 时间标识与数值

## 何时使用 / 何时不用

- **用**：时间戳/当前时刻、UUID、Cron、进制与数值换算
- **不用**：测网速/坐标/颜色 → 对应 toolbox Skill

## Agent 可调用（子集）

- `timestamp.convert` — `input` 为秒/毫秒/ISO/`now`|`当前`|`现在`；可选 `unit`
  - 返回 `iso`（**本机本地时区** RFC3339）、`iso_utc`、`unix_seconds`、`unix_millis`
  - 无时区的 `YYYY-MM-DD HH:MM:SS` 按本地解释；Unix 为绝对时刻
- `uuid.generate` — **仅单个 v4**，无参
- `cron.explain` — 5 字段 cron 启发式中文说明；`input`
- `number.convert` — 任意进制 2..=36；`input`+`from_base`+`to_base`

Agent **不能**：多时区选择 UI、UUID v7/v5/批量/验证/Base64、Cron 构建/校验/NL、罗马数字/科学计数/分数（请开页面）。

## 工具箱页面完整能力

- `/timestamp-converter`(16)：时间戳↔日期（auto/s/ms、多时区）；当前时间多种格式；相对时间
- `/uuid-tools`(30)：生成 v4/v7 · 批量(1–1000) · 验证 · UUID↔Base64 · 版本检测 · v5(namespace+name) · NIL
- `/cron-tools`(31)：字段构建+生成 · 自然语言 · 验证 · 解释 · 常用示例
- `/number-tools`(32)：十↔十六↔二↔八 · 科学计数法→小数 · 罗马数字 1–3999 · 分数↔小数

## 样本

**用户**：当前时间 → `timestamp.convert` `{input:"now"}`（读 `iso` 本地时间，勿说成 UTC）  
**用户**：生成 UUID → `uuid.generate`（要 v7/批量开 `/uuid-tools`）  
**用户**：`0 */5 * * *` 什么意思 → `cron.explain`  
**用户**：十六进制 FF 转十进制 → `number.convert` `{from_base:16,to_base:10}`  
**用户**：罗马数字 / 分数 → 引导 `/number-tools`
