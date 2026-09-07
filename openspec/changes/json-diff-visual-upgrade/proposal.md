## Why

JSON 对比工具当前输入区是纯文本框，对比后还会切换到只读高亮视图；差异定位依赖脆弱的行路径推断，结果也没有可扫描的明细表格，长 JSON 难以审阅。

## What Changes

- 工具内容区扩大到更宽的桌面布局，并为输入区提供一致的编辑高度。
- 输入区改为可编辑 JSON 代码视图：语法高亮、行号、格式化、自动滚动同步，编辑与对比状态不互相打断。
- 差异结果直接映射到格式化 JSON 的行，输入框内按行标记新增、删除与修改。
- 在统计下方增加差异明细表，按差异类型过滤，展示路径、原值、新值和类型。

## Capabilities

### Modified Capabilities

- `tool-registry`: JSON 对比工具页 SHALL 提供结构化 JSON 编辑、行级差异标识和差异明细表。

## Impact

- 前端：`src/views/tools/JsonDiff.vue`。
- Rust：`compare_json` 契约与差异算法保持不变。
- Non-goals：不引入 Monaco/CodeMirror 等编辑器依赖；不改语义对比规则；不做双向同步或数组智能匹配。
