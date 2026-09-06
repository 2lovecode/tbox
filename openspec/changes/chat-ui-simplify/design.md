## Context

See proposal.md。`chat-identity-copy` 已落地头像/助手名/常显复制；主规格中 `Message Copy` 与 `Agent Avatar Variety` 仍在。窗口默认在 `tauri.conf.json` 为 1400×1050。

## Goals / Non-Goals

**Goals:** 缩小默认窗；去掉身份 UI 与设置；弱用户气泡；悬停下方复制。  
**Non-Goals:** 工具轨大改；主题系统重做。

## Decisions

1. 窗口：`tauri.conf.json` → width 1200, height 780；窄屏断点改为 900px，并用 `.container > aside` 隐藏侧栏，避免默认宽落入断点导致单列乱版。  
2. 删除/停用 `useChatIdentity` 与 General 身份表单；General 恢复简短占位。  
3. `HomePage` 用户消息：无头像；弱气泡；`msg-actions` 悬停显示在气泡下。  
4. `AssistantTrajectory`：无头像/无名称；正文无强气泡；悬停下方复制。  
5. OpenSpec：MODIFIED Message Copy + Avatar Variety；REMOVED identity requirements（含 settings-general，即使尚未 archive 到 main）。

## Risks / Trade-offs

- [settings-general 未进 main] → archive 时以 REMOVED 为准，避免把身份要求合入长期规格。
