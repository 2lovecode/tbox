# Encoding and Crypto Tools

## Purpose

编码转换与加解密类工具的总体行为（由 IMPLEMENTATION_SUMMARY / ROADMAP 收敛为领域规格；具体算法细节随后续 change 增量细化）。

## Requirements

### Requirement: Local Encode Decode
编码类工具（Base64/Base58/Base62、URL、Unicode、HTML 实体、进制/Hex 等）SHALL 在本地完成转换，并在非法输入时给出可读错误而非崩溃。

#### Scenario: Invalid Base64 input
- **WHEN** 用户对非法 Base64 字符串执行解码
- **THEN** 界面或命令返回可读错误，应用保持可用

### Requirement: Hashing
系统 SHALL 支持常见哈希（至少 MD5 / SHA-1 / SHA-256 / SHA-512）的本地计算。

#### Scenario: Compute SHA-256
- **WHEN** 用户输入文本并选择 SHA-256
- **THEN** 返回正确的十六进制摘要并可复制

### Requirement: Symmetric and Asymmetric Crypto
系统 SHALL 提供 AES 与 RSA 相关加解密/密钥能力；JWT 工具 SHALL 支持解析（及已实现的生成/验证能力）。

#### Scenario: JWT parse
- **WHEN** 用户粘贴合法 JWT
- **THEN** 展示可读的 header/payload 字段

### Requirement: SM Algorithms
系统 SHALL 提供国密 SM2 / SM3 / SM4 相关能力，数据在本地处理。

#### Scenario: SM3 digest
- **WHEN** 用户对输入计算 SM3
- **THEN** 返回摘要结果且不上传原文
