use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use jieba_rs::Jieba;
use pinyin::{self, Args};

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub category_id: Option<u32>,
    pub category_name: Option<String>,
    pub tags: Vec<String>,
    pub gradient: String,
}

/// Story 5.1 (Phase 1.5 v0) lightweight local intent routing output.
/// `score` is non-normalised — higher is better — and `reason` is a short
/// Chinese string used by the UI to explain which signals fired
/// (e.g. "名称匹配" / "意图: 描述关键词 x2").
#[derive(Debug, Clone, Serialize)]
pub struct AiRouteResult {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub category_id: Option<u32>,
    pub category_name: Option<String>,
    pub tags: Vec<String>,
    pub gradient: String,
    pub score: f64,
    pub reason: String,
}
/// 全局分词器
static JIEBA: Lazy<Jieba> = Lazy::new(|| Jieba::new());

/// 全局搜索索引
static SEARCH_INDEX: Lazy<Mutex<SearchIndex>> = Lazy::new(|| Mutex::new(SearchIndex::new()));

/// 搜索索引结构
pub struct SearchIndex {
    /// 词语到工具ID的映射
    word_to_tool_ids: HashMap<String, HashSet<u32>>,
    /// 工具ID到工具信息的映射
    tool_id_to_info: HashMap<u32, SearchResult>,
}

impl SearchIndex {
    pub fn new() -> Self {
        SearchIndex {
            word_to_tool_ids: HashMap::new(),
            tool_id_to_info: HashMap::new(),
        }
    }

    /// 清空索引
    pub fn clear(&mut self) {
        self.word_to_tool_ids.clear();
        self.tool_id_to_info.clear();
    }

    /// 获取字符串的拼音（无声调）
    fn get_pinyin(s: &str) -> String {
        let args = Args::new();
        pinyin::lazy_pinyin(s, &args).join("")
    }

    /// 获取拼音首字母
    fn get_pinyin_initials(s: &str) -> String {
        let args = Args::new();
        pinyin::lazy_pinyin(s, &args)
            .iter()
            .filter_map(|p| p.chars().next())
            .collect()
    }

    /// 添加工具到索引
    pub fn add_tool(&mut self, tool: SearchResult) {
        let tool_id = tool.id;

        // 存储工具信息
        self.tool_id_to_info.insert(tool_id, tool.clone());

        // 提取关键词并索引
        self.index_text(&tool.name, tool_id);
        self.index_text(&tool.description, tool_id);
        for tag in &tool.tags {
            self.index_text(tag, tool_id);
        }
    }

    /// 对文本进行分词并索引
    fn index_text(&mut self, text: &str, tool_id: u32) {
        // 使用jieba分词
        let words = JIEBA.cut(text, false);

        for word in words {
            if word.is_empty() || word.len() < 2 {
                continue;
            }

            let word_lower = word.to_lowercase();

            // 原文
            self.word_to_tool_ids
                .entry(word_lower.clone())
                .or_insert_with(HashSet::new)
                .insert(tool_id);

            // 全拼（仅对中文词）
            if Self::is_chinese(word) {
                let pinyin = Self::get_pinyin(word);
                if !pinyin.is_empty() && pinyin != word_lower {
                    self.word_to_tool_ids
                        .entry(pinyin)
                        .or_insert_with(HashSet::new)
                        .insert(tool_id);
                }

                // 首字母
                let initials = Self::get_pinyin_initials(word);
                if !initials.is_empty() && initials != word_lower {
                    self.word_to_tool_ids
                        .entry(initials)
                        .or_insert_with(HashSet::new)
                        .insert(tool_id);
                }
            }
        }

        // 对英文部分进行额外处理（如 "JSON" -> "json"）
        for part in text.split(|c: char| !c.is_alphanumeric()) {
            if part.len() >= 2 {
                let part_lower = part.to_lowercase();
                self.word_to_tool_ids
                    .entry(part_lower)
                    .or_insert_with(HashSet::new)
                    .insert(tool_id);
            }
        }
    }

    /// 判断是否包含中文字符
    fn is_chinese(s: &str) -> bool {
        s.chars().any(|c| '\u{4e00}' <= c && c <= '\u{9fff}')
    }

    /// 搜索工具
    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        if query.trim().is_empty() {
            return self.tool_id_to_info.values().cloned().collect();
        }

        let query_lower = query.to_lowercase();
        let query_pinyin = Self::get_pinyin(query);
        let query_initials = Self::get_pinyin_initials(query);

        // 收集匹配的ID
        let mut matched_ids: HashSet<u32> = HashSet::new();

        // 原文精确匹配
        if let Some(ids) = self.word_to_tool_ids.get(&query_lower) {
            matched_ids.extend(ids);
        }

        // 模糊匹配：查询词是某个词的子串
        for (word, ids) in &self.word_to_tool_ids {
            if word.len() > query_lower.len() && word.contains(&query_lower) {
                matched_ids.extend(ids);
            }
        }

        // 全拼匹配
        if !query_pinyin.is_empty() {
            if let Some(ids) = self.word_to_tool_ids.get(&query_pinyin) {
                matched_ids.extend(ids);
            }
            // 模糊拼音匹配
            for (word, ids) in &self.word_to_tool_ids {
                if word.len() > query_pinyin.len() && word.contains(&query_pinyin) {
                    matched_ids.extend(ids);
                }
            }
        }

        // 首字母匹配
        if !query_initials.is_empty() {
            if let Some(ids) = self.word_to_tool_ids.get(&query_initials) {
                matched_ids.extend(ids);
            }
            // 模糊首字母匹配（前缀匹配）
            for (word, ids) in &self.word_to_tool_ids {
                if word.len() >= query_initials.len() && word.starts_with(&query_initials) {
                    matched_ids.extend(ids);
                }
            }
        }

        // 获取匹配的工具
        let mut results: Vec<SearchResult> = matched_ids
            .into_iter()
            .filter_map(|id| self.tool_id_to_info.get(&id).cloned())
            .collect();

        // 相关性排序
        results.sort_by(|a, b| {
            // 计算相关性分数
            let score_a = Self::calculate_relevance(a, &query_lower, &query_pinyin, &query_initials);
            let score_b = Self::calculate_relevance(b, &query_lower, &query_pinyin, &query_initials);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    /// 计算相关性分数
    fn calculate_relevance(tool: &SearchResult, query: &str, query_pinyin: &str, query_initials: &str) -> f64 {
        let mut score = 0.0;

        // 名称完全包含查询词
        if tool.name.to_lowercase().contains(query) {
            score += 100.0;
        }
        // 名称拼音包含查询词
        if Self::get_pinyin(&tool.name.to_lowercase()).contains(query_pinyin) {
            score += 80.0;
        }
        // 名称首字母前缀匹配
        if Self::get_pinyin_initials(&tool.name).starts_with(query_initials) {
            score += 60.0;
        }
        // 名称首字母包含匹配
        if Self::get_pinyin_initials(&tool.name).contains(query_initials) {
            score += 40.0;
        }

        // 描述匹配
        if tool.description.to_lowercase().contains(query) {
            score += 30.0;
        }

        // 标签匹配
        for tag in &tool.tags {
            if tag.to_lowercase().contains(query) {
                score += 50.0;
            }
        }

        score
    }
}

impl Default for SearchIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// 初始化搜索索引
pub fn build_search_index(tools: Vec<SearchResult>) {
    let mut index = SEARCH_INDEX.lock().unwrap();
    index.clear();
    for tool in tools {
        index.add_tool(tool);
    }
}

/// 搜索工具
#[tauri::command]
pub fn search_tools(query: String) -> Vec<SearchResult> {
    let index = SEARCH_INDEX.lock().unwrap();
    index.search(&query)
}

// ---------------------------------------------------------------------------
// Story 5.1 (Phase 1.5 v0): lightweight local intent routing.
//
// This is a deliberately narrow slice of "AI search": instead of shipping an
// LLM (which would require model download, 300MB+ binary, and bypasses the
// project constraint of zero-network) we score candidate tools using the
// jieba tokeniser, pinyin/initials, and a per-tool tag/description match
// across the whole query.
//
// The output `AiRouteResult` includes a `reason` string so the Spotlight UI
// can show users why a tool was promoted. This keeps the same UX contract
// we'd want from an LLM-backed intent router (transparency) without the
// binary-size cost.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub(crate) struct ScoredTool {
    info: SearchResult,
    score: f64,
    reason: String,
}

/// Slice the query through jieba and drop empty / single-char / pure-punct
/// segments. Used as the unit of "meaningful word" for intent scoring.
fn meaningful_query_tokens(query: &str) -> Vec<String> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    JIEBA
        .cut(trimmed, false)
        .into_iter()
        .map(|w| w.trim().to_lowercase())
        .filter(|w| !w.is_empty() && w.chars().any(|c| !c.is_whitespace()))
        .collect()
}

impl SearchIndex {
    /// Story 5.1: rank the tools inside `tool_ids` against the user's query.
    /// Returns scored candidates sorted by descending `score`. Tools missing
    /// from the index are silently skipped — callers should already have
    /// presented an authoritative tool list to the user.
    pub(crate) fn rank_intent(&self, query: &str, tool_ids: &[u32]) -> Vec<ScoredTool> {
        let tokens = meaningful_query_tokens(query);
        let query_lower = query.to_lowercase();
        let query_pinyin = Self::get_pinyin(query);
        let query_initials = Self::get_pinyin_initials(query);

        let mut scored = Vec::with_capacity(tool_ids.len());
        for id in tool_ids {
            let info = match self.tool_id_to_info.get(id) {
                Some(info) => info,
                None => continue,
            };

            let mut score = 0.0;
            let mut reasons: Vec<String> = Vec::new();

            // Whole-query name / pinyin match keeps it compatible with the
            // existing `search_tools` ranking so users see deterministic
            // ordering when they type exact tool names.
            if info.name.to_lowercase().contains(&query_lower) {
                score += 100.0;
                reasons.push("名称匹配".to_string());
            }
            if Self::get_pinyin(&info.name.to_lowercase()).contains(&query_pinyin) {
                score += 80.0;
                if !reasons.iter().any(|r| r == "拼音匹配") {
                    reasons.push("拼音匹配".to_string());
                }
            }
            let initials = Self::get_pinyin_initials(&info.name);
            if initials.starts_with(&query_initials) {
                score += 60.0;
                if !reasons.iter().any(|r| r == "首字母匹配") {
                    reasons.push("首字母匹配".to_string());
                }
            }

            // Intent: each jieba segment contributes based on where it lands
            // (tag > description > name suffix). Multi-token queries add
            // multiplicative matching so a query like "图片 压缩" picks
            // "图片压缩" instead of any single-word hit.
            if !tokens.is_empty() {
                let mut token_hits = 0usize;
                for tok in &tokens {
                    let tag_hit = info
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(tok));
                    let desc_hit = info.description.to_lowercase().contains(tok);
                    let name_hit = info.name.to_lowercase().contains(tok);

                    if tag_hit {
                        score += 25.0;
                        token_hits += 1;
                    } else if desc_hit {
                        score += 12.0;
                        token_hits += 1;
                    } else if name_hit {
                        score += 18.0;
                        token_hits += 1;
                    }
                }

                if tokens.len() > 1 && token_hits == tokens.len() {
                    // Multi-token full match — the strongest intent signal.
                    score += 40.0;
                    reasons.push(format!("意图匹配 {} 项", tokens.len()));
                } else if token_hits > 0 {
                    reasons.push(format!("意图部分匹配 {} 项", token_hits));
                }
            }

            // If nothing fired we still want to surface the candidate in
            // deterministic order so the UI doesn't show gaps; the small
            // base keeps `displayedResults` ordering stable.
            if reasons.is_empty() {
                score += 0.001 * (1.0 / (*id as f64 + 1.0));
            }

            scored.push(ScoredTool {
                info: info.clone(),
                score,
                reason: reasons.join(" · "),
            });
        }

        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.info.id.cmp(&b.info.id))
        });
        scored
    }
}

/// Local intent router used by Spotlight when `aiMode` is on.
///
/// Inputs:
///   - `query`: the user's natural-language query
///   - `tool_ids`: tools the frontend wants considered (typically the current
///     `search_tools` result ids). Empty means "rank the whole index".
///   - `role_ids`: tools mapped to any of these roles are weighted higher
///     (Story 2.4 role-aware boost, applied at the local AI layer).
#[tauri::command]
pub fn ai_route_intent(
    query: String,
    tool_ids: Vec<u32>,
) -> Vec<AiRouteResult> {
    let index = SEARCH_INDEX.lock().unwrap();

    // If the frontend didn't pre-filter, consider every tool in the index.
    let candidate_ids: Vec<u32> = if tool_ids.is_empty() {
        index.tool_id_to_info.keys().copied().collect()
    } else {
        tool_ids
    };

    let scored = index.rank_intent(&query, &candidate_ids);

    // Story 2.4 role boost is applied client-side inside
    // `displayedResults`. By design this command stays role-agnostic —
    // it scores purely on jieba/pinyin/tag/description intent signals
    // derived from `query` + `tool_ids`. Keeping it pure makes it easy
    // to unit-test and avoids loading the tool<->role mapping here.

    scored
        .into_iter()
        .map(|s| AiRouteResult {
            id: s.info.id,
            name: s.info.name,
            description: s.info.description,
            icon: s.info.icon,
            category_id: s.info.category_id,
            category_name: s.info.category_name,
            tags: s.info.tags,
            gradient: s.info.gradient,
            score: s.score,
            reason: s.reason,
        })
        .collect()
}
