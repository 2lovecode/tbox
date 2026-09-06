## 1. 设置页布局与提供方选择弹窗

- [x] 1.1 去掉 `SettingsPage.vue` 冗余大号「设置」页眉（滑块图标+标题+副标题），收紧布局；验证：打开 `/settings/llm` 不再出现该页眉块
- [x] 1.2 实现独立 `ProviderPickerDialog`（筛选 + 表格/网格，含自定义端点），新建流程从编辑对话框打开；验证：新建时点选提供方弹出第二层对话框，选中后回填
- [x] 1.3 前端展示预设 icon/iconColor（无资源则首字母色块）；验证：提供方列表可见图标或兜底色块，`vue-tsc` 通过

## 2. CC Switch 预设同步

- [x] 2.1 对照最新 `claudeProviderPresets.ts` 更新 `llm_presets.rs`（含 icon/iconColor、快照日期），并扩展 `list_llm_presets` / 前端类型；验证：`cargo check -p tbox`（或等价包名）通过，预设数量/关键条目与上游一致抽查

## 3. 本地模型目录与自定义下载

- [x] 3.1 将精选目录切换为 Qwen3 约 0.6B / 1.7B 级 Instruct Q4_K_M（锁定 HF URL），移除 Qwen2.5 主推；验证：`list_local_models` 返回新目录项
- [x] 3.2 实现自定义 GGUF URL 下载 command + 设置 UI（进度/取消/失败不标已安装）；验证：相关 unit/集成测试或手动下载失败路径，`cargo test` 覆盖 catalog 关键测试

## 4. GPU 优先

- [x] 4.1 配置 Windows/Linux 的 CUDA→Vulkan→CPU 与 macOS Metal；加载路径按可用性设 `n_gpu_layers`；引擎状态暴露加速后端；验证：`cargo check` 在本机目标通过，状态字段在无 GPU/无 GPU 下语义正确（文档注明 CUDA feature 限制）

## 5. 回归

- [x] 5.1 跑 `vue-tsc --noEmit` 与关键 `cargo check`；手工核对：编辑提供方只读、OAuth 不可保存、旧 GGUF 仍可被扫描；验证：命令通过且无 SettingsModal 回归
