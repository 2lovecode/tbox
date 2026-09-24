---
name: 摘要与令牌
description: MD5/SHA 哈希、JWT 解析与生成、国密 SM2/SM3/SM4/HMAC-SM3。Base64 正文用 encoding；密码保管用 toolbox.security。
tool_id: hash.digest, jwt.parse
toolbox_id: 11, 14, 22
keywords: hash, 哈希, md5, sha1, sha256, sha384, sha512, JWT, token, 解析JWT, 生成JWT, 国密, SM2, SM3, SM4, HMAC-SM3
avoid_keywords: Base64编码正文, JSON格式化, 密码管理
---

# 摘要与令牌

## 何时使用 / 何时不用

- **用**：算哈希、解析/生成 JWT、国密加解密与签验
- **不用**：正文 Base64 → `encoding`；密码保管 → `toolbox.security`

## Agent 可调用（子集）

- `hash.digest` — 仅 `algorithm`=`md5`|`sha256`；`input`
- `jwt.parse` — 解 header/payload，`verified:false`；**不验签、不生成**

Agent **不能**：SHA-1/384/512、JWT 生成、国密全套（请开页面）。

## 工具箱页面完整能力

- `/hash-generator`(11)：MD5 / SHA-1 / SHA-256 / SHA-384 / SHA-512（可全部或单选）
- `/jwt-tool`(14)：解析（`parse_jwt`）+ 生成（`generate_jwt` + payload + secret）；解析侧无验签 UI
- `/gm-crypto`(22)：SM3 摘要 · SM4 加解密+密钥生成 · SM2 密钥对+签验 · HMAC-SM3

## 样本

**用户**：算 hello 的 sha256 → `hash.digest` `{algorithm:"sha256"}`  
**用户**：解析这段 JWT → `jwt.parse`  
**用户**：生成 JWT / SHA-512 / SM4 → 引导对应页面
