---
tool_id: charset.convert
keywords: charset, 字符集, 编码, encoding, UTF-8, GBK, GB2312, 乱码, 标签
---

# 字符集标签校验

`charset.convert` 验证一段文本按指定字符集标签能否正确解码（无替换字符）。
参数：
- `input` — 文本内容
- `charset` — 字符集标签（如 `UTF-8` / `utf-8` / `GBK`）

## 典型应用场景

- 排查接口返回的乱码（怀疑编码不匹配）
- 校验文件是否声明的字符集一致
- 临时把 UTF-8 内容转 GBK（不常见；当前实现仅做标签校验 + UTF-8 往返）

## 用户问法 → 工具调用样本

**用户**：校验这段文本是不是 UTF-8：`你好`
**工具调用**：`<tool_call>{"name": "charset.convert", "arguments": {"input": "你好", "charset": "UTF-8"}}</tool_call>`
**预期输出**：`你好`（成功，原样返回）

**用户**：check charset of `hello` with utf-8
**工具调用**：`<tool_call>{"name": "charset.convert", "arguments": {"input": "hello", "charset": "utf-8"}}</tool_call>`
**预期输出**：`hello`

边界：未知字符集标签报错；按指定字符集解码出现替换字符时报错。