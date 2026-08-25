# 约束解码实测记录（任务 4.3）

## 结论

`llama.cpp` 0.5B Instruct + 约束解码（GBNF）会**显著降低意图命中率**。Harness
默认对 `SmallLocalStrategy` 关闭约束解码（`constrained=false`），保留 grammar
基础设施与 `TBOX_ENABLE_TOOL_GRAMMAR=1` 环境变量供手工开启。

## 实测数据

评测集：`src-tauri/src/agent/harness/eval/cases.json`（33 条 golden cases）。
后端：embedded（`llama-cpp-2`）+ 本机已下载 `qwen2.5-0.5b-instruct-q4_k_m.gguf`。
命令：`cargo test --lib --features agent-eval agent::harness::eval::e2e -- --nocapture`。

| 配置                 | intent hit | tool correct | args valid | e2e success | 总耗时 |
|----------------------|-----------:|-------------:|-----------:|------------:|-------:|
| **Grammar OFF**（默认）| 21 / 63.6% | 15 / 45.5%   | 15 / 45.5% | 28 / 84.8%  | **52.3s** |
| Grammar ON（GBNF 强制）|  4 / 12.1% |  0 / 0.0%    |  0 / 0.0%  | 33 / 100.0% | 90.0s  |

注：Grammar ON 的 `tool correct=0%` 是因为模型被 grammar 强制过早进入
`<tool_call>` 形态，对负例（闲聊类）也会产出 tool_call（往往是 `uuid.generate`），
dispatch 走通导致 `e2e_success=100%` 但意图命中归零。

## 分析

grammar 根 `root ::= (other | toolcall)*` 设计上允许任意文本 ∪ 工具调用，但
0.5B 模型在采样阶段一旦命中 `toolcall` 分支就会被拉入生成 `<tool_call>` 字面
量的轨道，对负例问题（如「你好」「今天天气怎么样」）无法给出纯文本回答。同时
grammar 的 JSON 语法只约束 arguments 结构，不约束参数值域（如 `algorithm` 应为
`md5`/`sha256`），因此常见 schema 错误仍依赖 reask 修复。

## 后续路径

- grammar 基础设施（生成器 + sampler 通道）保留，待：
  - 更大模型（如 1.5B Instruct/3B）实测有益时切回默认；
  - GBNF 进一步约束参数值域（当前 arguments 为松散 JSON，参数正确性交给
    schema 校验 + reask）；
  - 出现更智能的「延迟约束」（grammar_lazy + trigger）方案时再评估。
- 设计文档 D3 已经把 grammar 标记为「增强项、可降级」，本笔记作为 D3 的实证。