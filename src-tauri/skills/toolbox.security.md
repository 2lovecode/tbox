---
name: 密码管理
description: 本地保管账户密码（CRUD/搜索）、随机强密码生成。JWT/哈希/国密用 crypto。读写密码库需用户批准。
tool_id: password.generate, password.list, password.get, password.save, password.delete
toolbox_id: 3
keywords: 密码管理, 强密码, password, 密码保管, 随机密码
avoid_keywords: JWT解析, 哈希摘要, 国密
---

# 密码管理

## 何时使用 / 何时不用

- **用**：本地保管/搜索账户密码、生成随机强密码
- **不用**：JWT/哈希/国密 → `crypto`

## 页面完整能力

`/password-manage`(3)：与 Agent 共用同一份本地密码库。`password.generate` 直接生成；`password.list` / `password.get` / `password.save` / `password.delete` 必须先经用户批准。

## 样本

**用户**：生成强密码 → `password.generate`  
**用户**：列出已存密码 → `password.list`（先批准）
