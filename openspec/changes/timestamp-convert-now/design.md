## Context

Agent 轨迹显示：问「看下当前时间」→ 调 `timestamp.convert` 传 `now` 失败 → 模型编造 `2023-10-05T12:34:56Z` 当结果。UI 层已有 `current_timestamp`，未暴露给 Agent。

## Goals / Non-Goals

**Goals:** `input` 为当前时刻别名时返回真实 `Utc::now()` 快照。

**Non-Goals:** 新工具 id；本地时区参数；改前端转换器页面。

## Decisions

1. **扩展现有 `timestamp.convert`，不新建工具** — 模型已会选它；改契约成本最低。
2. **别名**：`now`、`当前`、`现在`（trim + ASCII 小写比较对 `now`；中文精确匹配）。
3. **输出格式不变**：`{ iso, unix_seconds, unix_millis }`，与现有成功路径一致。
4. **`unit` 在 now 路径下忽略**（仍可为任意合法值）。

## Risks / Trade-offs

- [小模型仍可能编造日期] → Skill + schema 明确写「问当前时间用 input=now」降低误用。
- [时钟依赖本机] → 与桌面 App 预期一致。

## Migration Plan

无迁移；旧调用不受影响。
