---
name: 编码与解析
description: URL/Unicode/Base64/Base58/Hex/HTML/Punycode/二进制/摩尔斯/ROT13/凯撒等编解码；URL/表单解析；字符集检测与 UTF-8↔GBK。JSON 转义用 json。
tool_id: base64.encode, base64.decode, encoding.convert, encoding.url_decode, encoding.unicode_encode, encoding.unicode_decode, encoding.base58_encode, encoding.base58_decode, encoding.hex_encode, encoding.hex_decode, encoding.html_encode, encoding.html_decode, encoding.punycode_encode, encoding.punycode_decode, encoding.binary_encode, encoding.binary_decode, encoding.morse_encode, encoding.morse_decode, encoding.rot13, encoding.caesar, encoding.case, encoding.detect, charset.convert, charset.detect, charset.convert_text, url.parse, form.parse
toolbox_id: 10, 19, 33
keywords: base64, Base64, Base58, URL编码, URL解码, Unicode, 十六进制, hex, HTML实体, Punycode, 二进制, 摩尔斯, Morse, ROT13, 凯撒, 编码检测, charset, UTF-8, GBK, 乱码, url解析, form, form-urlencoded
avoid_keywords: JSON格式化, 转义JSON, 哈希, JWT, XML, 图片转Base64
---

# 编码与解析

## 何时使用 / 何时不用

- **用**：各类文本编解码、字符集检测/UTF-8↔GBK、URL/表单结构化解析
- **不用**：JSON 美化/转义 → `json`；图片↔Base64 → `toolbox.media`；哈希/JWT → `crypto`

## Agent 可调用

- `base64.encode` / `base64.decode` — 标准 Base64；`input`
- `encoding.convert` — URL 编码；`encoding.url_decode` — URL 解码
- `encoding.unicode_encode` / `encoding.unicode_decode`、`encoding.base58_encode` / `encoding.base58_decode`
- `encoding.hex_encode` / `encoding.hex_decode`、`encoding.html_encode` / `encoding.html_decode`
- `encoding.punycode_encode` / `encoding.punycode_decode`、`encoding.binary_encode` / `encoding.binary_decode`
- `encoding.morse_encode` / `encoding.morse_decode`、`encoding.rot13`、`encoding.caesar`（`shift`、`direction`）、`encoding.case`（`mode`=upper/lower/reverse）、`encoding.detect`
- `charset.detect`、`charset.convert_text`（`from`/`to`，如 UTF-8 与 GBK）
- `charset.convert` — 按字符集标签校验/解码，不是互转
- `url.parse` / `form.parse`

## 样本

**用户**：把 hi 编成 Base64 → `base64.encode`  
**用户**：URL 编码 `a b` → `encoding.convert`  
**用户**：URL 解码 / 摩尔斯 / 凯撒 / UTF-8 转 GBK → `encoding.url_decode` / `encoding.morse_encode` / `encoding.caesar` / `charset.convert_text`  
**用户**：拆解 URL / 解析表单 → `url.parse` / `form.parse`

## 工具箱页面完整能力

### `/encoding-tools`(19) 编码转换（13 类）

URL 编解码 · Unicode↔中文 · Base64 · Base58 · 文本↔十六进制（含 ASCII 表）· HTML 实体 · Punycode · 文本↔二进制 · 摩尔斯 · ROT13 · 凯撒（可调 shift）· 大小写/反转 · 启发式编码检测（Hex/Base64/摩尔斯/ASCII/中文）。公共：示例、互换、导入/导出。**无 Base62**。

### `/base64-tool`(10)

独立 Base64 编码/解码（UTF-8 安全）。

### `/charset-tools`(33)

检测编码 · **UTF-8↔GBK** 互转 · URL component 编解码 · HTML 实体 · Punycode（IDN 域名）。

## 样本

**用户**：URL 编码 `a b` → `encoding.convert`  
**用户**：拆解 URL / 解析表单 → `url.parse` / `form.parse`
