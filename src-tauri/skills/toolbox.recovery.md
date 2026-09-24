---
name: 文件恢复
description: 文件恢复扫描与勾选恢复界面（当前为演示数据）。普通文件管理/PDF 用 toolbox.media。无 Agent tool。
toolbox_id: 7
keywords: 文件恢复, 误删, recover, 恢复文件
avoid_keywords: 密码, PDF合并
---

# 文件恢复

## 何时使用 / 何时不用

- **用**：尝试恢复误删文件（页面流程）
- **不用**：普通文件管理/PDF → `toolbox.media`

## 页面完整能力

`/file-recovery`(7)：路径扫描 UI · 勾选恢复。当前实现为假进度 + 硬编码示例列表，**非真实磁盘恢复**。无 Agent tool。

## 样本

**用户**：恢复误删文件 → 引导 `/file-recovery` 并说明能力边界；勿编造 tool_call。
