## Context

`AssistantTrajectory` 已有 `summaryLines` 摘要与 `liveStatus` shimmer，但 streaming 开始时强制 `processOpen = false`，导致用户看不到过程列表。既有 living spec 要求流式展开、结束后收起。

## Goals / Non-Goals

**Goals**

- Streaming：过程列表默认可见，逐条更新简略行
- Done：自动收起为「思考了 Xs」芯片
- 细节（推理全文、工具参数/结果）按需点开

**Non-Goals**

- 不改 agent-event / trajectory 持久化
- 不在摘要行内嵌完整工具结果
- 不重做 AgentRuns 轨迹页（审计页保持现状）

## Decisions

1. **复用 `summaryLines`**：不新造 feed 数据结构；streaming 时 `processOpen = true`，结束后 `false`。
2. **标题行**：streaming 时顶部仍可显示当前 `liveStatus`（或弱化为列表末行高亮）；chevron 在 streaming 也可显示，允许用户手动收起过程列表。
3. **自动滚动**：过程列表容器在新增行时 `scrollIntoView` 最新项（仅 streaming），限制 `max-height` 避免占满屏。
4. **细节默认关**：`detailOpen` 在 streaming 开始与结束时清空；不因 streaming 自动展开某步详情。

## Risks / Trade-offs

- 多工具回合列表变长 → 用 max-height + 滚到底缓解
- 用户手动收起后再有新步骤 → 保持用户选择（不强制再展开），除非回合重新开始

## Migration

无。
