## 1. Rust 存储层（llm.rs）

- [x] 1.1 定义 `LlmProfile`（id/name/provider/protocol/base_url/model/has_api_key）与 `ProfileStore`（`llm_profiles.json` 读写：profiles + activeId），含单元测试
- [x] 1.2 密钥按 profile 独立存储：`llm_secrets/<id>.bin`，复用现有 AES-256-GCM 加解密；删除 profile 时同步删密钥文件
- [x] 1.3 旧配置迁移：profiles 文件缺失且存在 `llm_config.json` 时，惰性迁移为激活 profile（旧文件改名 `.migrated`），含迁移单测（含带 Key/不带 Key/全新安装三种）

## 2. Tauri commands

- [x] 2.1 新增 `list_llm_profiles` / `save_llm_profile` / `delete_llm_profile` / `set_active_llm_profile` 并注册到 invoke handler
- [x] 2.2 旧 `save_llm_config`/`get_llm_config`/`clear_llm_api_key` 改为基于激活 profile 的兼容垫片，标注 deprecated
- [x] 2.3 连通性测试命令支持按 profile id 测试

## 3. 推理侧接线

- [x] 3.1 `genai_model.rs::resolve_target` 改读激活 profile（回合开始快照），保证回合内配置不变
- [x] 3.2 补测试：激活 profile 切换后 resolve_target 返回新端点/模型

## 4. 前端：设置页

- [x] 4.1 LLM 设置改为 profile 列表 + 编辑表单（新建/编辑/删除/设为默认），复用现有字段与预设选择
- [x] 4.2 连通性测试与 API Key 设置/清除按 profile 工作

## 5. 前端：聊天切换器

- [x] 5.1 新增 `ModelSwitcher` 组件：列出 profiles（名称 + provider + 模型），当前项标识，空态引导去设置
- [x] 5.2 接入聊天顶栏：切换调用 `set_active_llm_profile`，乐观更新 + 失败回滚；流式输出中允许切换但不打断当前回合

## 6. 验证与收尾

- [x] 6.1 端到端验证：旧配置迁移、多 profile 保存、聊天中切换后下一轮消息使用新模型、删除激活 profile 后的不可用引导
- [x] 6.2 `cargo test` + 前端构建通过；更新（如需）工具/能力相关文档

## 7. 模型级切换（输入框切换器）

- [x] 7.1 新增 `list_profile_models(profile_id)` command：openai_chat/openai_responses 走 `GET {base}/models`（Bearer Key），ollama 走 `/api/tags`，local 列已安装 GGUF；anthropic_messages/gemini_native 返回空由前端走手动输入
- [x] 7.2 切换器从顶栏移入输入框区域（composer），按 profile 分组展示模型列表（懒加载、当前模型打勾），点选模型 = 保存该 profile 的 model + 设为激活
- [x] 7.3 无模型列表的协议提供手动输入模型；切换不打断流式回合；空态引导不变
- [x] 7.4 设置页配置提供方时支持自动拉取可用模型：新增按端点直查命令（provider/protocol/baseUrl/apiKey，未保存的草稿也可查），表单模型字段提供「拉取模型」按钮 + 点选回填
- [x] 7.5 拉取成功后模型改为「下拉列表 + 添加按钮」交互：选中一个模型点「添加」回填表单模型字段；下拉含当前值与全部拉取结果
- [x] 7.6 API Key 回填展示：新增 `reveal_llm_profile_api_key`（按 profile id 解密返回，仅本机 UI 展示用）；编辑已保存配置时点「回填」把已存 Key 填入输入框（默认掩码，可切换明文），留空保存仍保留原 Key

## 8. 思考内容分离与流式分块

- [x] 8.1 新增 `split_think_tags`：剥离内联 `<think>…</think>`（含未闭合截断）到 reasoning，与后端已分离 reasoning 合并；含单测
- [x] 8.2 loop.rs Text 分支：reasoning/token 事件按块流式发出（替代整段单发），持久化仍写完整文本

## 9. 助手头像随机化

- [x] 9.1 头像集合（活泼图标 × 渐变配色）+ 按会话 id 哈希稳定选取；新会话随机、同会话稳定；流式气泡与历史消息一致

