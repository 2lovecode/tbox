---
name: 摘要与令牌
description: MD5/SHA 哈希、JWT 解析与生成、国密 SM2/SM3/SM4/HMAC-SM3。Base64 正文用 encoding；密码保管用 toolbox.security。
tool_id: hash.digest, jwt.parse, jwt.generate, gm.sm3, gm.sm4_encrypt, gm.sm4_decrypt, gm.sm4_key, gm.sm2_keypair, gm.sm2_sign, gm.sm2_verify, gm.hmac_sm3
toolbox_id: 11, 14, 22
keywords: hash, 哈希, md5, sha1, sha256, sha384, sha512, SHA-512, SHA-1, JWT, token, 解析JWT, 生成JWT, 国密, SM2, SM3, SM4, HMAC-SM3
avoid_keywords: Base64编码正文, JSON格式化, 密码管理
---

# 摘要与令牌

## 何时使用 / 何时不用

- **用**：算哈希、解析/生成 JWT、国密加解密与签验
- **不用**：正文 Base64 → `encoding`；密码保管 → `toolbox.security`

## Agent 可调用

- `hash.digest` — `algorithm`=`md5`|`sha1`|`sha256`|`sha384`|`sha512`；`input`
- `jwt.parse` — 解 header/payload，不验签
- `jwt.generate` — `payload` JSON + `secret`
- `gm.sm3`、`gm.sm4_encrypt` / `gm.sm4_decrypt` / `gm.sm4_key`、`gm.sm2_keypair` / `gm.sm2_sign` / `gm.sm2_verify`、`gm.hmac_sm3`

## 样本

**用户**：算 hello 的 sha256 → `hash.digest` `{algorithm:"sha256"}`  
**用户**：算 SHA-512 → `hash.digest` `{algorithm:"sha512"}`  
**用户**：解析这段 JWT → `jwt.parse`  
**用户**：生成 JWT / SM4 → `jwt.generate` / `gm.sm4_encrypt`

## 工具箱页面完整能力

- `/hash-generator`(11)：MD5 / SHA-1 / SHA-256 / SHA-384 / SHA-512（可全部或单选）
- `/jwt-tool`(14)：解析（`parse_jwt`）+ 生成（`generate_jwt` + payload + secret）；解析侧无验签 UI
- `/gm-crypto`(22)：SM3 摘要 · SM4 加解密+密钥生成 · SM2 密钥对+签验 · HMAC-SM3

## 样本

**用户**：算 hello 的 sha256 → `hash.digest` `{algorithm:"sha256"}`  
**用户**：解析这段 JWT → `jwt.parse`
