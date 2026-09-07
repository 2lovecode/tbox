## ADDED Requirements

### Requirement: JSON Diff Structured Editing And Review
JSON 对比工具 SHALL 提供宽幅、可编辑且带 JSON 语法高亮的输入区；对比后 MUST 在原始输入位置继续保留编辑能力，并按行标识新增、删除与修改。工具 SHALL 提供差异明细表，包含差异类型、路径、原值与新值。

#### Scenario: Edit JSON with syntax highlighting
- **WHEN** 用户在任一输入区输入或格式化 JSON
- **THEN** 输入区保持可编辑，并以代码编辑器形式显示 JSON 语法

#### Scenario: Locate differences in editors
- **WHEN** 两个 JSON 存在新增 key、缺失 key 或同 key 不同 value
- **THEN** 对应行在相应输入区分别显示新增、删除或修改颜色

#### Scenario: Review difference details
- **WHEN** 对比完成且存在差异
- **THEN** 明细表展示类型、路径、原值与新值，并可按差异类型过滤
