## 1. 配置模型与预设数据

- [x] 1.1 在 Rust `LlmConfig` / `LlmProvider`（或等价预设 id）增加 `protocol` 枚举与 serde 兼容推断；补单元测试覆盖旧 JSON 无 protocol。验证：`cargo test -p toolbox -- llm` 相关测试通过。
- [x] 1.2 新增 CC Switch Claude 预设快照模块（含默认 base_url/model/protocol、oauth 标记、上游 commit/日期注释）；暴露 list 给前端。验证：`cargo check -p toolbox`；抽样比对上游 `claudeProviderPresets.ts` 条目数与若干默认 URL。
- [x] 1.3 更新 `src/types/llm.ts` 与 `src/stores/llm.ts` 同步 protocol / 预设元数据。验证：`npx vue-tsc --noEmit`（或项目既有类型检查命令）。

## 2. 设置页：提供商、协议、下载 UX

- [x] 2.1 SettingsModal：提供商改为预设列表（含 local/ollama）；增加协议下拉；OAuth 项禁用并提示。验证：手动打开设置切换预设，默认值与协议联动。
- [x] 2.2 GGUF 下载接入 `ProgressBar`，成功/失败/取消明确提示；成功后刷新已安装。验证：手动下载或 mock 进度事件；失败路径不出现「已安装」。
- [x] 2.3 若选项过多，为提供商下拉增加简单过滤（可选但推荐）。验证：输入关键字可缩小列表。

## 3. Ollama pull

- [x] 3.1 新增 Rust command：启动/取消 Ollama pull，解析进度并 emit 事件；失败清理不可用状态。验证：`cargo test` 对进度解析纯函数；有 Ollama 时手动 pull 小模型。
- [x] 3.2 设置页 Ollama 区：模型名、pull 按钮、进度条、成败提示。验证：手动路径；Ollama 关闭时失败提示清晰。

## 4. genai 适配与 Agent

- [x] 4.1 `Cargo.toml` 引入 `genai`；实现 `ChatModel` 适配器（消息/工具互转）。验证：`cargo check -p toolbox`。
- [x] 4.2 扩展 `resolve_backend`：按 provider+protocol 构造后端；local 仍指 sidecar；OAuth/缺配置 → Unavailable。验证：单元测试覆盖 local 无模型不打云端、Anthropic/Ollama 配置完整时 resolve 成功。
- [x] 4.3 Agent 主路径改用 genai 适配器；保留旧 OpenAI 客户端可编译或测试旁路至多一个版本。验证：`cargo test` Agent/llm 相关；手动各测 openai_chat 与 anthropic_messages（有 Key 时）。
- [x] 4.4 `test_llm_connection` 按协议探测。验证：手动对 OpenAI 兼容与 Ollama；Anthropic 有 Key 时。

## 5. 收尾

- [x] 5.1 回归：无模型 local 发消息有引导；切换云端/Ollama 后对话走对应后端。验证：手动 checklist。
- [x] 5.2 `openspec validate llm-multi-provider-protocols`（或项目等价校验）通过。验证：CLI 无 error。
