## MODIFIED Requirements

### Requirement: Phase 1.5 Lightweight Local Intelligence
系统 SHALL 提供不依赖云端大模型的本地检索增强，至少包括分词（jieba）与拼音匹配，并在 Spotlight 中可开关。

#### Scenario: Pinyin tool match
- **WHEN** 用户在启用本地智能搜索时用拼音片段检索工具
- **THEN** 能匹配到对应中文名称工具

#### Scenario: AI assist toggle
- **WHEN** 用户关闭 Spotlight 中的 AI/本地智能开关
- **THEN** 搜索回退到基础匹配行为
