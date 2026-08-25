//! 由注册表运行时生成 GBNF 语法：约束解码保证 `<tool_call>` 块结构合法
//! （工具名 ∈ 注册表枚举，arguments 为松散 JSON——参数正确性交给
//! schema 校验 + reask，见 design.md D3/D4）。
//!
//! 注：llama.cpp GBNF 规则名仅支持 `[a-zA-Z][a-zA-Z0-9]*`，**不接受下划线**
//! （已踩坑：`tool_call`/`json_object` 等会被解析器在 `_` 处截断为非法
//! 标识符，触发 `expecting newline or end at _call`）。所有自定义规则名
//! 用 camelCase；触发词内的下划线 `<tool_call>` 仅作为字面量，无关规则名。

use crate::agent::registry;

const TRIGGER: &str = "<tool_call>";

/// 生成「任意文本 ∪ 合法 toolcall 块」的 GBNF 根语法。
pub fn tool_call_grammar() -> String {
    let names: Vec<String> = registry::all_tools()
        .iter()
        .map(|t| format!("\"{}\"", escape(t.id)))
        .collect();

    let chars: Vec<char> = TRIGGER.chars().collect();
    let mut other_alts: Vec<String> = vec!["[^<]+".to_string()];
    // 触发词每个真前缀后跟一个不匹配下一字符的字符（GBNF 标准防误匹配技巧）：
    // 否则模型采样到部分触发词前缀时 grammar 会因后续字符非法而拒绝采样。
    for i in 0..chars.len() - 1 {
        let prefix: String = chars[..=i].iter().collect();
        let next = chars[i + 1];
        other_alts.push(format!("\"{}\" [^{}]", escape(&prefix), next));
    }
    let other_rule = other_alts.join(" | ");

    format!(
        r#"root ::= (other | toolcall)*
toolcall ::= "<tool_call>" tcobject "</tool_call>"
tcobject ::= "{{" ws "\"name\"" ws ":" ws namestr ws "," ws "\"arguments\"" ws ":" ws jsonobject ws "}}"
namestr ::= {names}
other ::= {other_rule}
jsonobject ::= "{{" ws (jsonmember (ws "," ws jsonmember)*)? ws "}}"
jsonmember ::= jsonstring ws ":" ws jsonvalue
jsonvalue ::= jsonobject | jsonarray | jsonstring | jsonnumber | "true" | "false" | "null"
jsonarray ::= "[" ws (jsonvalue (ws "," ws jsonvalue)*)? ws "]"
jsonstring ::= "\"" chars "\""
chars ::= char*
char ::= [^"\\\x00-\x1F] | "\\" escape
escape ::= ["\\/bfnrt] | "u" hex hex hex hex
hex ::= [0-9a-fA-F]
jsonnumber ::= "-"? int frac? exp?
int ::= [0-9]+ | [1-9] [0-9]*
frac ::= "." [0-9]+
exp ::= [eE] [+-]? [0-9]+
ws ::= [ \t\n]*
"#,
        names = names.join(" | "),
        other_rule = other_rule,
    )
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// 语法与注册表一致性：包含全部工具 id。
pub fn grammar_covers_registry(grammar: &str) -> bool {
    registry::all_tools()
        .iter()
        .all(|t| grammar.contains(&format!("\"{}\"", t.id)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grammar_contains_all_tool_names() {
        let g = tool_call_grammar();
        assert!(grammar_covers_registry(&g));
        assert!(g.contains("base64.encode"));
        assert!(g.contains("uuid.generate"));
        // 触发词与 JSON 结构规则存在
        assert!(g.contains(r#""<tool_call>""#));
        assert!(g.contains("jsonobject"));
        // other 规则带规则名前缀（曾因丢失前缀导致 GBNF 解析失败）
        assert!(g.contains("other ::="));
        // 非触发词分支覆盖（示例：`"<too" [^l]`）
        assert!(g.contains(r#""<too" [^l]"#));
        // 规则定义名 camelCase（GBNF 解析器不接受下划线）：
// 出现在字符串字面量内的 `tool_call`（触发词 `<tool_call>`）是允许的。
        let rule_defs = [
            "toolcall ::=",
            "tcobject ::=",
            "namestr ::=",
            "jsonobject ::=",
            "jsonmember ::=",
            "jsonvalue ::=",
        ];
        for r in rule_defs {
            assert!(g.contains(r), "rule definition {r} missing");
        }
    }

    #[test]
    fn grammar_excludes_unregistered() {
        let g = tool_call_grammar();
        assert!(!g.contains("\"http.request\""));
    }

    /// 用本机已安装 GGUF 实测 grammar 能否被 llama-cpp-2 成功解析（防回归）。
    /// 无权重时跳过；不计入逻辑层 `cargo test` 的硬指标。
    #[test]
    fn grammar_parses_with_installed_model() {
        let Some(path) = crate::commands::model_catalog::enabled_model_path() else {
            eprintln!("SKIP grammar_parses: no installed GGUF");
            return;
        };
        let backend = llama_cpp_2::llama_backend::LlamaBackend::init().expect("backend");
        let params = llama_cpp_2::model::params::LlamaModelParams::default();
        let model = llama_cpp_2::model::LlamaModel::load_from_file(&backend, &path, &params)
            .expect("model load");
        let g = tool_call_grammar();
        if let Err(e) = llama_cpp_2::sampling::LlamaSampler::grammar(&model, &g, "root") {
            panic!("grammar parse failed: {e:?}\n---\n{g}\n---");
        }
    }
}