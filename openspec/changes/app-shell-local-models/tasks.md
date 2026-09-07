## 1. 应用壳

- [x] 1.1 调整 `App.vue` / 侧栏 / 主区高度链，消除整页滚动；聊天与历史仅内部滚动；目视确认窗口无外层滚动条
- [x] 1.2 为工具箱列表建立内部滚动容器，分类筛选保持固定且不随工具卡片滚动

## 2. 通用日志

- [x] 2.1 将引擎日志表单迁到 `GeneralSettingsPanel`，并从 `LlmSettingsPanel` 移除；保存/回读仍走原 command

## 3. 本地模型偏好

- [x] 3.1 实现 `local_models.json` 与 list/update/clear commands；`enabled_model_path` 尊重启用集；单元测试覆盖默认启用与清除不删文件
- [x] 3.2 设置 UI：勾选可用、编辑显示名/icon、清除（可选删文件，默认否）
- [x] 3.3 `ModelSwitcher`（及 local 模型列表）展示 effective icon + 显示名，且仅列可用模型；`vue-tsc` 通过
