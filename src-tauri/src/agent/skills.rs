//! 预置 Skill 检索：按关键词匹配工具说明书，不扩展注册表。
//! 用户可在设置中禁用单项；禁用只影响检索注入。

/// 检索命中的 Skill 文档（渐进披露 L1：skill_id + 可选 Agent 工具列表 + 正文）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillDoc {
    pub skill_id: String,
    pub tool_ids: Vec<String>,
    pub body: String,
}

/// Agent Skills L0 目录项（name + description 常驻，正文按需）。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillCatalogEntry {
    pub id: String,
    pub name: String,
    pub description: String,
}

/// 设置页展示的内置 / 用户 Skill 目录项。
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillInfo {
    pub id: String,
    pub name: String,
    pub tool_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub toolbox_ids: Vec<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,
    pub description: String,
    pub source: String,
    pub body: String,
    pub enabled: bool,
    pub editable: bool,
    pub modified: bool,
    pub can_restore_default: bool,
}

/// 版本历史条目。
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillVersionInfo {
    pub seq: u64,
    pub saved_at: String,
    pub preview: String,
}

struct EmbeddedSkill {
    id: &'static str,
    source: &'static str,
}

static SKILLS: &[EmbeddedSkill] = &[
    EmbeddedSkill {
        id: "crypto",
        source: include_str!("../../skills/crypto.md"),
    },
    EmbeddedSkill {
        id: "datetime-id",
        source: include_str!("../../skills/datetime-id.md"),
    },
    EmbeddedSkill {
        id: "encoding",
        source: include_str!("../../skills/encoding.md"),
    },
    EmbeddedSkill {
        id: "json",
        source: include_str!("../../skills/json.md"),
    },
    EmbeddedSkill {
        id: "markup",
        source: include_str!("../../skills/markup.md"),
    },
    EmbeddedSkill {
        id: "os.shell",
        source: include_str!("../../skills/os.shell.md"),
    },
    EmbeddedSkill {
        id: "toolbox.design",
        source: include_str!("../../skills/toolbox.design.md"),
    },
    EmbeddedSkill {
        id: "toolbox.dev",
        source: include_str!("../../skills/toolbox.dev.md"),
    },
    EmbeddedSkill {
        id: "toolbox.media",
        source: include_str!("../../skills/toolbox.media.md"),
    },
    EmbeddedSkill {
        id: "toolbox.network",
        source: include_str!("../../skills/toolbox.network.md"),
    },
    EmbeddedSkill {
        id: "toolbox.recovery",
        source: include_str!("../../skills/toolbox.recovery.md"),
    },
    EmbeddedSkill {
        id: "toolbox.security",
        source: include_str!("../../skills/toolbox.security.md"),
    },
];

struct ParsedSkill {
    name: Option<String>,
    /// L0 路由描述（Agent Skills：常驻目录用；正文按需加载）。
    description: Option<String>,
    tool_ids: Vec<String>,
    toolbox_ids: Vec<u32>,
    keywords: Vec<String>,
    avoid_keywords: Vec<String>,
    body: String,
}

fn parse_skill(source: &str) -> Option<ParsedSkill> {
    let source = source.trim_start_matches('\u{feff}').replace("\r\n", "\n");
    let rest = source.strip_prefix("---\n")?;
    let end = rest.find("\n---\n")?;
    let front_matter = &rest[..end];
    let body = rest[end + 5..].trim().to_string();

    let mut name = None;
    let mut description = None;
    let mut tool_ids = Vec::new();
    let mut toolbox_ids = Vec::new();
    let mut keywords = Vec::new();
    let mut avoid_keywords = Vec::new();

    for line in front_matter.lines() {
        if let Some(value) = line.strip_prefix("name:") {
            let v = value.trim();
            if !v.is_empty() {
                name = Some(v.to_string());
            }
        } else if let Some(value) = line.strip_prefix("description:") {
            let v = value.trim();
            if !v.is_empty() {
                description = Some(v.to_string());
            }
        } else if let Some(value) = line.strip_prefix("tool_id:") {
            for id in value.split(',') {
                let id = id.trim();
                if !id.is_empty() {
                    tool_ids.push(id.to_string());
                }
            }
        } else if let Some(value) = line.strip_prefix("toolbox_id:") {
            for id in value.split(',') {
                let id = id.trim();
                if let Ok(n) = id.parse::<u32>() {
                    toolbox_ids.push(n);
                }
            }
        } else if let Some(value) = line.strip_prefix("keywords:") {
            for kw in value.split(',') {
                let kw = kw.trim();
                if !kw.is_empty() {
                    keywords.push(kw.to_string());
                }
            }
        } else if let Some(value) = line.strip_prefix("avoid_keywords:") {
            for kw in value.split(',') {
                let kw = kw.trim();
                if !kw.is_empty() {
                    avoid_keywords.push(kw.to_string());
                }
            }
        }
    }

    // 至少关联一个 Agent 工具或工具箱工具
    if tool_ids.is_empty() && toolbox_ids.is_empty() {
        return None;
    }

    Some(ParsedSkill {
        name,
        description,
        tool_ids,
        toolbox_ids,
        keywords,
        avoid_keywords,
        body,
    })
}

/// L0 描述：优先 frontmatter `description`，否则从正文「何时使用」推导。
fn resolve_l0_description(parsed: &ParsedSkill) -> String {
    if let Some(d) = parsed.description.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        return d.to_string();
    }
    skill_routing_description(&parsed.body)
}

const SKILL_PREFS_NAME: &str = "skills.json";
const CUSTOM_SKILLS_DIR: &str = "skills/custom";
const OVERRIDE_SKILLS_DIR: &str = "skills/overrides";
const VERSION_SKILLS_DIR: &str = "skills/versions";

static TEST_PREFS: std::sync::Mutex<Option<Vec<String>>> = std::sync::Mutex::new(None);

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SkillPrefsFile {
    #[serde(default)]
    disabled_skills: Vec<String>,
}

fn skill_prefs_path() -> std::path::PathBuf {
    crate::agent::llama_log::toolbox_dir().join(SKILL_PREFS_NAME)
}

fn custom_skills_dir() -> std::path::PathBuf {
    crate::agent::llama_log::toolbox_dir().join(CUSTOM_SKILLS_DIR)
}

fn override_skills_dir() -> std::path::PathBuf {
    crate::agent::llama_log::toolbox_dir().join(OVERRIDE_SKILLS_DIR)
}

fn versions_dir(id: &str) -> std::path::PathBuf {
    crate::agent::llama_log::toolbox_dir()
        .join(VERSION_SKILLS_DIR)
        .join(id)
}

fn override_path(id: &str) -> std::path::PathBuf {
    override_skills_dir().join(format!("{id}.md"))
}

fn builtin_has_override(id: &str) -> bool {
    override_path(id).is_file()
}

fn read_override_raw(id: &str) -> Option<String> {
    std::fs::read_to_string(override_path(id)).ok()
}

/// 解析内置 Skill 的当前有效源（覆盖优先）。
fn resolve_builtin_raw(id: &str, embedded_source: &str) -> String {
    read_override_raw(id).unwrap_or_else(|| embedded_source.to_string())
}

fn next_version_seq(id: &str) -> u64 {
    let dir = versions_dir(id);
    if !dir.is_dir() {
        return 1;
    }
    let mut max = 0u64;
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let s = name.to_string_lossy();
            if let Some(n) = s.strip_suffix(".md").and_then(|n| n.parse::<u64>().ok()) {
                max = max.max(n);
            }
        }
    }
    max + 1
}

fn append_version_snapshot(id: &str, raw: &str) -> Result<(), String> {
    let dir = versions_dir(id);
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建版本目录失败: {e}"))?;
    let seq = next_version_seq(id);
    let path = dir.join(format!("{seq}.md"));
    let stamp = chrono_like_stamp();
    let content = format!("<!-- saved_at: {stamp} -->\n{raw}");
    std::fs::write(path, content).map_err(|e| format!("写入版本失败: {e}"))
}

fn chrono_like_stamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

fn version_preview(raw: &str) -> String {
    let body = raw
        .lines()
        .skip_while(|l| l.starts_with("<!--") || l.trim().is_empty())
        .take(3)
        .collect::<Vec<_>>()
        .join(" ");
    let chars: String = body.chars().take(120).collect();
    if body.chars().count() > 120 {
        format!("{chars}…")
    } else {
        chars
    }
}

fn serialize_builtin_override(
    name: Option<&str>,
    description: Option<&str>,
    tool_ids: &[String],
    toolbox_ids: &[u32],
    keywords: &[String],
    avoid_keywords: &[String],
    body: &str,
) -> String {
    let mut out = String::from("---\n");
    if let Some(n) = name.map(str::trim).filter(|s| !s.is_empty()) {
        out.push_str(&format!("name: {n}\n"));
    }
    if let Some(d) = description.map(str::trim).filter(|s| !s.is_empty()) {
        out.push_str(&format!("description: {d}\n"));
    }
    if !tool_ids.is_empty() {
        out.push_str(&format!("tool_id: {}\n", tool_ids.join(", ")));
    }
    if !toolbox_ids.is_empty() {
        let ids = toolbox_ids
            .iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("toolbox_id: {ids}\n"));
    }
    if !keywords.is_empty() {
        out.push_str(&format!("keywords: {}\n", keywords.join(", ")));
    }
    if !avoid_keywords.is_empty() {
        out.push_str(&format!(
            "avoid_keywords: {}\n",
            avoid_keywords.join(", ")
        ));
    }
    out.push_str("---\n\n");
    out.push_str(body.trim());
    out.push('\n');
    out
}

fn builtin_skill_info(id: &str, embedded_source: &str, enabled: bool) -> Option<SkillInfo> {
    let raw = resolve_builtin_raw(id, embedded_source);
    let parsed = parse_skill(&raw)?;
    let modified = builtin_has_override(id);
    let name = parsed
        .name
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| skill_title(&parsed.body, id));
    let description = resolve_l0_description(&parsed);
    Some(SkillInfo {
        id: id.to_string(),
        name,
        tool_ids: parsed.tool_ids,
        toolbox_ids: parsed.toolbox_ids,
        keywords: Some(parsed.keywords),
        description,
        source: "builtin".to_string(),
        body: parsed.body,
        enabled,
        editable: true,
        modified,
        can_restore_default: modified,
    })
}

#[derive(Debug, Default)]
struct UserSkillInput {
    name: String,
    description: String,
    keywords: Vec<String>,
    tool_ids: Vec<String>,
    body: String,
}

fn sanitize_id(value: &str) -> String {
    let id: String = value
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let id = id.split('-').filter(|part| !part.is_empty()).collect::<Vec<_>>().join("-");
    if id.is_empty() { "skill".to_string() } else { id.chars().take(64).collect() }
}

fn unique_user_skill_path(id: &str, exclude_id: Option<&str>) -> std::path::PathBuf {
    let dir = custom_skills_dir();
    let mut candidate = id.to_string();
    let mut index = 1;
    loop {
        let path = dir.join(format!("{candidate}.md"));
        let taken = path.exists() && Some(candidate.as_str()) != exclude_id;
        if !taken {
            return path;
        }
        index += 1;
        candidate = format!("{id}-{index}");
    }
}

fn parse_user_markdown(id: &str, raw: &str) -> Result<UserSkillInput, String> {
    if !raw.trim_start().starts_with("---") {
        return Err("用户 Skill 必须包含 YAML front matter".to_string());
    }
    let mut lines = raw.lines();
    if lines.next() != Some("---") {
        return Err("Skill 元数据格式错误".to_string());
    }
    let mut front_matter = String::new();
    let mut closed = false;
    for line in lines.by_ref() {
        if line.trim() == "---" {
            closed = true;
            break;
        }
        front_matter.push_str(line);
        front_matter.push('\n');
    }
    if !closed {
        return Err("Skill 元数据缺少结束标记".to_string());
    }

    let mut parsed = UserSkillInput {
        ..Default::default()
    };
    for line in front_matter.lines() {
        let Some((key, value)) = line.split_once(':') else { continue };
        let value = value.trim().trim_matches('"').trim_matches('\'').to_string();
        match key.trim() {
            "name" => parsed.name = value,
            "description" => parsed.description = value,
            "keywords" | "avoid_keywords" => {
                parsed.keywords.extend(value.split(',').map(str::trim).filter(|v| !v.is_empty()).map(String::from))
            }
            "tool_ids" | "tool_id" => {
                parsed.tool_ids.extend(value.split(',').map(str::trim).filter(|v| !v.is_empty()).map(String::from))
            }
            _ => {}
        }
    }
    parsed.body = lines.collect::<Vec<_>>().join("\n").trim().to_string();

    if parsed.name.trim().is_empty() {
        parsed.name = skill_title(&parsed.body, id);
    }
    if parsed.description.trim().is_empty() {
        parsed.description = skill_description(&format!("\n{}", parsed.body));
    }
    if parsed.keywords.is_empty() {
        parsed.keywords = parsed.name.split_whitespace().map(String::from).collect();
    }
    if parsed.body.trim().is_empty() {
        return Err("Skill 正文不能为空".to_string());
    }
    if parsed.tool_ids.iter().any(|id| crate::agent::registry::lookup(id).is_none()) {
        return Err("关联工具不存在；外部 Skill 不能注册新工具".to_string());
    }
    Ok(parsed)
}

fn serialize_user_markdown(skill: &UserSkillInput) -> String {
    let mut output = String::from("---\n");
    output.push_str(&format!("name: {}\n", skill.name.trim()));
    output.push_str(&format!("description: {}\n", skill.description.trim()));
    output.push_str(&format!("keywords: {}\n", skill.keywords.join(", ")));
    output.push_str(&format!("tool_ids: {}\n", skill.tool_ids.join(", ")));
    output.push_str("---\n\n");
    output.push_str(skill.body.trim());
    output.push('\n');
    output
}

fn user_skill_from_file(path: &std::path::Path, raw: &str, disabled: &std::collections::HashSet<String>) -> Option<SkillInfo> {
    let id = path.file_stem()?.to_string_lossy().to_string();
    let parsed = parse_user_markdown(&id, raw).ok()?;
    Some(user_skill_info(&id, parsed, disabled.contains(&id)))
}

fn user_skill_info(id: &str, parsed: UserSkillInput, enabled: bool) -> SkillInfo {
    SkillInfo {
        id: id.to_string(),
        name: parsed.name,
        tool_ids: parsed.tool_ids,
        toolbox_ids: Vec::new(),
        keywords: Some(parsed.keywords),
        description: parsed.description,
        source: "user".to_string(),
        body: parsed.body,
        enabled,
        editable: true,
        modified: false,
        can_restore_default: false,
    }
}

fn read_user_skills(disabled: &std::collections::HashSet<String>) -> Result<Vec<SkillInfo>, String> {
    let dir = custom_skills_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut skills = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(|e| format!("读取用户 Skill 失败: {e}"))? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let raw = std::fs::read_to_string(&path).map_err(|e| format!("读取 {} 失败: {e}", path.display()))?;
        skills.push(user_skill_from_file(&path, &raw, disabled).ok_or_else(|| format!("Skill 文件无效: {}", path.display()))?);
    }
    skills.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(skills)
}

fn write_user_skill(id: Option<&str>, input: UserSkillInput, imported: bool) -> Result<SkillInfo, String> {
    let base_id = sanitize_id(id.unwrap_or(&input.name));
    let dir = custom_skills_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建 Skill 目录失败: {e}"))?;

    let existing_id = id.filter(|id| {
        custom_skills_dir()
            .join(format!("{id}.md"))
            .exists()
    });
    let target = if imported {
        unique_user_skill_path(&base_id, existing_id.as_deref())
    } else {
        custom_skills_dir().join(format!("{}.md", existing_id.unwrap_or(&base_id)))
    };
    let final_id = target
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| base_id.clone());

    // 保存前快照
    if let Some(old_id) = &existing_id {
        let old_path = custom_skills_dir().join(format!("{old_id}.md"));
        if old_path.exists() {
            if let Ok(prev) = std::fs::read_to_string(&old_path) {
                let _ = append_version_snapshot(old_id, &prev);
            }
        }
    }

    let old_path = existing_id.map(|id| custom_skills_dir().join(format!("{id}.md")));
    if let (Some(old_path), false) = (&old_path, imported) {
        if old_path != &target {
            std::fs::remove_file(old_path).map_err(|e| format!("重命名 Skill 失败: {e}"))?;
        }
    }

    std::fs::write(&target, serialize_user_markdown(&input))
        .map_err(|e| format!("保存 Skill 失败: {e}"))?;
    let disabled = load_disabled_skills();
    Ok(user_skill_info(
        &final_id,
        input,
        !disabled.contains(&final_id),
    ))
}

fn load_disabled_skills() -> std::collections::HashSet<String> {
    if let Ok(prefs) = TEST_PREFS.lock() {
        if prefs.is_some() {
            let prefs = prefs.as_ref().expect("test prefs checked");
            return prefs.iter().cloned().collect();
        }
    }

    let raw = match std::fs::read(skill_prefs_path()) {
        Ok(raw) => raw,
        Err(_) => return std::collections::HashSet::new(),
    };
    serde_json::from_slice::<SkillPrefsFile>(&raw)
        .map(|prefs| prefs.disabled_skills.into_iter().collect())
        .unwrap_or_default()
}

fn save_disabled_skills(disabled: std::collections::HashSet<String>) -> Result<(), String> {
    if TEST_PREFS.lock().unwrap().is_some() {
        *TEST_PREFS.lock().unwrap() = Some(disabled.into_iter().collect());
        return Ok(());
    }

    let path = skill_prefs_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败: {e}"))?;
    }
    let file = SkillPrefsFile {
        disabled_skills: disabled.into_iter().collect(),
    };
    let raw = serde_json::to_vec_pretty(&file).map_err(|e| e.to_string())?;
    std::fs::write(path, raw).map_err(|e| format!("写入 Skill 配置失败: {e}"))
}

pub fn list_skills() -> Vec<SkillInfo> {
    let disabled = load_disabled_skills();
    let mut skills: Vec<SkillInfo> = SKILLS
        .iter()
        .filter_map(|embedded| {
            builtin_skill_info(embedded.id, embedded.source, !disabled.contains(embedded.id))
        })
        .collect();
    skills.extend(read_user_skills(&disabled).unwrap_or_default());
    skills
}

fn skill_title(source: &str, fallback: &str) -> String {
    source
        .lines()
        .find_map(|line| {
            let title = line.strip_prefix("# ")?.trim();
            if title.is_empty() {
                None
            } else {
                Some(title.to_string())
            }
        })
        .unwrap_or_else(|| fallback.to_string())
}

fn skill_description(source: &str) -> String {
    source
        .lines()
        .skip_while(|line| !line.trim().is_empty())
        .skip_while(|line| line.trim().is_empty())
        .find(|line| !line.trim().is_empty())
        .unwrap_or("")
        .trim()
        .to_string()
}

/// 路由用短描述：优先「何时使用 / Use when」首条要点（对齐 Agent Skills description）。
fn skill_routing_description(body: &str) -> String {
    let mut in_when = false;
    for line in body.lines() {
        let t = line.trim();
        if t.starts_with("## 何时使用") || t.starts_with("## Use when") || t.starts_with("## When to use")
        {
            in_when = true;
            continue;
        }
        if in_when {
            if t.starts_with("## ") {
                break;
            }
            if let Some(rest) = t.strip_prefix("- **用**：").or_else(|| t.strip_prefix("- **Use**:")) {
                return rest.trim().to_string();
            }
            if let Some(rest) = t.strip_prefix("- ") {
                if !rest.is_empty() {
                    return rest.to_string();
                }
            }
        }
    }
    let fallback = skill_description(body);
    if fallback.starts_with('#') {
        fallback.trim_start_matches('#').trim().to_string()
    } else {
        fallback
    }
}

/// 启用中的 Skill L0 目录（仅元数据：name + description；正文不进目录）。
pub fn skills_catalog_l0() -> Vec<SkillCatalogEntry> {
    list_skills()
        .into_iter()
        .filter(|s| s.enabled)
        .map(|s| {
            let description = {
                let d = s.description.trim();
                if d.chars().count() > 160 {
                    format!("{}…", d.chars().take(159).collect::<String>())
                } else {
                    d.to_string()
                }
            };
            SkillCatalogEntry {
                id: s.id,
                name: s.name,
                description,
            }
        })
        .collect()
}

/// 渲染 L0 目录块（注入模型上下文；与系统提示词模块分离组装）。
pub fn render_skills_catalog_l0() -> String {
    let entries = skills_catalog_l0();
    if entries.is_empty() {
        return String::new();
    }
    let mut out = String::from(
        "Available skills (id — when to use). Full skill bodies load only when selected for this turn:\n",
    );
    for e in &entries {
        out.push_str(&format!("- {} ({}): {}\n", e.name, e.id, e.description));
    }
    out
}

/// 单个 Skill 注入正文的字符上限（多工具合并后需容纳完整页面能力清单）。
pub const SKILL_BODY_CHAR_CAP: usize = 2200;

/// 截断 Skill 正文：优先在段落/句子边界断开。
pub fn truncate_skill_body(body: &str) -> String {
    if body.chars().count() <= SKILL_BODY_CHAR_CAP {
        return body.to_string();
    }
    let truncated: String = body.chars().take(SKILL_BODY_CHAR_CAP).collect();
    match truncated.rfind(['\n', '。']) {
        Some(pos) if pos > SKILL_BODY_CHAR_CAP / 2 => truncated[..=pos].to_string(),
        _ => truncated,
    }
}

/// 渲染 L1 正文块（仅命中项；正文按预算截断）。
pub fn render_skill_bodies_l1(docs: &[SkillDoc]) -> String {
    if docs.is_empty() {
        return String::new();
    }
    let mut out = String::from("Loaded skill instructions for this turn:\n");
    for sk in docs {
        let body = truncate_skill_body(&sk.body);
        let agent_tools: Vec<&str> = sk
            .tool_ids
            .iter()
            .filter(|id| crate::agent::registry::lookup(id).is_some())
            .map(|s| s.as_str())
            .collect();
        if !agent_tools.is_empty() {
            out.push_str(&format!(
                "### {}（Agent 工具：{}）\n按正文「何时使用」选择其中一个工具调用；不要改用不相关工具。\n{}\n\n",
                sk.skill_id,
                agent_tools.join(", "),
                body
            ));
        } else {
            out.push_str(&format!(
                "### {}\n此为工具箱能力说明，**不要**编造未注册的 `<tool_call>`；引导用户打开对应页面。\n{}\n\n",
                sk.skill_id, body
            ));
        }
    }
    out
}

pub fn set_skill_enabled(skill_id: &str, enabled: bool) -> Result<SkillInfo, String> {
    let skill_id = skill_id.trim();
    let embedded = SKILLS.iter().find(|skill| skill.id == skill_id);
    if embedded.is_none() {
        let disabled = load_disabled_skills();
        let user_skill = read_user_skills(&disabled)?
            .into_iter()
            .find(|skill| skill.id == skill_id)
            .ok_or_else(|| "Skill 不存在".to_string())?;
        let mut disabled = load_disabled_skills();
        if enabled {
            disabled.remove(skill_id);
        } else {
            disabled.insert(skill_id.to_string());
        }
        save_disabled_skills(disabled)?;
        return Ok(user_skill);
    }
    let embedded = embedded.expect("embedded checked");

    let mut disabled = load_disabled_skills();
    if enabled {
        disabled.remove(skill_id);
    } else {
        disabled.insert(skill_id.to_string());
    }
    save_disabled_skills(disabled)?;

    let parsed = parse_skill(&resolve_builtin_raw(embedded.id, embedded.source))
        .ok_or_else(|| "Skill 文档无效".to_string())?;
    let modified = builtin_has_override(embedded.id);
    let name = parsed
        .name
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| skill_title(&parsed.body, embedded.id));
    let description = resolve_l0_description(&parsed);
    Ok(SkillInfo {
        id: embedded.id.to_string(),
        name,
        tool_ids: parsed.tool_ids,
        toolbox_ids: parsed.toolbox_ids,
        keywords: Some(parsed.keywords),
        description,
        source: "builtin".to_string(),
        body: parsed.body,
        enabled,
        editable: true,
        modified,
        can_restore_default: modified,
    })
}

pub fn create_skill(
    name: String,
    description: String,
    keywords: Vec<String>,
    tool_ids: Vec<String>,
    body: String,
) -> Result<SkillInfo, String> {
    write_user_skill(
        None,
        UserSkillInput {
            name,
            description,
            keywords,
            tool_ids,
            body,
        },
        false,
    )
}

pub fn update_skill(
    id: String,
    name: String,
    description: String,
    keywords: Vec<String>,
    tool_ids: Vec<String>,
    body: String,
) -> Result<SkillInfo, String> {
    let disabled = load_disabled_skills();
    if let Some(embedded) = SKILLS.iter().find(|s| s.id == id) {
        let prev = resolve_builtin_raw(embedded.id, embedded.source);
        append_version_snapshot(&id, &prev)?;
        let parsed_prev = parse_skill(&prev);
        let avoid = parsed_prev
            .as_ref()
            .map(|p| p.avoid_keywords.clone())
            .unwrap_or_default();
        let toolbox = parsed_prev
            .as_ref()
            .map(|p| p.toolbox_ids.clone())
            .unwrap_or_default();
        let ids = if tool_ids.is_empty() {
            parsed_prev
                .map(|p| p.tool_ids)
                .unwrap_or_else(|| {
                    if toolbox.is_empty() {
                        vec![id.clone()]
                    } else {
                        Vec::new()
                    }
                })
        } else {
            tool_ids.clone()
        };
        if ids.iter().any(|tid| crate::agent::registry::lookup(tid).is_none()) {
            return Err("关联工具不存在".to_string());
        }
        let mut body_out = body;
        // 若用户未带标题，用 name 作 H1；L0 description 写入 frontmatter（正文按需，不塞进目录）。
        if !body_out.trim_start().starts_with('#') && !name.trim().is_empty() {
            body_out = format!("# {}\n\n{}", name.trim(), body_out.trim());
        }
        let desc = description.trim();
        let raw = serialize_builtin_override(
            Some(name.trim()).filter(|s| !s.is_empty()),
            Some(desc).filter(|s| !s.is_empty()),
            &ids,
            &toolbox,
            &keywords,
            &avoid,
            &body_out,
        );
        std::fs::create_dir_all(override_skills_dir())
            .map_err(|e| format!("创建覆盖目录失败: {e}"))?;
        std::fs::write(override_path(&id), raw).map_err(|e| format!("保存覆盖失败: {e}"))?;
        return builtin_skill_info(&id, embedded.source, !disabled.contains(&id))
            .ok_or_else(|| "Skill 文档无效".to_string());
    }

    let existing = read_user_skills(&disabled)?
        .into_iter()
        .find(|skill| skill.id == id)
        .ok_or_else(|| "Skill 不存在".to_string())?;
    write_user_skill(
        Some(&existing.id),
        UserSkillInput {
            name,
            description,
            keywords,
            tool_ids,
            body,
        },
        false,
    )
}

pub fn list_skill_versions(id: &str) -> Result<Vec<SkillVersionInfo>, String> {
    let dir = versions_dir(id);
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name();
        let s = name.to_string_lossy();
        let Some(seq_str) = s.strip_suffix(".md") else {
            continue;
        };
        let Ok(seq) = seq_str.parse::<u64>() else {
            continue;
        };
        let raw = std::fs::read_to_string(entry.path()).map_err(|e| e.to_string())?;
        let saved_at = raw
            .lines()
            .next()
            .and_then(|l| {
                l.strip_prefix("<!-- saved_at: ")
                    .and_then(|r| r.strip_suffix(" -->"))
            })
            .unwrap_or("")
            .to_string();
        out.push(SkillVersionInfo {
            seq,
            saved_at,
            preview: version_preview(&raw),
        });
    }
    out.sort_by(|a, b| b.seq.cmp(&a.seq));
    Ok(out)
}

pub fn restore_skill_version(id: &str, seq: u64) -> Result<SkillInfo, String> {
    let path = versions_dir(id).join(format!("{seq}.md"));
    let raw = std::fs::read_to_string(&path).map_err(|e| format!("读取版本失败: {e}"))?;
    let content = raw
        .lines()
        .skip_while(|l| l.starts_with("<!--"))
        .collect::<Vec<_>>()
        .join("\n")
        .trim_start()
        .to_string();

    if let Some(embedded) = SKILLS.iter().find(|s| s.id == id) {
        let prev = resolve_builtin_raw(embedded.id, embedded.source);
        append_version_snapshot(id, &prev)?;
        std::fs::create_dir_all(override_skills_dir()).map_err(|e| e.to_string())?;
        std::fs::write(override_path(id), &content).map_err(|e| e.to_string())?;
        let disabled = load_disabled_skills();
        return builtin_skill_info(id, embedded.source, !disabled.contains(id))
            .ok_or_else(|| "恢复后文档无效".to_string());
    }

    let disabled = load_disabled_skills();
    let _ = read_user_skills(&disabled)?
        .into_iter()
        .find(|s| s.id == id)
        .ok_or_else(|| "Skill 不存在".to_string())?;
    let input = parse_user_markdown(id, &content)?;
    write_user_skill(Some(id), input, false)
}

pub fn restore_skill_default(id: &str) -> Result<SkillInfo, String> {
    let embedded = SKILLS
        .iter()
        .find(|s| s.id == id)
        .ok_or_else(|| "仅内置 Skill 可恢复默认".to_string())?;
    if builtin_has_override(id) {
        let prev = resolve_builtin_raw(id, embedded.source);
        append_version_snapshot(id, &prev)?;
        let _ = std::fs::remove_file(override_path(id));
    }
    let disabled = load_disabled_skills();
    builtin_skill_info(id, embedded.source, !disabled.contains(id))
        .ok_or_else(|| "Skill 文档无效".to_string())
}

pub fn delete_skill(id: String) -> Result<(), String> {
    if SKILLS.iter().any(|skill| skill.id == id) {
        return Err("内置 Skill 不可删除".to_string());
    }
    let path = custom_skills_dir().join(format!("{id}.md"));
    if path.exists() {
        std::fs::remove_file(path).map_err(|e| format!("删除 Skill 失败: {e}"))?;
    }
    let mut disabled = load_disabled_skills();
    disabled.remove(&id);
    save_disabled_skills(disabled)
}

pub fn import_skill(raw: String, name: Option<String>) -> Result<SkillInfo, String> {
    let override_name = name.clone();
    let provisional_id = sanitize_id(name.as_deref().unwrap_or("imported-skill"));
    let mut input = match parse_user_markdown(&provisional_id, &raw) {
        Ok(input) => input,
        Err(_) => {
            let body = raw.trim().to_string();
            if body.is_empty() {
                return Err("Markdown 文件内容为空".to_string());
            }
            UserSkillInput {
                name: override_name.unwrap_or_else(|| skill_title(&body, "导入 Skill")),
                description: skill_description(&format!("\n{body}")),
                keywords: Vec::new(),
                tool_ids: Vec::new(),
                body,
            }
        }
    };
    if let Some(name) = name {
        input.name = name;
    }
    write_user_skill(None, input, true)
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

/// 判断一行是否更像粘贴的 JSON/DSL 正文，而非用户意图句。
fn looks_like_payload_line(line: &str) -> bool {
    let t = line.trim();
    if t.is_empty() {
        return false;
    }
    if t.starts_with('{')
        || t.starts_with('[')
        || t.starts_with("{\\")
        || t.starts_with("[\\")
        || t.starts_with("\\\"")
    {
        return true;
    }
    let colon = t.matches(':').count();
    let quotes = t.matches('"').count() + t.matches("\\\"").count();
    t.len() > 80 && colon >= 2 && quotes >= 4
}

const INTENT_FOCUS_SOFT_CAP: usize = 320;

/// 超长粘贴时抽出**全文中的自然语言意图句**（前/中/后皆可）作为检索焦点，
/// 丢弃 JSON/DSL 正文，避免字段名污染打分。
pub fn intent_focus_for_retrieval(query: &str) -> String {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.chars().count() <= 240 {
        return trimmed.to_string();
    }

    let intent_lines: Vec<&str> = trimmed
        .lines()
        .map(str::trim)
        .filter(|t| !t.is_empty() && !looks_like_payload_line(t))
        .collect();

    if !intent_lines.is_empty() {
        let joined = intent_lines.join("\n");
        if joined.chars().count() <= INTENT_FOCUS_SOFT_CAP {
            return joined;
        }
        // 意图句过多时保留头部与尾部，兼顾「意图在前」与「意图在后」
        return clip_head_and_tail(&joined, INTENT_FOCUS_SOFT_CAP);
    }

    // 无换行或整段都像 payload：取头尾各一段，跳过中间大块
    clip_head_and_tail(trimmed, 240)
}

fn clip_head_and_tail(text: &str, budget: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= budget {
        return text.to_string();
    }
    let head_n = budget / 2;
    let tail_n = budget - head_n;
    let head: String = chars[..head_n].iter().collect();
    let tail: String = chars[chars.len() - tail_n..].iter().collect();
    format!("{head}\n…\n{tail}")
}

fn score_skill(query: &str, parsed: &ParsedSkill) -> i32 {
    let focus = intent_focus_for_retrieval(query);
    let mut score = score_skill_against(&focus, parsed);

    // 全文仅作弱补充：意图焦点已足够时不再让 payload 泛词翻盘。
    if focus != query && query.len() > focus.len().saturating_add(80) {
        let weak = score_skill_against(query, parsed);
        if weak > score {
            score += ((weak - score) / 4).max(0);
        }
    }

    score
}

fn score_skill_against(query: &str, parsed: &ParsedSkill) -> i32 {
    let mut score = 0;

    for kw in &parsed.keywords {
        if contains_match(query, kw) {
            // 更长的路由词权重更高，避免泛词（如 json）压过具体意图。
            let boost = if kw.chars().count() >= 3 { 3 } else { 1 };
            score += boost;
        }
    }

    for tool_id in &parsed.tool_ids {
        if contains_match(query, tool_id) {
            score += 3;
        }
        for segment in tool_id.split('.') {
            // 跳过过短/过泛的段（如 json），减少近邻误排。
            if segment.len() >= 3 && segment != "json" && contains_match(query, segment) {
                score += 2;
            }
        }
    }

    // 命中「何时不用」关键词时强力降权，避免近邻工具抢首位。
    for kw in &parsed.avoid_keywords {
        if contains_match(query, kw) {
            score -= 8;
        }
    }

    score
}

/// 按 query 关键词/工具名/中文别名匹配预置 Skill，最多返回 `limit` 条。
pub fn retrieve_skills(query: &str, limit: usize) -> Vec<SkillDoc> {
    if limit == 0 || query.trim().is_empty() {
        return Vec::new();
    }

    let disabled = load_disabled_skills();
    let mut candidates: Vec<(String, ParsedSkill)> = SKILLS
        .iter()
        .filter(|embedded| !disabled.contains(embedded.id))
        .filter_map(|embedded| {
            let raw = resolve_builtin_raw(embedded.id, embedded.source);
            let parsed = parse_skill(&raw)?;
            Some((embedded.id.to_string(), parsed))
        })
        .collect();
    candidates.extend(
        read_user_skills(&disabled)
            .unwrap_or_default()
            .into_iter()
            .filter(|skill| skill.enabled)
            .filter_map(|skill| {
                Some((
                    skill.id,
                    ParsedSkill {
                        name: Some(skill.name),
                        description: Some(skill.description),
                        tool_ids: skill.tool_ids,
                        toolbox_ids: skill.toolbox_ids,
                        keywords: Vec::new(),
                        avoid_keywords: Vec::new(),
                        body: skill.body,
                    },
                ))
            }),
    );

    let mut ranked: Vec<(i32, SkillDoc)> = candidates
        .into_iter()
        .filter_map(|(id, parsed)| {
            let score = score_skill(query, &parsed);
            if score <= 0 {
                return None;
            }
            Some((
                score,
                SkillDoc {
                    skill_id: id,
                    tool_ids: parsed.tool_ids,
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
    use crate::agent::registry::{all_tools, lookup};

    #[test]
    fn jwt_query_hits_jwt_skill() {
        let hits = retrieve_skills("帮我解析这段 JWT", 3);
        assert!(hits.iter().any(|s| {
            s.skill_id == "crypto" || s.tool_ids.iter().any(|t| t.contains("jwt"))
        }));
        assert!(hits.len() <= 3);
    }

    #[test]
    fn flatten_query_ranks_json_skill_first() {
        let hits = retrieve_skills("把嵌套 JSON 平铺开", 3);
        assert!(!hits.is_empty());
        assert_eq!(hits[0].skill_id, "json");
        assert!(hits[0].tool_ids.iter().any(|t| t == "json.flatten"));
    }

    #[test]
    fn query_string_ranks_json_skill_first() {
        let hits = retrieve_skills("把 JSON 转成 query string", 3);
        assert!(!hits.is_empty());
        assert_eq!(hits[0].skill_id, "json");
        assert!(hits[0].tool_ids.iter().any(|t| t == "json.to_query"));
    }

    #[test]
    fn escape_intent_with_es_payload_ranks_format_first() {
        let mut q = String::from(
            r#"{\"query\":{\"function_score\":{\"boost_mode\":\"replace\",\"functions\":[{\"filter\":{\"term\":{\"keywd\":\"朝阳\"}},\"weight\":3}],\"query\":{\"bool\":{\"must\":{\"regexp\":{\"keywd.keyword\":{\"value\":\"朝\"}}}}}}},\"size\":10}"#,
        );
        while q.chars().count() < 300 {
            q.push_str(r#"{\"extra\":\"padding\"}"#);
        }
        q.push_str("\n\n把这个转义一下");
        let focus = intent_focus_for_retrieval(&q);
        assert!(focus.contains("转义"), "focus={focus}");
        let hits = retrieve_skills(&q, 3);
        let ids: Vec<_> = hits.iter().map(|h| h.skill_id.as_str()).collect();
        assert_eq!(ids.first().copied(), Some("json"), "hits={ids:?}");
        assert!(
            !ids.iter().any(|id| *id == "encoding"),
            "encoding must not rank first for JSON 转义: hits={ids:?}"
        );
    }

    #[test]
    fn escape_short_query_ranks_json() {
        let hits = retrieve_skills("转义下", 5);
        let ids: Vec<_> = hits.iter().map(|h| h.skill_id.as_str()).collect();
        assert_eq!(ids.first().copied(), Some("json"), "hits={ids:?}");
    }

    #[test]
    fn json_skill_body_keeps_tool_id_under_cap() {
        let hits = retrieve_skills("转义下这段 JSON", 1);
        assert_eq!(hits[0].skill_id, "json");
        let rendered = render_skill_bodies_l1(&hits);
        assert!(
            rendered.contains("json.format")
                && rendered.contains("按正文「何时使用」选择其中一个工具"),
            "rendered={rendered}"
        );
        let body = truncate_skill_body(&hits[0].body);
        assert!(body.contains("json.format"), "truncated body missing tool: {body}");
    }

    #[test]
    fn leading_intent_with_payload_ranks_format_first() {
        let mut payload = String::from(
            r#"{\"query\":{\"function_score\":{\"boost_mode\":\"replace\",\"query\":{\"bool\":{\"must\":{\"term\":{\"keywd\":\"x\"}}}}}}}"#,
        );
        while payload.chars().count() < 300 {
            payload.push_str(r#"{\"extra\":\"padding\"}"#);
        }
        let q = format!("请帮我把下面这段 JSON 转义/格式化一下：\n\n{payload}");
        let focus = intent_focus_for_retrieval(&q);
        assert!(focus.contains("转义") || focus.contains("格式化"), "focus={focus}");
        assert!(!focus.contains("function_score"), "payload leaked into focus={focus}");
        let hits = retrieve_skills(&q, 3);
        let ids: Vec<_> = hits.iter().map(|h| h.skill_id.as_str()).collect();
        assert_eq!(ids.first().copied(), Some("json"), "hits={ids:?}");
    }

    #[test]
    fn loading_skills_does_not_register_tools() {
        let before = lookup("http.request");
        let _ = retrieve_skills("http", 3);
        assert!(before.is_none());
        assert!(lookup("http.request").is_none());
    }

    #[test]
    fn disabled_skill_is_filtered_from_retrieval() {
        TEST_PREFS.lock().unwrap().replace(Vec::new());
        let query = "帮我解析这段 JWT";
        let before = retrieve_skills(query, 3);
        assert!(before.iter().any(|skill| skill.skill_id == "crypto"));

        set_skill_enabled("crypto", false).expect("disable skill");
        let after = retrieve_skills(query, 3);
        assert!(!after.iter().any(|skill| skill.skill_id == "crypto"));

        set_skill_enabled("crypto", true).expect("restore skill");
        TEST_PREFS.lock().unwrap().take();
    }

    #[test]
    fn all_registry_tools_have_builtin_skill_coverage() {
        let covered: std::collections::HashSet<String> = list_skills()
            .into_iter()
            .filter(|skill| skill.source == "builtin")
            .flat_map(|skill| skill.tool_ids)
            .collect();
        let missing: Vec<&str> = all_tools()
            .iter()
            .map(|tool| tool.id)
            .filter(|id| !covered.contains(*id))
            .collect();
        assert!(missing.is_empty(), "missing skill coverage: {missing:?}");
    }

    #[test]
    fn all_toolbox_tools_have_builtin_skill_coverage() {
        let covered: std::collections::HashSet<u32> = list_skills()
            .into_iter()
            .filter(|skill| skill.source == "builtin")
            .flat_map(|skill| skill.toolbox_ids)
            .collect();
        let missing: Vec<u32> = (1u32..=36).filter(|id| !covered.contains(id)).collect();
        assert!(
            missing.is_empty(),
            "missing toolbox skill coverage for ids: {missing:?}; covered={covered:?}"
        );
        assert_eq!(
            list_skills().iter().filter(|s| s.source == "builtin").count(),
            SKILLS.len()
        );
        // 渐进披露：合并后 L0 目录应远少于工具数
        assert!(
            SKILLS.len() <= 16,
            "too many L0 skills (want consolidated): {}",
            SKILLS.len()
        );
    }

    #[test]
    fn l0_catalog_uses_precise_frontmatter_description() {
        let catalog = skills_catalog_l0();
        assert!(!catalog.is_empty());
        for e in &catalog {
            assert!(
                !e.description.trim().is_empty(),
                "empty L0 description for {}",
                e.id
            );
            assert!(
                !e.description.starts_with('#'),
                "L0 must not dump body heading: {}",
                e.id
            );
            assert!(
                !e.description.contains("<tool_call>"),
                "L0 must not include L1 samples: {}",
                e.id
            );
            assert!(
                e.description.chars().count() <= 160,
                "L0 description too long for {}: {}",
                e.id,
                e.description.chars().count()
            );
        }
        let json = catalog.iter().find(|e| e.id == "json").expect("json skill");
        assert!(
            json.description.contains("JSON") || json.description.contains("json"),
            "json L0 should mention JSON: {}",
            json.description
        );
        // frontmatter description 优先于正文推导
        assert!(
            json.description.contains("实体类") || json.description.contains("query"),
            "expected frontmatter description, got: {}",
            json.description
        );
    }

    #[test]
    fn builtin_override_and_restore_default() {
        let id = "datetime-id";
        let dir = crate::agent::llama_log::toolbox_dir().join("skills/overrides");
        let _ = std::fs::remove_file(dir.join(format!("{id}.md")));
        let before = list_skills()
            .into_iter()
            .find(|s| s.id == id)
            .expect("builtin");
        assert!(!before.modified);

        update_skill(
            id.to_string(),
            "时间标识测试".into(),
            "测试描述".into(),
            vec!["时间戳".into()],
            vec!["timestamp.convert".into()],
            "# 时间标识测试\n\n## 何时使用 / 何时不用\n\n- **用**：测试\n".into(),
        )
        .expect("update builtin");
        let mid = list_skills()
            .into_iter()
            .find(|s| s.id == id)
            .expect("builtin");
        assert!(mid.modified);
        assert!(mid.can_restore_default);
        assert!(mid.body.contains("时间标识测试"));

        let versions = list_skill_versions(id).expect("versions");
        assert!(!versions.is_empty());

        restore_skill_default(id).expect("restore");
        let after = list_skills()
            .into_iter()
            .find(|s| s.id == id)
            .expect("builtin");
        assert!(!after.modified);
        assert!(after.body.contains("时间标识"));
    }
}
