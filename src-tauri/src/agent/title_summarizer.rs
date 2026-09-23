//! Independent Title Summarizer: one-shot completion, no agent/skills/tools.
//!
//! Pipeline: first-sentence seed → (optional) LLM compress to core noun phrase → sanitize.

use crate::agent::llm::{build_model_from_disk, split_think_tags, ModelMessage, ModelTurn};

const SYSTEM_PROMPT: &str = "\
你是会话标题提取器。输入是用户消息的第一句话（或其核心片段）。
任务：提取这句话的核心意图/对象，写成一个短标题。

硬性规则：
1. 只输出标题本身，一行，不要引号、不要解释、不要句号、不要标签或思考过程。
2. 中文优先，不超过 16 个字；专有名词/工具名可保留原文（如 Base64、JWT、UUID）。
3. 去掉礼貌与过程套话：请/帮我/麻烦/请问/我想/能不能/如何/怎么/一下 等。
4. 不要复述整句；用名词短语概括，例如「JWT 解析」「Base64 编码 hello」。
5. 若输入已是短核心短语，可轻微整理后原样输出，不要编造未出现的主题。";

const INPUT_LIMIT: usize = 200;
/// Max chars for LLM-produced titles.
const OUTPUT_LIMIT: usize = 16;
/// Placeholders / seeds may be slightly longer before summarizer.
const PLACEHOLDER_LIMIT: usize = 40;
/// If the first-sentence core is at most this long, skip LLM and use it as-is.
const SHORT_TITLE_LIMIT: usize = 16;

/// Politeness / process prefixes common in Chinese (and a few English).
const LEADING_FILLERS: &[&str] = &[
    "请问一下",
    "请问您",
    "请问",
    "麻烦您帮我",
    "麻烦你帮我",
    "麻烦帮我",
    "麻烦您",
    "麻烦你",
    "麻烦",
    "请帮我一下",
    "请帮我",
    "请帮忙",
    "帮我一下",
    "帮我",
    "帮忙",
    "我想要",
    "我想请问",
    "我想问一下",
    "我想问",
    "我想",
    "我需要",
    "能不能帮我",
    "能不能",
    "可不可以",
    "可以帮我",
    "如何才能",
    "如何",
    "怎么才能",
    "怎么样",
    "怎么做",
    "怎么",
    "怎样",
    "请你",
    "请",
    "Could you please ",
    "Could you ",
    "Can you please ",
    "Can you ",
    "Please help me ",
    "Please ",
    "I want to ",
    "I need to ",
    "I'd like to ",
    "Help me ",
];

fn is_mock() -> bool {
    std::env::var("TBOX_AGENT_MOCK")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn looks_like_think_tag(s: &str) -> bool {
    let t = s.trim();
    if t.is_empty() {
        return false;
    }
    let lower = t.to_ascii_lowercase();
    lower == "<think>"
        || lower == "</think>"
        || lower.starts_with("<think>")
        || lower.starts_with("</think>")
}

fn is_sentence_break(c: char) -> bool {
    matches!(
        c,
        '。' | '！' | '？' | '；' | '.' | '!' | '?' | ';' | '\n' | '\r'
    )
}

/// Take the first sentence of `text` (CJK / Latin punctuation + newlines).
pub fn first_sentence(text: &str) -> &str {
    let t = text.trim();
    if t.is_empty() {
        return t;
    }
    for (i, ch) in t.char_indices() {
        if is_sentence_break(ch) {
            let end = if i == 0 { ch.len_utf8() } else { i };
            let s = t[..end].trim();
            if !s.is_empty() {
                return s;
            }
        }
    }
    t
}

fn strip_leading_fillers(s: &str) -> String {
    let mut out = s.trim().to_string();
    // Repeat: nested fillers like「请帮我请问…」
    for _ in 0..4 {
        let before = out.clone();
        let lower = out.to_ascii_lowercase();
        let mut stripped = false;
        for filler in LEADING_FILLERS {
            let f_lower = filler.to_ascii_lowercase();
            if lower.starts_with(&f_lower) {
                out = out.chars().skip(filler.chars().count()).collect::<String>();
                out = out.trim_start_matches(['，', ',', '、', ':', '：', ' ']).trim().to_string();
                stripped = true;
                break;
            }
        }
        if !stripped || out == before {
            break;
        }
    }
    out
}

/// Deterministic seed for titles: first sentence → strip fillers → length cap.
/// Used as placeholder and as LLM input so summarizer only sees the core clause.
pub fn extract_title_seed(text: &str) -> String {
    let sentence = first_sentence(text);
    let core = strip_leading_fillers(sentence);
    let seed = if core.is_empty() {
        sentence.trim().to_string()
    } else {
        core
    };
    seed.chars().take(PLACEHOLDER_LIMIT).collect()
}

/// `true` when first-sentence core is short enough to use as the final title (no LLM).
pub fn is_short_title_seed(text: &str) -> bool {
    let seed = extract_title_seed(text);
    !seed.is_empty() && seed.chars().count() <= SHORT_TITLE_LIMIT
}

/// Clean model output into a short title.
pub fn sanitize_title(raw: &str) -> String {
    // Reasoning models often emit `<think>…</think>` first — strip before first line.
    let (_, body) = split_think_tags(raw);
    let mut s = body.trim().to_string();
    if s.is_empty() {
        return s;
    }
    if s.starts_with("```") {
        s = s
            .lines()
            .skip(1)
            .take_while(|l| !l.trim().starts_with("```"))
            .collect::<Vec<_>>()
            .join(" ");
    }
    if let Some(line) = s.lines().find(|l| !l.trim().is_empty()) {
        s = line.to_string();
    }
    s = s
        .trim()
        .trim_matches(|c| c == '"' || c == '\'' || c == '「' || c == '」' || c == '《' || c == '》')
        .to_string();
    s = s.trim_start_matches(['#', '*', '-', ' ']).trim().to_string();
    // Drop trailing sentence punctuation on titles.
    s = s
        .trim_end_matches(['。', '！', '？', '.', '!', '?', '；', ';'])
        .trim()
        .to_string();
    s = strip_leading_fillers(&s);
    if looks_like_think_tag(&s) {
        return String::new();
    }
    // Reject if model echoed a long multi-clause dump (prefer None → keep seed).
    if s.chars().count() > OUTPUT_LIMIT + 8 {
        s = s.chars().take(OUTPUT_LIMIT).collect();
    } else {
        s = s.chars().take(OUTPUT_LIMIT).collect();
    }
    s
}

/// Summarize the first user message into a short title.
/// Returns `None` on empty/failure so caller keeps the truncate placeholder.
///
/// If the first-sentence core is already short (`≤ SHORT_TITLE_LIMIT` chars),
/// skips the LLM and returns that seed directly.
pub fn summarize_title(first_user_message: &str) -> Option<String> {
    let seed = extract_title_seed(first_user_message);
    if seed.is_empty() {
        return None;
    }

    // Short first sentence: use as title without an extra LLM round-trip.
    if seed.chars().count() <= SHORT_TITLE_LIMIT {
        let direct = sanitize_title(&seed);
        return if direct.is_empty() { None } else { Some(direct) };
    }

    if is_mock() {
        let stub = format!("标题:{}", seed.chars().take(12).collect::<String>());
        let cleaned = sanitize_title(&stub);
        return if cleaned.is_empty() { None } else { Some(cleaned) };
    }

    // Only feed the first-sentence core — keeps the model focused.
    let input: String = seed.chars().take(INPUT_LIMIT).collect();
    let msgs = [
        ModelMessage::system(SYSTEM_PROMPT),
        ModelMessage::user(format!("第一句核心：{input}")),
    ];

    let mut model = build_model_from_disk().ok()?;
    let turn = model.complete(&msgs).ok()?;
    let text = match turn {
        ModelTurn::Text { text, .. } => text,
        ModelTurn::ToolCalls(_) => return None,
    };
    let cleaned = sanitize_title(&text);
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_sentence_stops_at_cjk_period() {
        assert_eq!(
            first_sentence("请帮我做 Base64 编码。然后算哈希。"),
            "请帮我做 Base64 编码"
        );
    }

    #[test]
    fn extract_seed_strips_politeness() {
        let seed = extract_title_seed("请帮我把这段 JWT 解析一下，顺便验签。后面还有别的。");
        assert!(seed.contains("JWT"));
        assert!(!seed.contains("请帮我"));
        assert!(!seed.contains("后面还有"));
    }

    #[test]
    fn extract_seed_english() {
        let seed = extract_title_seed("Please help me encode hello with Base64. Then hash it.");
        assert!(seed.to_ascii_lowercase().contains("base64") || seed.contains("encode"));
        assert!(!seed.to_ascii_lowercase().starts_with("please"));
    }

    #[test]
    fn sanitize_takes_first_line_and_strips_quotes() {
        assert_eq!(sanitize_title("  \"你好世界\"  \n第二行"), "你好世界");
    }

    #[test]
    fn sanitize_strips_fence() {
        let raw = "```\nBase64 编码\n```";
        assert_eq!(sanitize_title(raw), "Base64 编码");
    }

    #[test]
    fn sanitize_hard_cap() {
        let long: String = "字".repeat(80);
        assert_eq!(sanitize_title(&long).chars().count(), OUTPUT_LIMIT);
    }

    #[test]
    fn sanitize_strips_think_block_before_title() {
        let raw = "<think>\n用户想问 Base64\n</think>\nBase64 编码";
        assert_eq!(sanitize_title(raw), "Base64 编码");
    }

    #[test]
    fn sanitize_rejects_bare_think_tag() {
        assert_eq!(sanitize_title("<think>"), "");
        assert_eq!(sanitize_title("<think>\n只写了一半"), "");
    }

    #[test]
    fn short_seed_skips_llm_path() {
        // Short seed returns before any LLM / mock check.
        let t = summarize_title("JWT 解析").unwrap();
        assert_eq!(t, "JWT 解析");
        assert!(is_short_title_seed("JWT 解析"));
    }

    #[test]
    fn short_after_filler_strip_skips_llm() {
        let t = summarize_title("请帮我 JWT 解析").unwrap();
        assert_eq!(t, "JWT 解析");
        assert!(t.chars().count() <= SHORT_TITLE_LIMIT);
        assert!(is_short_title_seed("请帮我 JWT 解析"));
    }

    #[test]
    fn long_seed_is_not_short() {
        let msg = "请帮我把这段非常非常长的用户输入内容做 Base64 编码处理一下。再哈希。";
        assert!(!is_short_title_seed(msg));
    }

    #[test]
    fn mock_summarize_uses_seed() {
        std::env::set_var("TBOX_AGENT_MOCK", "1");
        let msg = "请帮我把这段非常非常长的用户输入内容做 Base64 编码处理一下。再哈希。";
        assert!(!is_short_title_seed(msg));
        let t = summarize_title(msg).expect("mock summarizer");
        assert!(t.starts_with("标题:"));
        assert!(!t.contains("再哈希"));
        std::env::remove_var("TBOX_AGENT_MOCK");
    }
}
