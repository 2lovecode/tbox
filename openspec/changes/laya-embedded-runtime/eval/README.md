# 数值一致性验收结果（任务 2.3）

## 结论：**通过**

| 指标 | 结果 | 验收标准 |
|------|------|---------|
| choice 一致率 | **50/50 = 1.000** | ≥ 0.9 ✅ |
| 置信度平均绝对偏差 | **0.0000**（对齐 `answer_confidence` 口径后精确一致） | ≤ 0.05 ✅ |

- 测试：`src-tauri/src/commands/laya.rs::tests::parity_against_python_reference`（`#[ignore]`，需模型部署于 `~/.toolbox/laya`；`cargo test --lib laya -- --ignored parity` 运行，约 30s）
- 参考：Python `Agent`（eager PyTorch，multilingual，max_len=1024）逐条输出 → `python-reference-outputs.json`
- 评测集：复用 `laya-spotlight-tool-routing` change 的 50 条中文查询

## 复刻过程中的关键修正（对齐根因记录）

1. **候选顺序必须保持插入序**：`serde_json::Map` 默认 BTreeMap 会把工具 id 按字典序排序（"1","10","11",…"2"），而 laya 默认 sequential layout 的选项嵌入是**顺序敏感**的（官方注释：五个相同选项的 slot logits 相差 3.59），顺序改变导致 logits 全变（修正前一致率仅 0.82）。
   修复：`serde_json` 启用 `preserve_order` feature（同时修正了 external 模式发给 laya-serve 的候选顺序）。
2. **选项文本 = `"key: value"`**：`render_options` choice 分支渲染为 `"9: JSON处理工具|…"`，不是裸 value。
3. **特殊 token**：mmBERT 是 XLM-R 风格（`<bos>`/`<eos>`/`<mask>`），非 BERT 的 `[CLS]`/`[SEP]`/`[MASK]`。
4. **marker_mask 是 bool 张量**（导出图 `dtype=torch.bool`）。
5. **置信度口径**：`confidence` 字段是归一化熵置信度，`answer_confidence = max(p)` 才是温度校准后的命中置信（与 TBox 阈值语义一致），Rust 端返回后者。

## 复现

```bash
# 模型已部署 ~/.toolbox/laya（onnx + onnx.data + tokenizer.json + rl_agent_config.json）
cd src-tauri && cargo test --lib laya -- --ignored parity --nocapture
```
