---
tool_id: xml.format
keywords: xml, XML, 格式化, 美化, pretty, format, 缩进
---

# XML 格式化

`xml.format` 将 XML 字符串做美化（缩进/换行）。参数：`input` — 待格式化的 XML。

## 典型应用场景

- SOAP / Web Service 报文调试
- 配置文件（pom.xml / settings.xml）压缩→可读化
- 对比 XML 差异前先统一缩进

## 用户问法 → 工具调用样本

**用户**：格式化 XML `<a><b/></a>`
**工具调用**：`<tool_call>{"name": "xml.format", "arguments": {"input": "<a><b/></a>"}}</tool_call>`
**预期输出**（带缩进）：
```
<a>
  <b/>
</a>
```

**用户**：pretty print xml `<root><x>1</x></root>`
**工具调用**：`<tool_call>{"name": "xml.format", "arguments": {"input": "<root><x>1</x></root>"}}</tool_call>`
**预期输出**：带缩进的版本

边界：非合法 XML 时回退到「去多余空白」处理；空输入报错。