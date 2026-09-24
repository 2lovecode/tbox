---
name: 密码管理
description: 本地保管账户密码（CRUD/搜索）、随机强密码生成。JWT/哈希/国密用 crypto。无 Agent tool。
toolbox_id: 3
keywords: 密码管理, 强密码, password, 密码保管, 随机密码
avoid_keywords: JWT解析, 哈希摘要, 国密
---

# 密码管理

## 何时使用 / 何时不用

- **用**：本地保管/搜索账户密码、生成随机强密码
- **不用**：JWT/哈希/国密 → `crypto`

## 页面完整能力

`/password-manage`(3)：localStorage CRUD（名称/用户名/密码/URL/备注）· 搜索 · 随机密码 · 复制/编辑/删除。无 Agent tool。

## 样本

**用户**：生成强密码 / 存密码 → 引导 `/password-manage`；勿编造 tool_call。
