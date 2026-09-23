## 1. LLM 抽取

- [x] 1.1 新增抽取 prompt + JSON 候选解析（`{key?, text}[]`），非法/空结果可检测
- [x] 1.2 实现 `extract_candidates_via_llm`（当前 ChatModel/配置，无工具、短输出）；失败返回 Err/None
- [x] 1.3 改 `extract_and_ingest_session_delta`：开自动记忆时优先 LLM，失败回退 `extract_candidates_heuristic`，再 `ingest_candidates`
- [x] 1.4 单测：解析合法 JSON；非法 JSON 走回退路径（可 mock/纯解析测）；启发式路径仍绿

## 2. 注入与开关

- [x] 2.1 `loop.rs`：`retrieve_for_inject` 不再要求 `auto_memory_enabled`；仅抽取路径检查开关
- [x] 2.2 设置文案（可选）：标明关闭自动记忆后仍注入已有记忆
- [x] 2.3 验证：关开关时不调用抽取；有存量记忆时 system/上下文仍含 memory 块（单测或最小集成）

## 3. 回归

- [x] 3.1 `cargo test --lib agent::memory`（及 loop 相关若有）通过
- [x] 3.2 密钥拒绝与 UPDATE/revisions 既有测仍通过
