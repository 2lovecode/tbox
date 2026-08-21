//! 预置 Skill 检索：按关键词匹配工具说明书，不扩展注册表。

/// 检索命中的 Skill 文档（tool_id + 正文）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillDoc {
    pub tool_id: String,
    pub body: String,
}

struct EmbeddedSkill {
    source: &'static str,
}

static SKILLS: &[EmbeddedSkill] = &[
    EmbeddedSkill {
        source: include_str!("../../skills/json.format.md"),
    },
    EmbeddedSkill {
        source: include_str!("../../skills/base64.md"),
    },
    EmbeddedSkill {
        source: include_str!("../../skills/hash.digest.md"),
    },
    EmbeddedSkill {
        source: include_str!("../../skills/jwt.parse.md"),
    },
    EmbeddedSkill {
        source: include_str!("../../skills/timestamp.convert.md"),
    },
    EmbeddedSkill {
        source: include_str!("../../skills/encoding.convert.md"),
    },
    EmbeddedSkill {
        source: include_str!("../../skills/xml.format.md"),
    },
    EmbeddedSkill {
        source: include_str!("../../skills/yaml.format.md"),
    },
    EmbeddedSkill {
        source: include_str!("../../skills/uuid.generate.md"),
    },
    EmbeddedSkill {
        source: include_str!("../../skills/cron.explain.md"),
    },
    EmbeddedSkill {
        source: include_str!("../../skills/number.convert.md"),
    },
    EmbeddedSkill {
        source: include_str!("../../skills/charset.convert.md"),
    },
];

struct ParsedSkill {
    tool_ids: Vec<String>,
    keywords: Vec<String>,
    body: String,
}

fn parse_skill(source: &str) -> Option<ParsedSkill> {
    let rest = source.strip_prefix("---\n")?;
    let end = rest.find("\n---\n")?;
    let front_matter = &rest[..end];
    let body = rest[end + 5..].trim().to_string();

    let mut tool_ids = Vec::new();
    let mut keywords = Vec::new();

    for line in front_matter.lines() {
        if let Some(value) = line.strip_prefix("tool_id:") {
            for id in value.split(',') {
                let id = id.trim();
                if !id.is_empty() {
                    tool_ids.push(id.to_string());
                }
            }
        } else if let Some(value) = line.strip_prefix("keywords:") {
            for kw in value.split(',') {
                let kw = kw.trim();
                if !kw.is_empty() {
                    keywords.push(kw.to_string());
                }
            }
        }
    }

    if tool_ids.is_empty() {
        return None;
    }

    Some(ParsedSkill {
        tool_ids,
        keywords,
        body,
    })
}

fn contains_match(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    if needle.is_ascii() {
        haystack
            .to_ascii_lowercase()
            .contains(&needle.to_ascii_lowercase())
    } else {
        haystack.contains(needle)
    }
}

fn score_skill(query: &str, parsed: &ParsedSkill) -> i32 {
    let mut score = 0;

    for kw in &parsed.keywords {
        if contains_match(query, kw) {
            score += 1;
        }
    }

    for tool_id in &parsed.tool_ids {
        if contains_match(query, tool_id) {
            score += 3;
        }
        for segment in tool_id.split('.') {
            if segment.len() >= 2 && contains_match(query, segment) {
                score += 2;
            }
        }
    }

    score
}

/// 按 query 关键词/工具名/中文别名匹配预置 Skill，最多返回 `limit` 条。
pub fn retrieve_skills(query: &str, limit: usize) -> Vec<SkillDoc> {
    if limit == 0 || query.trim().is_empty() {
        return Vec::new();
    }

    let mut ranked: Vec<(i32, SkillDoc)> = SKILLS
        .iter()
        .filter_map(|embedded| {
            let parsed = parse_skill(embedded.source)?;
            let score = score_skill(query, &parsed);
            if score <= 0 {
                return None;
            }
            Some((
                score,
                SkillDoc {
                    tool_id: parsed.tool_ids[0].clone(),
                    body: parsed.body,
                },
            ))
        })
        .collect();

    ranked.sort_by(|a, b| b.0.cmp(&a.0));
    ranked.truncate(limit);
    ranked.into_iter().map(|(_, doc)| doc).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::registry::lookup;

    #[test]
    fn jwt_query_hits_jwt_skill() {
        let hits = retrieve_skills("帮我解析这段 JWT", 3);
        assert!(hits.iter().any(|s| s.tool_id.contains("jwt")));
        assert!(hits.len() <= 3);
    }

    #[test]
    fn loading_skills_does_not_register_tools() {
        let before = lookup("http.request");
        let _ = retrieve_skills("http", 3);
        assert!(before.is_none());
        assert!(lookup("http.request").is_none());
    }
}
