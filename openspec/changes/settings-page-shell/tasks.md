## 1. 路由与设置页壳

- [x] 1.1 在 `src/router/main.ts` 注册 `/settings` 与 `/settings/:section?`（section ∈ llm|general|about，非法回退 llm），并新增 `src/views/SettingsPage.vue` 左菜单 + 右内容骨架；验证：浏览器/客户端直接打开 `/settings` 可见左菜单且默认 LLM 选中
- [x] 1.2 调整 `App.vue` 的 `showSidebar`，使 `/settings*` 不显示工具侧栏；验证：进入设置页时工具 SideBar 隐藏，仅见设置内左菜单
- [x] 1.3 新增通用 / 关于占位面板组件，切换菜单时右侧内容切换；验证：点「通用」「关于」看到占位文案，URL section 同步

## 2. LLM 列表与编辑对话框

- [x] 2.1 从 `SettingsModal.vue` 抽出 `LlmSettingsPanel.vue`：仅 profile 列表 + 新建/使用/编辑/删除；验证：设置页 LLM 区无内联完整表单，空态与有数据列表正确
- [x] 2.2 实现 `ProfileEditorDialog.vue`，迁移现有新建/编辑表单字段、保存/测试连通、本地模型与 Ollama 相关 UI；验证：新建保存后列表出现新项，编辑保存后字段更新，关闭对话框后列表仍在
- [x] 2.3 编辑模式下提供方只读（不渲染可切换列表 / 不调用 applyPreset）；验证：编辑已有 profile 时无法改提供方
- [x] 2.4 实现 `ProviderPicker.vue`（筛选 + 展开全部预设，OAuth 标明且不可保存为有效后端），仅用于新建；验证：筛选关键词缩小列表，点击可选中，OAuth 项不可保存

## 3. 入口迁移与清理

- [x] 3.1 将顶栏齿轮、`Cmd/Ctrl+,`、`ModelSwitcher` / 首页等「打开设置」改为 `router.push('/settings')`（已在设置页再按时 `back` 或回 `/`）；验证：各入口进入设置页而非弹窗
- [x] 3.2 精简 `useSettingsStore`（移除 `isOpen` 弹窗态），删除或停用 `SettingsModal.vue` 并去掉 `App.vue` 挂载；验证：全局搜索无 SettingsModal 引用，`npm run build` 或 `vue-tsc`/项目既有前端检查通过

## 4. 回归核对

- [x] 4.1 手工核对：激活切换、删除确认、API Key 保存与测试连通、local 下载进度在关对话框后仍可继续；验证：行为与改版前一致（仅交互壳变化）
