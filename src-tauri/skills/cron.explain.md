---
tool_id: cron.explain
keywords: cron, CRON, 定时, 表达式, 调度, schedule, explain, 说明
---

# Cron 表达式说明

## 何时使用 / 何时不用

- **用**：用户请求与本工具能力描述一致时
- **不用**：请求属于其它近邻工具或纯闲聊时，勿强行调用

`cron.explain` 把标准 5 字段 cron 表达式翻译为中文。参数：`input` —
`分 时 日 月 周` 5 字段。**不校验表达式是否语义合理**，仅机械展开。

## 典型应用场景

- 排查线上定时任务执行时间
- 文档化 cron 配置
- 给非运维同事解释「`0 2 * * *`」表示「每天凌晨 2 点」

## 用户问法 → 工具调用样本

**用户**：解释一下 cron 表达式 `*/5 * * * *`
**工具调用**：`<tool_call>{"name": "cron.explain", "arguments": {"input": "*/5 * * * *"}}</tool_call>`
**预期输出**：`每 5 分钟；每小时；每日；每月；每星期`

**用户**：what does `0 2 * * *` mean in cron
**工具调用**：`<tool_call>{"name": "cron.explain", "arguments": {"input": "0 2 * * *"}}</tool_call>`
**预期输出**：`分钟=0；小时=2；每日；每月；每星期`

边界：非 5 字段报错；支持 `*` `*/n` `n` 形式，其他形式按字面量输出。