---
tool_id: hash.digest
keywords: hash, 哈希, md5, MD5, sha256, SHA-256, digest, 摘要, fingerprint
---

# 哈希摘要

`hash.digest` 计算 `md5` 或 `sha256` 摘要（小写 hex 字符串）。参数：
- `input` — 待哈希的字符串
- `algorithm` — `md5` 或 `sha256`

## 典型应用场景

- 校验文件内容（与 `xxd` / `shasum` 对照）
- 接口签名：`md5(secret + body)` 一类签名计算
- 与 `base64.encode` 串联（少见）：先哈希再 base64

## 用户问法 → 工具调用样本

**用户**：算一下 `hi` 的 md5
**工具调用**：`<tool_call>{"name": "hash.digest", "arguments": {"input": "hi", "algorithm": "md5"}}</tool_call>`
**预期输出**：`764efa883dda1e11db47671c4a3bbd9e`

**用户**：sha256 digest of `abc`
**工具调用**：`<tool_call>{"name": "hash.digest", "arguments": {"input": "abc", "algorithm": "sha256"}}</tool_call>`
**预期输出**：`ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad`

**用户**：帮我算 `secret` 的 SHA-256
**工具调用**：`<tool_call>{"name": "hash.digest", "arguments": {"input": "secret", "algorithm": "sha256"}}</tool_call>`
**预期输出**：`2bb80d537b1da3e38bd30361aa855686bde0eacd7162fef6a25fe97bf527a25b`

边界：algorithm 不在枚举内报错；input 不能为空。