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
