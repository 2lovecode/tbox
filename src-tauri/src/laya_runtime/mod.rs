//! 内置 Laya ONNX 推理运行时（change: laya-embedded-runtime）。
//!
//! 进程内复刻 Python `laya` 的 choice 决策路径：mmBERT 分词 →
//! `build_sequence` 序列构造（`[CLS] <type> instructions [SEP] [MASK] opt... [SEP] state [SEP]`）
//! → ONNX 单次前向 → 温度校准 softmax → argmax + `answer_confidence = max(p)`。
//! 复刻依据见本 change `eval/python-reference.md`。
//!
//! 仅支持固定 multilingual checkpoint 的 choice 决策（见 design Non-Goals）。

pub mod download;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tokenizers::Tokenizer;

/// choice 的 qtype 编号（`laya.common.QTYPES`）。
const QTYPE_CHOICE: i64 = 0;
/// 选项 token 上限（`build_head` 中 `max_length=48`）。
const OPTION_TOKEN_CAP: usize = 48;
/// 温度 clamp 边界（`laya.common.TEMP_MIN/MAX`）。
const TEMP_MIN: f64 = 0.5;
const TEMP_MAX: f64 = 5.0;

/// 模型目录：`~/.toolbox/laya`（与 db.rs 的 `~/.toolbox` 约定一致）。
pub fn model_dir() -> PathBuf {
    let mut dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push(".toolbox");
    dir.push("laya");
    dir
}

/// 模型文件清单（与 download.rs 的 manifest 对应）。
pub const MODEL_FILES: &[&str] = &[
    "laya-multilingual.onnx",
    // torch exporter 的外部权重文件（fp32，约 1.2GB）
    "laya-multilingual.onnx.data",
    "tokenizer.json",
    "rl_agent_config.json",
];

/// 模型是否已下载就绪（全部文件存在）。
pub fn model_ready() -> bool {
    let dir = model_dir();
    MODEL_FILES.iter().all(|f| dir.join(f).is_file())
}

// ---------------------------------------------------------------------------
// checkpoint 配置
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct RlAgentConfig {
    #[serde(default = "default_max_len")]
    max_len: usize,
    #[serde(default = "default_head_max_len")]
    head_max_len: usize,
    #[serde(default)]
    temperature: Vec<f64>,
}

fn default_max_len() -> usize {
    1024
}
fn default_head_max_len() -> usize {
    256
}

impl RlAgentConfig {
    /// choice（qtype 0）的生效温度，clamp 到 [0.5, 5.0]，缺失/非法回退 1.0。
    fn choice_temperature(&self) -> f64 {
        let raw = self.temperature.first().copied().unwrap_or(1.0);
        if !raw.is_finite() {
            return 1.0;
        }
        raw.clamp(TEMP_MIN, TEMP_MAX)
    }
}

// ---------------------------------------------------------------------------
// 运行时会话
// ---------------------------------------------------------------------------

pub struct LayaRuntime {
    tokenizer: Tokenizer,
    /// ORT 的 run 需要 `&mut Session`；共享会话用 Mutex 串行化（单前向 ~30ms，够用）。
    session: std::sync::Mutex<ort::session::Session>,
    pub(crate) config: RlAgentConfig,
    pub(crate) cls_id: i64,
    pub(crate) sep_id: i64,
    pub(crate) mask_id: i64,
}

static RUNTIME: std::sync::Mutex<Option<Result<std::sync::Arc<LayaRuntime>, String>>> =
    std::sync::Mutex::new(None);

/// 获取全局运行时（Arc 克隆，廉价）；首次调用加载模型（懒加载），
/// 加载失败结果同样被缓存，直到模型文件变化后调用 [`reload_runtime`]。
pub fn shared_runtime() -> Result<std::sync::Arc<LayaRuntime>, String> {
    let mut guard = RUNTIME.lock().map_err(|e| e.to_string())?;
    if guard.is_none() {
        *guard = Some(load().map(std::sync::Arc::new));
    }
    match guard.as_ref().unwrap() {
        Ok(rt) => Ok(rt.clone()),
        Err(e) => Err(e.clone()),
    }
}

/// 下载完成 / 模型文件变更后调用：丢弃缓存并在下次访问时重新加载。
pub fn reload_runtime() -> Result<(), String> {
    if let Ok(mut guard) = RUNTIME.lock() {
        *guard = None;
    }
    shared_runtime().map(|_| ())
}

fn load() -> Result<LayaRuntime, String> {
    let dir = model_dir();
    let onnx_path = dir.join("laya-multilingual.onnx");
    let tokenizer_path = dir.join("tokenizer.json");
    let config_path = dir.join("rl_agent_config.json");
    for p in [&onnx_path, &tokenizer_path, &config_path] {
        if !p.is_file() {
            return Err(format!("模型文件缺失：{}", p.display()));
        }
    }

    let tokenizer = Tokenizer::from_file(&tokenizer_path)
        .map_err(|e| format!("tokenizer 加载失败：{e}"))?;
    let config: RlAgentConfig = serde_json::from_str(
        &std::fs::read_to_string(&config_path).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("rl_agent_config.json 解析失败：{e}"))?;

    // 特殊 token id：按 BERT 风格 ([CLS]…) 优先，回退 mmBERT/XLM-R 风格 (<bos>/<eos>/<mask>)。
    let special = |names: &[&str]| -> Result<i64, String> {
        for n in names {
            if let Some(id) = tokenizer.token_to_id(n) {
                return Ok(id as i64);
            }
        }
        Err(format!("tokenizer 缺少特殊 token：{names:?}"))
    };
    let cls_id = special(&["[CLS]", "<bos>"])?;
    let sep_id = special(&["[SEP]", "<eos>"])?;
    let mask_id = special(&["[MASK]", "<mask>"])?;

    let env = ort_environment();
    let session = ort::session::Session::builder(&env)
        .map_err(|e| format!("ORT builder 失败：{e}"))?
        .with_optimization_level(ort::session::builder::GraphOptimizationLevel::Level3)
        .map_err(|e| e.to_string())?
        .with_intra_threads(4)
        .map_err(|e| e.to_string())?
        .commit_from_file(&onnx_path)
        .map_err(|e| format!("ONNX 加载失败：{e}"))?;

    Ok(LayaRuntime {
        tokenizer,
        session: std::sync::Mutex::new(session),
        config,
        cls_id,
        sep_id,
        mask_id,
    })
}

/// 进程级 ONNX Runtime 环境（`ort::init` 幂等初始化）。
fn ort_environment() -> &'static ort::environment::Environment {
    static ENV: std::sync::OnceLock<ort::environment::Environment> = std::sync::OnceLock::new();
    ENV.get_or_init(|| {
        ort::init()
            .with_name("tbox-laya")
            .build()
            .expect("ONNX Runtime 环境初始化失败")
    })
}

// ---------------------------------------------------------------------------
// 序列构造（对齐 laya.common.build_head / build_sequence）
// ---------------------------------------------------------------------------

/// 一个待决策项：question half + state，输出 ids 与每个选项的 [MASK] marker 位置。
pub struct DecisionItem<'a> {
    pub state: &'a str,
    pub instructions: &'a str,
    /// `(key, description)` 选项列表，顺序即候选顺序。
    pub options: &'a [(String, String)],
}

impl LayaRuntime {
    /// 编码文本为无特殊 token 的 id 序列；`cap` 为 Some 时使用 tokenizer 截断
    /// （对齐 `truncation=True, max_length=cap`：保留前 cap 个 token）。
    fn encode(&self, text: &str, cap: Option<usize>) -> Result<Vec<i64>, String> {
        let enc = self
            .tokenizer
            .encode(text, false)
            .map_err(|e| format!("分词失败：{e}"))?;
        let ids: Vec<i64> = enc.get_ids().iter().map(|&i| i as i64).collect();
        Ok(match cap {
            Some(n) => ids.into_iter().take(n).collect(),
            None => ids,
        })
    }

    pub(crate) fn build_item(&self, item: &DecisionItem) -> Result<(Vec<i64>, Vec<i64>), String> {
        let max_len = self.config.max_len;
        let head_max_len = self.config.head_max_len;

        // question half（build_head）："[CLS] <t> question: <ins> [SEP] [MASK] opt..."
        let ins = item.instructions.replace("[MASK]", " ");
        let question_text = format!("{QTYPE_NAME_CHOICE} question: {ins}");
        let mut head_ids = self.encode(&question_text, None)?;
        let mut opt_ids: Vec<Vec<i64>> = Vec::with_capacity(item.options.len());
        // 选项文本 = "key: value"（render_options choice 分支；value 空则只有 key）。
        for (key, desc) in item.options {
            let desc = desc.replace("[MASK]", " ");
            let rendered = if desc.is_empty() {
                key.clone()
            } else {
                format!("{key}: {desc}")
            };
            let text = format!(" {rendered}");
            let mut o = vec![self.mask_id];
            o.extend(self.encode(&text, Some(OPTION_TOKEN_CAP))?);
            opt_ids.push(o);
        }
        // 选项预算：总和挤占 question 长度（build_head 的 per-option clamp 分支）。
        let opt_budget = head_max_len.saturating_sub(opt_ids.iter().map(|o| o.len()).sum::<usize>());
        if opt_budget < 16 {
            let per = (head_max_len.saturating_sub(16) / item.options.len().max(1)).max(4);
            for o in &mut opt_ids {
                o.truncate(per);
            }
            let opt_budget =
                head_max_len.saturating_sub(opt_ids.iter().map(|o| o.len()).sum::<usize>());
            head_ids.truncate(opt_budget.max(8));
        } else {
            head_ids.truncate(opt_budget);
        }

        let mut ids = vec![self.cls_id];
        ids.extend(head_ids);
        ids.push(self.sep_id);
        let mut markers = Vec::with_capacity(opt_ids.len());
        for o in opt_ids {
            markers.push(ids.len() as i64);
            ids.extend(o);
        }
        ids.push(self.sep_id);
        let head_len = ids.len();

        // state half（build_sequence）：截断到剩余空间，取前 room 个 token。
        let room = max_len.saturating_sub(head_len + 1);
        let state_text = item.state.replace("[MASK]", " ");
        let state_ids = self.encode(&state_text, None)?;
        ids.extend(state_ids.into_iter().take(room));
        ids.push(self.sep_id);
        ids.truncate(max_len);
        markers.retain(|&m| (m as usize) < ids.len());

        Ok((ids, markers))
    }

    /// 对一个决策项做 choice 推理，返回 `(选项下标, 置信度)`。
    pub fn decide_choice(&self, item: &DecisionItem) -> Result<(usize, f64), String> {
        let (ids, markers) = self.build_item(item)?;
        if markers.is_empty() {
            return Err("marker 为空（序列被截断到无可选项）".to_string());
        }
        let n = ids.len();
        let k = markers.len();

        let input_ids = ort::value::Tensor::<i64>::from_array((vec![1usize, n], ids))
            .map_err(|e| format!("张量创建失败：{e}"))?;
        let attention_mask = ort::value::Tensor::<i64>::from_array((vec![1usize, n], vec![1i64; n]))
            .map_err(|e| format!("张量创建失败：{e}"))?;
        let marker_pos = ort::value::Tensor::<i64>::from_array((vec![1usize, k], markers))
            .map_err(|e| format!("张量创建失败：{e}"))?;
        // 导出图中 marker_mask 为 bool 张量（collate_items 的 dtype=torch.bool）。
        let marker_mask = ort::value::Tensor::<bool>::from_array((vec![1usize, k], vec![true; k]))
            .map_err(|e| format!("张量创建失败：{e}"))?;
        let qtype = ort::value::Tensor::<i64>::from_array((vec![1usize], vec![QTYPE_CHOICE]))
            .map_err(|e| format!("张量创建失败：{e}"))?;

        let mut session = self.session.lock().map_err(|e| e.to_string())?;
        let outputs = session
            .run(ort::inputs![
                "input_ids" => input_ids,
                "attention_mask" => attention_mask,
                "marker_pos" => marker_pos,
                "marker_mask" => marker_mask,
                "qtype" => qtype,
            ])
            .map_err(|e| format!("推理失败：{e}"))?;

        let (shape, logits) = outputs["logits"]
            .try_extract_tensor::<f32>()
            .map_err(|e| format!("输出提取失败：{e}"))?;
        if shape.len() != 2 || shape[0] < 1 {
            return Err(format!("logits 形状异常：{shape:?}"));
        }
        let k_out = shape[1] as usize;
        let row = &logits[..k_out.min(logits.len())];

        // 温度校准 softmax（choice 温度来自 rl_agent_config.json，clamp [0.5,5]）。
        let t = self.config.choice_temperature() as f32;
        let max_logit = row.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let exps: Vec<f32> = row.iter().map(|&l| ((l - max_logit) / t).exp()).collect();
        let sum: f32 = exps.iter().sum();
        let probs: Vec<f64> = exps.iter().map(|&e| (e / sum) as f64).collect();

        let (best, &conf) = probs
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .ok_or("空 logits")?;
        Ok((best, conf.clamp(0.0, 1.0)))
    }
}

const QTYPE_NAME_CHOICE: &str = "choice";

/// 决策模式（对应 spec：off | embedded | external）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LayaMode {
    #[default]
    Off,
    Embedded,
    External,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choice_temperature_clamps() {
        let cfg = RlAgentConfig {
            max_len: 1024,
            head_max_len: 256,
            temperature: vec![0.10058],
        };
        assert!((cfg.choice_temperature() - TEMP_MIN).abs() < 1e-9);
        let cfg = RlAgentConfig {
            max_len: 1024,
            head_max_len: 256,
            temperature: vec![9.9],
        };
        assert!((cfg.choice_temperature() - TEMP_MAX).abs() < 1e-9);
        let cfg = RlAgentConfig {
            max_len: 1024,
            head_max_len: 256,
            temperature: vec![1.0],
        };
        assert!((cfg.choice_temperature() - 1.0).abs() < 1e-9);
        let cfg = RlAgentConfig::default();
        assert!((cfg.choice_temperature() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn mode_default_is_off() {
        assert_eq!(LayaMode::default(), LayaMode::Off);
        assert_eq!(
            serde_json::to_string(&LayaMode::Embedded).unwrap(),
            "\"embedded\""
        );
    }
}
