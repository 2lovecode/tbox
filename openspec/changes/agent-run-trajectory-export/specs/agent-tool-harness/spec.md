## MODIFIED Requirements

### Requirement: Threshold Session Compression

当估算使用率达到配置阈值（默认 0.8）时，系统 SHALL 在两次模型调用之间对 **Runtime** 上下文按策略瀑布降载，**首条 system 提示词 MUST 始终保留且正文不改写**。Audit / `session_events` / SQLite 原始轨迹 MUST 保持可恢复，与 Runtime 分离。

**瀑布顺序（够用即停，每步后重算 budget）**：

1. `keep_recent`：丢弃较旧的非 system 消息，保留最近 K 条非 system（默认 K=12）；尽量不拆开 assistant(tool_calls) 与其后 tool 结果对
2. `summarize_tools`：对未标记的大体积 tool 结果做就地摘要替换（`[COMPRESSED]`）
3. `reset`：Runtime 清空为 `[system, user(本回合原文)]`

连续无法降载达到熔断次数后 MUST 停止重试并表面错误或降级，MUST NOT 死循环。

#### Scenario: Prefer keep_recent first
- **WHEN** 使用率超过阈值且非 system 消息数 > K
- **THEN** 优先丢弃较旧非 system，保留 system + 最近 K；Audit 原文仍可查

#### Scenario: Fall back to summarize_tools
- **WHEN** `keep_recent` 后仍超阈值且存在可摘要的大体积 tool results
- **THEN** 对这些 tool 结果就地摘要；system 不变

#### Scenario: Last resort reset
- **WHEN** 前两级后仍超阈值
- **THEN** Runtime 变为 `[system, user(本回合原文)]`

#### Scenario: No per-turn thrashing
- **WHEN** 使用率远低于阈值
- **THEN** 系统 MUST NOT 对本步 Runtime 做压缩

#### Scenario: Circuit breaker on compress failure
- **WHEN** 无法降载且连续失败达到熔断阈值
- **THEN** 停止重试并给出明确失败

#### Scenario: Trajectory marks compress not append
- **WHEN** 本步应用了任一压缩策略
- **THEN** 追加 `compact/checkpoint`（含 `strategies`）；`request/header.delta` MUST 含 `compressed: true` 与生效 `strategy`；若含 `reset` 则另有 `reset: true`；账本 MUST NOT 将该步标为普通「追加」

### Requirement: Skills Progressive Disclosure

Skill 加载 SHALL 对齐 Agent Skills 渐进式披露：**L0 目录**（name + description）常驻可发现，**L1 正文**仅在 harness 检索选中后装载。`build_system_prompt` MUST NOT 再把 Skill 全文写入系统提示词。L0 经 `skills/catalog`、L1 经 `context/snapshot(source=skills)` 分条落库。无可用的通用 Rust 渐进披露运行时包时，允许 harness 自研实现，但语义 MUST 与上述规范一致。

#### Scenario: System prompt excludes skill bodies
- **WHEN** 小模型或默认策略渲染系统提示且用户问题可命中 Skill
- **THEN** `system/message` / `build_*_prompt` 输出 MUST NOT 含「相关技能说明」或 Skill 全文块

#### Scenario: Catalog and bodies logged separately
- **WHEN** 本回合启用 Skill 非空且检索命中 ≥1 条
- **THEN** 存在 `skills/catalog` 与 `context/snapshot(source=skills)`；发给模型的上下文可含 L0+L1；账本将二者与静态系统提示词同组、分条展示（系统提示词行不含 Skill 全文）

#### Scenario: Skill retrieval prefers natural-language intent over payload
- **WHEN** 用户消息含大段 JSON/DSL 粘贴，且**前部或后部**有自然语言意图（如「请格式化下面这段」「转义一下」）
- **THEN** `retrieve_skills` MUST 主要依据全文中的意图句打分（而非固定只看末尾），MUST NOT 仅因粘贴正文内出现泛词（如字段名 `query`）把 `json.to_query` / `url.parse` 排到「转义/格式化」类 Skill 之前

#### Scenario: json.format accepts escaped input
- **WHEN** 工具 `json.format` 收到已转义（`{\"a\":1}`）或模型双重转义后的 JSON 字符串
- **THEN** 工具 MUST 尝试宽松还原后再美化，MUST NOT 仅因一层多余转义就返回「key must be a string」

#### Scenario: json.format ignores trailing junk after first value
- **WHEN** 工具 `json.format` 的 `input` 在第一个完整 JSON 值之后仍有多余字符（如小模型多写的 `}`）
- **THEN** 工具 MUST 取第一个完整值并成功美化，MUST NOT 因 `trailing characters` 失败
