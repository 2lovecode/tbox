# Design: multi-provider-models

## Context

现状（`src-tauri/src/commands/llm.rs`）：单条 `LlmConfig`（provider/base_url/model/has_api_key）存于 `~/.toolbox/llm_config.json`，单个 API Key 存于 `llm_secret.bin`（AES-256-GCM，hostname+APP_SALT 派生密钥）。`genai_model.rs::resolve_target` 在每轮推理时读取该单配置。设置页是单表单；聊天顶栏无模型入口。

约束：Tauri 2 命令边界、无新依赖、加密方案不变、必须平滑迁移旧配置。

## Goals / Non-Goals

**Goals:**
- 多 profile 持久化与独立密钥存储
- 旧单配置自动迁移
- 聊天顶栏实时切换器，切换对下一轮生效

**Non-Goals:**
- 会话级模型绑定、自动故障转移、keychain/OAuth/云同步

## Decisions

1. **存储布局**：新增 `~/.toolbox/llm_profiles.json`（`{ profiles: [...], activeId }`，非敏感字段），密钥改为 `llm_secrets/<profile_id>.bin`（复用现有加解密函数，逐 profile 一文件）。
   - 备选：SQLite。否决——配置量小、需与用户可见文件语义一致，JSON 足够且迁移简单。
2. **profile id**：UUID v4，由 Rust 侧生成；前端只读。
3. **迁移策略**：首次读 profiles 文件不存在时，若存在旧 `llm_config.json`，则转换为其一 profile（id 新建），API Key 从 `llm_secret.bin` 解密后按新路径重新写入并删除旧文件（或重命名为 `.migrated`，保留回滚窗口）；`activeId` 指向它。迁移在 Tauri command 层惰性执行（首次 `list_llm_profiles`），避免启动顺序问题。
4. **Tauri commands**（新，均在 `commands/llm.rs`）：
   - `list_llm_profiles() -> { profiles, activeId }`
   - `save_llm_profile(profile: ProfileInput)`（含可选 api_key；新建或更新，id 缺省则新建）
   - `delete_llm_profile(id)`（若为激活项则清空 activeId）
   - `set_active_llm_profile(id)`
   - 旧 `save_llm_config`/`clear_llm_api_key`/`get_llm_config` 保留为迁移兼容垫片（内部转发到 profile 机制），供未更新前端路径过渡；验收后可移除。
5. **推理侧接线**：`genai_model.rs::resolve_target` 改为读取激活 profile（`LlmConfig` 结构由激活 profile 投影得到，最小化改动）。回合开始时取一次快照，回合内不重读——天然满足"进行中回合不被打断"。
6. **前端**：
   - 新组件 `ModelSwitcher.vue`（顶栏下拉：profile 显示名 + provider 图标/标签 + 模型名，当前项打勾；空态按钮跳设置）。
   - 设置 LLM 页改为 profile 列表 + 编辑表单（复用现有表单字段与连通性测试，测试目标改为指定 profile）。
   - 切换调用 `set_active_llm_profile`，乐观更新 + 失败回滚提示。
7. **并发**：profiles 文件读写沿用现状（整体读-改-写，无跨进程竞争假设），与今天风险一致。

## Risks / Trade-offs

- [迁移误删密钥] → 迁移只重命名旧文件为 `.migrated` 不物理删除；迁移后旧配置读取路径继续可用作回滚。
- [多 profile 逐一拉模型列表变慢] → 切换器只展示本地保存的 model 字符串，不实时探测 `/models`。
- [删除激活 profile 导致无模型] → 清空 activeId 并在聊天侧触发既有"LLM Unavailable"引导。
- [前端旧命令残留] → 兼容垫片标注 deprecated，tasks 内列入清理项。

## Migration Plan

1. Rust 层先落 profiles 存储与迁移 + 垫片命令（旧前端仍可用）。
2. 前端设置页切换到 profile API。
3. 聊天顶栏加切换器。
4. 验证迁移场景后移除垫片命令（可在后续 change 处理）。
回滚：恢复旧前端 + 垫片命令即可；数据侧 `.migrated` 文件可手工还原。

## Open Questions

- profile 是否需要"描述/备注"字段？（暂不加，保持最小）
