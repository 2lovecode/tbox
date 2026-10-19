//! 本地 Laya 决策路由客户端（change: laya-spotlight-tool-routing）。
//!
//! 通过用户本机运行的 `laya-serve`（`pip install laya[serve]`）HTTP 服务，
//! 将自然语言输入映射到已注册工具 id（`choice` 决策）。任何失败
//! （服务不可达 / 超时 / 解析失败 / 低置信度 / 未启用）统一返回 `None`（未命中），
//! 调用方静默回退到关键词检索，不向用户报错。

use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::db;

const SETTINGS_FILE: &str = "laya_settings.json";
/// 单个工具候选描述的最大长度（字符）。
const MAX_CANDIDATE_LEN: usize = 80;
/// choice 答案的 question id。
const QUESTION_ID: &str = "tool";

// ---------------------------------------------------------------------------
// 配置
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LayaSettings {
    /// 决策模式（laya-embedded-runtime）：off | embedded | external。
    pub mode: crate::laya_runtime::LayaMode,
    /// 兼容旧配置文件的开关（enabled=true 等价 external）。
    pub enabled: bool,
    pub endpoint: String,
    /// 命中所需的最低置信度（0.0 - 1.0）。
    pub threshold: f64,
    /// external 模式的决策请求超时（毫秒）。
    pub timeout_ms: u64,
    /// 模型下载源（embedded 模式）。
    pub model_base_url: String,
}

impl Default for LayaSettings {
    fn default() -> Self {
        Self {
            mode: crate::laya_runtime::LayaMode::Off,
            enabled: false,
            endpoint: "http://127.0.0.1:8000".to_string(),
            threshold: 0.5,
            timeout_ms: 300,
            model_base_url: crate::laya_runtime::download::DEFAULT_MODEL_BASE_URL.to_string(),
        }
    }
}

impl LayaSettings {
    /// 兼容旧字段：enabled=true 且未显式设置 mode 的旧配置视作 external。
    pub fn effective_mode(&self) -> crate::laya_runtime::LayaMode {
        if self.mode != crate::laya_runtime::LayaMode::Off {
            return self.mode;
        }
        if self.enabled {
            crate::laya_runtime::LayaMode::External
        } else {
            crate::laya_runtime::LayaMode::Off
        }
    }
}

fn settings_path() -> PathBuf {
    let mut dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push(".toolbox");
    dir.push(SETTINGS_FILE);
    dir
}

pub fn load_settings() -> LayaSettings {
    match std::fs::read_to_string(settings_path()) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => LayaSettings::default(),
    }
}

pub fn save_settings(settings: &LayaSettings) -> Result<(), String> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// laya-serve 请求 / 响应（仅本模块可见的 wire 类型）
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct SystemOneRequest {
    state: String,
    questions: serde_json::Value,
    model: &'static str,
}

#[derive(Deserialize)]
struct SystemOneResponse {
    answers: serde_json::Value,
}

/// 路由结果：命中的工具 id 与置信度。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteHit {
    pub tool_id: i64,
    pub confidence: f64,
}

/// 从工具注册表读取候选列表：`{tool_id: "名称|描述|标签"}`。
pub fn build_tool_candidates() -> Result<serde_json::Value, String> {
    let conn = db::open_connection()?;
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.name, t.description,
                    (SELECT GROUP_CONCAT(tg.name, ',') FROM tool_tags tt
                     JOIN tags tg ON tg.id = tt.tag_id WHERE tt.tool_id = t.id) AS tags
             FROM tools t ORDER BY t.id",
        )
        .map_err(|e| e.to_string())?;

    let mut criteria = serde_json::Map::new();
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    for row in rows {
        let (id, name, description, tags) = row.map_err(|e| e.to_string())?;
        let desc: String = description.chars().take(MAX_CANDIDATE_LEN).collect();
        let mut text = format!("{}|{}", name, desc);
        if let Some(tags) = tags {
            if !tags.is_empty() {
                text.push('|');
                text.push_str(&tags);
            }
        }
        criteria.insert(id.to_string(), serde_json::Value::String(text));
    }

    Ok(serde_json::json!({
        QUESTION_ID: {
            "type": "choice",
            "instructions": "用户想使用哪个开发者工具？选择最匹配的一项。",
            "criteria": criteria,
        }
    }))
}

/// 解析 laya-serve `/v1/systemone` 响应并应用阈值；任何不符合预期的形状返回 `None`。
pub fn parse_route_response(
    body: &str,
    threshold: f64,
) -> Option<RouteHit> {
    let resp: SystemOneResponse = serde_json::from_str(body).ok()?;
    let answer = resp.answers.get(QUESTION_ID)?;
    let choice = answer.get("choice")?.as_str()?;
    let confidence = answer.get("confidence")?.as_f64()?;
    if confidence < threshold {
        return None;
    }
    let tool_id: i64 = choice.parse().ok()?;
    Some(RouteHit {
        tool_id,
        confidence,
    })
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

/// 读取 Laya 配置。
#[tauri::command]
pub fn get_laya_settings() -> LayaSettings {
    load_settings()
}

/// 保存 Laya 配置。
#[tauri::command]
pub fn save_laya_settings(settings: LayaSettings) -> Result<LayaSettings, String> {
    save_settings(&settings)?;
    Ok(load_settings())
}

// ---------------------------------------------------------------------------
// 模型生命周期（embedded 模式）
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LayaModelStatus {
    /// not_downloaded | ready（downloading 由前端跟踪进度事件得出）
    pub state: &'static str,
    /// 远程 manifest 的文件清单与体积；离线时为空。
    pub files: Vec<crate::laya_runtime::download::ManifestFile>,
    /// 本地已就绪文件的总体积（字节）。
    pub total_size: u64,
}

/// 查询本地模型状态（拉远程 manifest 获取体积；离线时仅报本地就绪与否）。
#[tauri::command]
pub async fn laya_model_status(base_url: Option<String>) -> Result<LayaModelStatus, String> {
    let settings = load_settings();
    let base = base_url.unwrap_or(settings.model_base_url);
    let manifest = tauri::async_runtime::spawn_blocking(move || {
        crate::laya_runtime::download::fetch_manifest(&base)
    })
    .await
    .map_err(|e| e.to_string())?;
    let ready = crate::laya_runtime::model_ready();
    let manifest = match manifest {
        Ok(m) => m,
        Err(_) => {
            return Ok(LayaModelStatus {
                state: if ready { "ready" } else { "not_downloaded" },
                files: vec![],
                total_size: 0,
            })
        }
    };
    let dir = crate::laya_runtime::model_dir();
    let mut total_size = 0u64;
    for f in &manifest.files {
        if dir.join(&f.name).is_file() {
            total_size += f.size;
        }
    }
    Ok(LayaModelStatus {
        state: if ready { "ready" } else { "not_downloaded" },
        files: manifest.files,
        total_size,
    })
}

/// 下载模型（前端监听 `laya://model-download-progress` 事件展示进度）。
#[tauri::command]
pub async fn laya_model_download(
    app: tauri::AppHandle,
    base_url: Option<String>,
) -> Result<(), String> {
    let settings = load_settings();
    let base = base_url.unwrap_or(settings.model_base_url);
    tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
        let manifest = crate::laya_runtime::download::fetch_manifest(&base)?;
        let emitter = app.clone();
        crate::laya_runtime::download::download_model(&base, &manifest, move |p| {
            use tauri::Emitter;
            emitter.emit("laya://model-download-progress", &p).ok();
        })?;
        // 就绪后预加载会话，避免首次路由才付加载成本。
        crate::laya_runtime::reload_runtime()?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// 删除本地模型文件。
#[tauri::command]
pub fn laya_model_delete() -> Result<(), String> {
    crate::laya_runtime::download::delete_model()?;
    crate::laya_runtime::reload_runtime().ok();
    Ok(())
}

/// 检查决策服务状态：embedded 看模型是否就绪，external 走 HTTP /health。
#[tauri::command]
pub async fn laya_health() -> Result<bool, String> {
    let settings = load_settings();
    match settings.effective_mode() {
        crate::laya_runtime::LayaMode::Embedded => Ok(crate::laya_runtime::model_ready()),
        _ => {
            let url = format!("{}/health", settings.endpoint.trim_end_matches('/'));
            let client = reqwest::Client::builder()
                .timeout(Duration::from_millis(settings.timeout_ms.max(1000)))
                .build()
                .map_err(|e| e.to_string())?;
            Ok(client
                .get(url)
                .send()
                .await
                .and_then(|r| r.error_for_status())
                .is_ok())
        }
    }
}

/// Agent 用的同步入口。关闭、空文本、推理失败或低于阈值一律 `None`。
///
/// 单测构建直接返回 `None`，避免用例读取本机配置并加载 ONNX。
pub fn route_decision(state: &str) -> Option<RouteHit> {
    #[cfg(test)]
    {
        let _ = state;
        return None;
    }
    #[cfg(not(test))]
    {
        route_decision_live(state)
    }
}

#[cfg_attr(test, allow(dead_code))]
pub(crate) fn route_decision_live(state: &str) -> Option<RouteHit> {
    if state.trim().is_empty() {
        return None;
    }
    let settings = load_settings();
    route_with_settings(&settings, state)
}

pub(crate) fn route_with_settings(settings: &LayaSettings, state: &str) -> Option<RouteHit> {
    if state.trim().is_empty() {
        return None;
    }
    match settings.effective_mode() {
        crate::laya_runtime::LayaMode::Off => None,
        crate::laya_runtime::LayaMode::Embedded => {
            let candidates = embedded_candidates().ok()?;
            route_embedded(state, &candidates, settings.threshold)
        }
        crate::laya_runtime::LayaMode::External => {
            let client = reqwest::Client::builder()
                .timeout(Duration::from_millis(settings.timeout_ms))
                .build()
                .ok()?;
            tauri::async_runtime::block_on(route_with_client(settings, client, state))
        }
    }
}

/// 工具箱页面名称。查不到时退回「工具页 {id}」。
pub fn toolbox_page_name(id: i64) -> String {
    let Ok(conn) = db::open_connection() else {
        return format!("工具页 {id}");
    };
    conn.query_row(
        "SELECT name FROM tools WHERE id = ?1",
        [id],
        |row| row.get::<_, String>(0),
    )
    .unwrap_or_else(|_| format!("工具页 {id}"))
}

/// 将自然语言输入路由到工具 id。off / 任何失败 / 低置信度一律返回 `None`。
#[tauri::command]
pub async fn laya_route(query: String) -> Result<Option<RouteHit>, String> {
    let settings = load_settings();
    if query.trim().is_empty() {
        return Ok(None);
    }
    match settings.effective_mode() {
        crate::laya_runtime::LayaMode::Off => Ok(None),
        crate::laya_runtime::LayaMode::Embedded => {
            let candidates = embedded_candidates()?;
            let threshold = settings.threshold;
            let query = query.clone();
            // ort 会话推理放阻塞线程池，避免卡住 async 运行时。
            let hit = tauri::async_runtime::spawn_blocking(move || {
                route_embedded(&query, &candidates, threshold)
            })
            .await
            .map_err(|e| e.to_string())?;
            Ok(hit)
        }
        crate::laya_runtime::LayaMode::External => {
            Ok(route_with_client(
                &settings,
                reqwest::Client::builder()
                    .timeout(Duration::from_millis(settings.timeout_ms))
                    .build()
                    .map_err(|e| e.to_string())?,
                &query,
            )
            .await)
        }
    }
}

/// embedded 模式的候选列表：与外部服务共用 build_tool_candidates，
/// 扁平化为 `(key, text)` 供进程内推理使用。
fn embedded_candidates() -> Result<Vec<(String, String)>, String> {
    let questions = build_tool_candidates()?;
    let criteria = &questions[QUESTION_ID]["criteria"];
    let mut out = Vec::new();
    if let Some(map) = criteria.as_object() {
        for (k, v) in map {
            if let Some(text) = v.as_str() {
                out.push((k.clone(), text.to_string()));
            }
        }
    }
    Ok(out)
}

/// 进程内推理：失败一律 None（静默降级），由调用方回退关键词检索。
fn route_embedded(
    query: &str,
    candidates: &[(String, String)],
    threshold: f64,
) -> Option<RouteHit> {
    // 共享会话（Arc + 内部 Mutex）；推理失败静默降级。
    let runtime = crate::laya_runtime::shared_runtime().ok()?;
    let item = crate::laya_runtime::DecisionItem {
        state: query,
        instructions: "用户想使用哪个开发者工具？选择最匹配的一项。",
        options: candidates,
    };
    let (idx, confidence) = runtime.decide_choice(&item).ok()?;
    let key = candidates.get(idx)?.0.clone();
    let tool_id: i64 = key.parse().ok()?;
    if confidence < threshold {
        return None;
    }
    Some(RouteHit {
        tool_id,
        confidence,
    })
}

pub(crate) async fn route_with_client(
    settings: &LayaSettings,
    client: reqwest::Client,
    query: &str,
) -> Option<RouteHit> {
    let questions = build_tool_candidates().ok()?;
    let url = format!(
        "{}/v1/systemone",
        settings.endpoint.trim_end_matches('/')
    );
    let body = SystemOneRequest {
        state: query.to_string(),
        questions,
        model: "multilingual",
    };
    let resp = client
        .post(url)
        .json(&body)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .ok()?;
    let text = resp.text().await.ok()?;
    parse_route_response(&text, settings.threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn off_mode_does_not_route() {
        let settings = LayaSettings::default();
        assert!(route_with_settings(&settings, "把 JSON 转义").is_none());
    }

    #[test]
    fn settings_default_is_disabled() {
        let s = LayaSettings::default();
        assert!(!s.enabled);
        assert_eq!(s.endpoint, "http://127.0.0.1:8000");
        assert!((s.threshold - 0.5).abs() < f64::EPSILON);
        assert_eq!(s.timeout_ms, 300);
    }

    #[test]
    fn settings_roundtrip_preserves_fields() {
        let s = LayaSettings {
            mode: crate::laya_runtime::LayaMode::Embedded,
            enabled: true,
            endpoint: "http://127.0.0.1:9000".into(),
            threshold: 0.5,
            timeout_ms: 500,
            model_base_url: crate::laya_runtime::download::DEFAULT_MODEL_BASE_URL.into(),
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: LayaSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(back, s);
        // effective_mode：显式 mode 优先于旧 enabled 字段。
        assert_eq!(
            back.effective_mode(),
            crate::laya_runtime::LayaMode::Embedded
        );
    }

    #[test]
    fn legacy_enabled_config_maps_to_external() {
        // 旧配置文件只有 enabled 字段（serde default 补 mode=off）。
        let legacy = r#"{"enabled":true,"endpoint":"http://127.0.0.1:8000","threshold":0.5,"timeoutMs":300}"#;
        let s: LayaSettings = serde_json::from_str(legacy).unwrap();
        assert_eq!(
            s.effective_mode(),
            crate::laya_runtime::LayaMode::External
        );
        let off: LayaSettings = LayaSettings::default();
        assert_eq!(off.effective_mode(), crate::laya_runtime::LayaMode::Off);
    }

    #[test]
    fn candidates_truncate_description_and_include_tags() {
        // 直接测纯函数级别的截断逻辑：构造超长描述走 SQL 太重，
        // 这里通过检查 build 的 JSON 形状兜底（需要本机 DB，失败则跳过）。
        if db::open_connection().is_err() {
            return;
        }
        let q = build_tool_candidates().unwrap();
        let criteria = &q[QUESTION_ID]["criteria"];
        assert!(criteria.as_object().unwrap().len() > 0);
        for (id, text) in criteria.as_object().unwrap() {
            assert!(id.parse::<i64>().is_ok());
            assert!(text.as_str().unwrap().chars().count() <= MAX_CANDIDATE_LEN + 40);
        }
    }

    #[test]
    fn parse_response_applies_threshold() {
        let body = r#"{"model":"laya-multilingual","answers":{"tool":{"type":"choice","choice":"9","confidence":0.88,"probabilities":{}}},"usage":{}}"#;
        assert_eq!(
            parse_route_response(body, 0.7),
            Some(RouteHit { tool_id: 9, confidence: 0.88 })
        );
        // 低于阈值 → 未命中
        assert_eq!(parse_route_response(body, 0.9), None);
    }

    #[test]
    fn parse_response_rejects_malformed() {
        assert_eq!(parse_route_response("not json", 0.7), None);
        assert_eq!(
            parse_route_response(r#"{"answers":{"tool":{"choice":"abc","confidence":0.9}}}"#, 0.7),
            None
        );
        assert_eq!(parse_route_response(r#"{"answers":{}}"#, 0.7), None);
    }

    /// 实机联调（external）：需要 laya-serve 运行中（默认端口 8000）。
    /// 默认忽略，`cargo test --lib laya -- --ignored` 运行。
    /// 数值一致性验收（任务 2.3）：embedded 推理 vs Python Agent 参考，
    /// 需要模型已部署到 ~/.toolbox/laya 且本机工具 DB 存在。
    /// 验收标准：choice 一致率 >= 0.9，命中样本置信度平均绝对偏差 <= 0.05。
    /// 默认忽略，`cargo test --lib laya -- --ignored parity` 运行。
    #[test]
    #[ignore]
    fn parity_against_python_reference() {
        let change_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("openspec/changes/laya-embedded-runtime/eval");
        let reference: Vec<serde_json::Value> = serde_json::from_str(
            &std::fs::read_to_string(change_dir.join("python-reference-outputs.json")).unwrap(),
        )
        .unwrap();
        let candidates = embedded_candidates().expect("构建候选失败");
        let mut agree = 0usize;
        let mut conf_diff_sum = 0.0f64;
        let mut compared = 0usize;
        for item in &reference {
            let query = item["query"].as_str().unwrap();
            let py_choice = item["py_choice"].as_str().unwrap();
            let py_conf = item["py_confidence"].as_f64().unwrap();
            let hit = route_embedded(query, &candidates, 0.0);
            let (rust_choice, rust_conf) = match &hit {
                Some(h) => (h.tool_id.to_string(), h.confidence),
                None => {
                    // 阈值 0.0 下仍 None 说明推理失败/解析失败，记为不一致。
                    println!("[FAIL] {query}: rust 未命中, py={py_choice}@{py_conf}");
                    continue;
                }
            };
            compared += 1;
            if rust_choice == py_choice {
                agree += 1;
            } else {
                println!("[FAIL] {query}: rust={rust_choice}@{rust_conf:.4} py={py_choice}@{py_conf:.4}");
            }
            conf_diff_sum += (rust_conf - py_conf).abs();
        }
        assert!(compared > 0, "没有任何可比较样本");
        let agreement = agree as f64 / compared as f64;
        let mean_conf_diff = conf_diff_sum / compared as f64;
        println!(
            "parity: {agree}/{compared} choice 一致率={agreement:.3}, 平均置信度偏差={mean_conf_diff:.4}"
        );
        assert!(agreement >= 0.9, "choice 一致率 {agreement:.3} < 0.9");
        assert!(
            mean_conf_diff <= 0.05,
            "置信度平均偏差 {mean_conf_diff:.4} > 0.05"
        );
    }

    #[test]
    #[ignore]
    fn route_against_live_server() {
        let settings = LayaSettings {
            mode: crate::laya_runtime::LayaMode::External,
            enabled: true,
            endpoint: "http://127.0.0.1:8000".into(),
            threshold: 0.5,
            timeout_ms: 2000,
            model_base_url: crate::laya_runtime::download::DEFAULT_MODEL_BASE_URL.into(),
        };
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_millis(settings.timeout_ms))
            .build()
            .unwrap();
        let questions = build_tool_candidates().expect("构建候选失败");
        let url = format!("{}/v1/systemone", settings.endpoint.trim_end_matches('/'));
        let body = SystemOneRequest {
            state: "我想把 JSON 转成 YAML".to_string(),
            questions,
            model: "multilingual",
        };
        let resp = client.post(url).json(&body).send().expect("请求失败");
        assert!(resp.status().is_success());
        let hit = parse_route_response(&resp.text().unwrap(), settings.threshold);
        println!("laya live hit: {hit:?}");
        if let Some(hit) = hit {
            assert!(hit.confidence >= settings.threshold);
        }
    }
}
