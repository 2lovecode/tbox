pub mod tool;
pub mod file;
pub mod llm;
pub mod llm_presets;
pub mod ollama_pull;
pub mod image;
pub mod pdf;
pub mod code;
pub mod file_ops;
pub mod json;
pub mod encoding;
pub mod screen;
pub mod search;
pub mod conversation;
pub mod agent;
pub mod model_catalog;
pub mod hardware_info;
pub mod data_convert;
pub mod crypto;
pub mod text_utils;
pub mod network;
pub mod xml_utils;
pub mod yaml_utils;
pub mod gm_crypto;
pub mod sql_utils;
pub mod db_tools;
pub mod image_utils;
pub mod csv_utils;
pub mod log_analyzer;
pub mod color_tools;
pub mod qrcode_tools;
pub mod uuid_tools;
pub mod cron_tools;
pub mod number_tools;
pub mod charset_tools;

// 未接入（保留文件，暂不注册到 generate_handler!）：
// - json_diff: `compare_json` 已由 json 模块提供，该模块被取代
// - datetime:  时间戳转换当前为纯前端实现，无 Rust 调用方
// - system:    前端暂无调用
