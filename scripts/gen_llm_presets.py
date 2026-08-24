#!/usr/bin/env python3
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
data = json.loads((ROOT / "src-tauri/src/commands/llm_presets_data.json").read_text())


def map_api(a: str) -> str:
    return {
        "anthropic": "AnthropicMessages",
        "openai_chat": "OpenaiChat",
        "openai_responses": "OpenaiResponses",
        "gemini_native": "GeminiNative",
    }.get(a, "AnthropicMessages")


def esc(s: str) -> str:
    return s.replace("\\", "\\\\").replace('"', '\\"')


lines = [
    "//! CC Switch Claude preset snapshot + TBox-owned providers.",
    "//! Upstream: farion1231/cc-switch main claudeProviderPresets.ts (2026-08-22).",
    "",
    "use serde::Serialize;",
    "use super::llm::{LlmProtocol, LlmPresetView};",
    "",
    "#[derive(Debug, Clone, Serialize)]",
    "pub struct LlmPreset {",
    "    pub id: &'static str,",
    "    pub label: &'static str,",
    "    pub default_base_url: &'static str,",
    "    pub default_model: &'static str,",
    "    pub default_protocol: LlmProtocol,",
    "    pub requires_oauth: bool,",
    "}",
    "",
    "const TBOX_OWN: &[LlmPreset] = &[",
    '    LlmPreset { id: "local", label: "本地（内置 sidecar）", default_base_url: "", default_model: "", default_protocol: LlmProtocol::OpenaiChat, requires_oauth: false },',
    '    LlmPreset { id: "ollama", label: "Ollama", default_base_url: "http://127.0.0.1:11434", default_model: "llama3.2", default_protocol: LlmProtocol::OllamaNative, requires_oauth: false },',
    '    LlmPreset { id: "custom", label: "自定义端点", default_base_url: "", default_model: "", default_protocol: LlmProtocol::OpenaiChat, requires_oauth: false },',
    "];",
    "",
    "const CC_SWITCH: &[LlmPreset] = &[",
]
for e in data:
    proto = map_api(e["api"])
    oauth = "true" if e["oauth"] else "false"
    lines.append(
        f'    LlmPreset {{ id: "{esc(e["id"])}", label: "{esc(e["label"])}", '
        f'default_base_url: "{esc(e["base"])}", default_model: "{esc(e["model"])}", '
        f"default_protocol: LlmProtocol::{proto}, requires_oauth: {oauth} }},"
    )
lines += [
    "];",
    "",
    "pub fn all_presets() -> Vec<LlmPresetView> {",
    "    let mut out = Vec::with_capacity(TBOX_OWN.len() + CC_SWITCH.len());",
    "    for p in TBOX_OWN.iter().chain(CC_SWITCH.iter()) {",
    "        out.push(LlmPresetView::from_preset(p));",
    "    }",
    "    out",
    "}",
    "",
    "pub fn find_preset(id: &str) -> Option<&'static LlmPreset> {",
    "    TBOX_OWN.iter().chain(CC_SWITCH.iter()).find(|p| p.id == id)",
    "}",
    "",
    "pub fn default_protocol_for_provider(id: &str) -> LlmProtocol {",
    "    find_preset(id)",
    "        .map(|p| p.default_protocol)",
    "        .unwrap_or_else(|| infer_legacy_protocol(id))",
    "}",
    "",
    "fn infer_legacy_protocol(id: &str) -> LlmProtocol {",
    "    match id {",
    '        "local" => LlmProtocol::OpenaiChat,',
    '        "ollama" => LlmProtocol::OllamaNative,',
    '        "openai" | "deepseek" | "custom" => LlmProtocol::OpenaiChat,',
    '        "anthropic" => LlmProtocol::AnthropicMessages,',
    "        _ => LlmProtocol::AnthropicMessages,",
    "    }",
    "}",
    "",
]

out = ROOT / "src-tauri/src/commands/llm_presets.rs"
out.write_text("\n".join(lines) + "\n")
print(f"wrote {out} ({len(data)} cc presets)")
