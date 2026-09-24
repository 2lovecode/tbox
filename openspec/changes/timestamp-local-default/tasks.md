## 1. Agent timestamp.convert

- [x] 1.1 `dispatch_timestamp`：`iso` 用 `chrono::Local`；增加 `iso_utc`；朴素时间按本地解析
- [x] 1.2 更新工具 schema 描述与单测（now 的 iso offset ≠ 强制 UTC；iso_utc 存在）
- [x] 1.3 更新 `prompt` 少样本、`datetime-id` Skill 文案
