//! Tauri library entry point.
//!
//! Per the Tauri 2.x convention, all builder configuration lives here and
//! `main.rs` simply calls [`run`]. This keeps the entry point tiny and lets
//! the same setup be reused by the mobile entry point and integration tests.

use tauri::Emitter;
use tauri_plugin_global_shortcut::ShortcutState;

pub mod agent;
pub mod commands;
mod db;

/// Event name emitted to the frontend when the user invokes the global
/// Spotlight shortcut. The frontend listens on this channel to toggle the
/// Spotlight search modal.
pub const SPOTLIGHT_EVENT: &str = "spotlight:toggle";

/// Build and launch the Tauri application.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize the persistent database and seed it on first launch.
    if let Err(e) = commands::tool::init_db_if_needed() {
        eprintln!("初始化数据库失败: {}", e);
    }

    // Warm up the in-memory search index before the first query lands.
    if let Ok(tools) = commands::tool::get_all_tools() {
        let search_results: Vec<commands::search::SearchResult> = tools
            .into_iter()
            .map(|t| commands::search::SearchResult {
                id: t.id,
                name: t.name,
                description: t.description,
                icon: t.icon,
                category_id: t.category.as_ref().map(|c| c.id),
                category_name: t.category.as_ref().map(|c| c.name.clone()),
                tags: t.tags,
                gradient: t.gradient,
            })
            .collect();
        commands::search::build_search_index(search_results);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        if let Err(e) = app.emit(SPOTLIGHT_EVENT, ()) {
                            eprintln!("[tbox] 派发 Spotlight 事件失败: {}", e);
                        }
                    }
                })
                .build(),
        )
        .manage(commands::agent::AgentCancel::default())
        .manage(commands::model_catalog::DownloadCancels::default())
        .manage(commands::ollama_pull::PullCancels::default())
        .setup(|app| {
            use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

            // Give the embedded LLM engine an app handle for status events.
            agent::embedded_engine::set_app_handle(app.handle().clone());

            // Default shortcuts registered at startup. Ctrl+Space is NOT
            // registered: it conflicts with common IME toggle habits. macOS
            // uses Cmd+Shift+Space (Cmd+Space is claimed by the system
            // Spotlight); other platforms use Ctrl+Shift+Space. The plugin
            // logs and skips any shortcut that fails to register (e.g. due
            // to OS conflicts).
            let defaults: Vec<Shortcut> = if cfg!(target_os = "macos") {
                vec![Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::Space)]
            } else {
                vec![Shortcut::new(
                    Some(Modifiers::CONTROL | Modifiers::SHIFT),
                    Code::Space,
                )]
            };

            for shortcut in defaults {
                if let Err(e) = app.global_shortcut().register(shortcut) {
                    eprintln!("[tbox] 注册全局快捷键 {:?} 失败: {}", shortcut, e);
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 工具管理
            commands::tool::get_categories,
            commands::tool::get_all_tools,
            // 搜索
            commands::search::search_tools,
            // Story 5.1 (Phase 1.5 v0): 本地意图路由
            commands::search::ai_route_intent,

            // Story 5.1 v1: LLM provider configuration
            commands::llm::get_llm_config,
            commands::llm::save_llm_config,
            commands::llm::clear_llm_api_key,
            commands::llm::delete_llm_config,
            commands::llm::test_llm_connection,
            commands::llm::list_llm_presets,
            // multi-provider-models: 多提供方 profile 配置
            commands::llm::list_llm_profiles,
            commands::llm::save_llm_profile,
            commands::llm::delete_llm_profile,
            commands::llm::set_active_llm_profile,
            commands::llm::clear_llm_profile_api_key,
            commands::llm::list_profile_models,
            commands::llm::list_endpoint_models,
            commands::llm::reveal_llm_profile_api_key,

            // 文件操作
            commands::file::download_file,
            commands::file_ops::list_directory,
            commands::file_ops::get_file_size,
            commands::file_ops::file_exists,

            // 屏幕标尺
            commands::screen::get_window_info,
            commands::screen::get_global_mouse_position,
            commands::screen::calculate_global_position,

            // 图片处理
            commands::image::compress_image,
            commands::image::get_image_info,

            // PDF处理
            commands::pdf::merge_pdfs,
            commands::pdf::split_pdf,
            commands::pdf::compress_pdf,

            // 代码格式化
            commands::code::format_code,
            commands::code::format_json,

            // JSON处理
            commands::json::format_json_pretty,
            commands::json::compress_json,
            commands::json::escape_json,
            commands::json::unescape_json,
            commands::json::validate_json,
            commands::json::get_json_info,
            commands::json::compare_json,
            commands::json::json_to_query_params,

            // 编码工具
            commands::encoding::url_encode,
            commands::encoding::url_decode,
            commands::encoding::unicode_to_chinese,
            commands::encoding::chinese_to_unicode,
            commands::encoding::html_encode,
            commands::encoding::html_decode,
            commands::encoding::base58_encode,
            commands::encoding::base58_decode,
            commands::encoding::base62_encode,
            commands::encoding::base62_decode,
            commands::encoding::hex_to_string,
            commands::encoding::string_to_hex,
            commands::encoding::punycode_encode,
            commands::encoding::punycode_decode,
            commands::encoding::binary_to_hex,
            commands::encoding::hex_to_binary,

            // 会话
            commands::conversation::list_conversations,
            commands::conversation::get_conversation_messages,
            commands::conversation::delete_conversation,
            commands::conversation::append_user_message,

            // Agent 对话
            commands::agent::check_llm_ready,
            commands::agent::send_chat_turn,
            commands::agent::cancel_chat_turn,
            commands::agent::get_engine_status,

            // 本地模型目录
            commands::model_catalog::list_local_models,
            commands::model_catalog::start_model_download,
            commands::model_catalog::cancel_model_download,
            commands::ollama_pull::start_ollama_pull,
            commands::ollama_pull::cancel_ollama_pull,

            // 硬件信息
            commands::hardware_info::get_hardware_info,

            // JSON → 实体类
            commands::data_convert::json_to_java,
            commands::data_convert::json_to_csharp,
            commands::data_convert::json_to_go,
            commands::data_convert::json_to_python,
            commands::data_convert::json_to_typescript,

            // 加密与安全（AES / RSA / HMAC / JWT / 哈希）
            commands::crypto::aes_encrypt,
            commands::crypto::aes_decrypt,
            commands::crypto::generate_rsa_keypair,
            commands::crypto::hmac_sha256_sign,
            commands::crypto::hmac_sha512_sign,
            commands::crypto::parse_jwt,
            commands::crypto::generate_jwt,
            commands::crypto::sha256_hash,
            commands::crypto::sha512_hash,
            commands::crypto::md5_hash,
            commands::crypto::sha1_hash,

            // 文本工具集
            commands::text_utils::regex_test,
            commands::text_utils::regex_replace,
            commands::text_utils::text_compare,
            commands::text_utils::text_deduplicate,
            commands::text_utils::text_sort,
            commands::text_utils::text_reverse,
            commands::text_utils::text_statistics,
            commands::text_utils::convert_naming,
            commands::text_utils::convert_case,

            // 网络工具
            commands::network::http_request,
            commands::network::dns_lookup,
            commands::network::check_port_open,
            commands::network::get_public_ip,
            commands::network::ping_test,
            commands::network::get_ssl_cert,

            // XML
            commands::xml_utils::format_xml,
            commands::xml_utils::minify_xml,
            commands::xml_utils::xml_to_json,
            commands::xml_utils::json_to_xml,
            commands::xml_utils::xml_to_yaml,
            commands::xml_utils::yaml_to_xml,
            commands::xml_utils::xpath_query,

            // YAML
            commands::yaml_utils::format_yaml,
            commands::yaml_utils::yaml_to_json,
            commands::yaml_utils::json_to_yaml,
            commands::yaml_utils::validate_yaml,
            commands::yaml_utils::merge_yaml,

            // 国密 SM2/SM3/SM4
            commands::gm_crypto::sm3_hash,
            commands::gm_crypto::sm4_encrypt,
            commands::gm_crypto::sm4_decrypt,
            commands::gm_crypto::generate_sm4_key,
            commands::gm_crypto::generate_sm2_keypair,
            commands::gm_crypto::sm2_sign,
            commands::gm_crypto::sm2_verify,
            commands::gm_crypto::hmac_sm3,

            // SQL 格式化
            commands::sql_utils::format_sql,
            commands::sql_utils::minify_sql,
            commands::sql_utils::escape_sql,
            commands::sql_utils::unescape_sql,

            // 数据库工具
            commands::db_tools::test_mysql_connection,
            commands::db_tools::test_postgres_connection,
            commands::db_tools::test_sqlite_connection,
            commands::db_tools::execute_mysql_query,
            commands::db_tools::execute_postgres_query,
            commands::db_tools::execute_sqlite_query,

            // 图片工具集
            commands::image_utils::convert_image_format,
            commands::image_utils::resize_image,
            commands::image_utils::crop_image,
            commands::image_utils::rotate_image,
            commands::image_utils::flip_image,
            commands::image_utils::compress_image_quality,
            commands::image_utils::get_detailed_image_info,
            commands::image_utils::add_watermark,
            commands::image_utils::image_to_base64,
            commands::image_utils::base64_to_image,

            // CSV
            commands::csv_utils::csv_to_json,
            commands::csv_utils::json_to_csv,
            commands::csv_utils::format_csv,
            commands::csv_utils::csv_to_excel,
            commands::csv_utils::csv_stats,

            // 日志分析
            commands::log_analyzer::analyze_logs,
            commands::log_analyzer::extract_log_levels,
            commands::log_analyzer::filter_logs,
            commands::log_analyzer::count_errors,
            commands::log_analyzer::extract_logs_by_time,
            commands::log_analyzer::highlight_logs,
            commands::log_analyzer::find_duplicate_logs,
            commands::log_analyzer::generate_log_report,

            // 颜色转换
            commands::color_tools::rgb_to_hex,
            commands::color_tools::hex_to_rgb,
            commands::color_tools::rgb_to_hsl,
            commands::color_tools::hsl_to_rgb,
            commands::color_tools::rgb_to_hsv,
            commands::color_tools::hsv_to_rgb,
            commands::color_tools::parse_color,
            commands::color_tools::random_color,
            commands::color_tools::adjust_brightness,

            // 二维码 / 条形码
            commands::qrcode_tools::generate_qrcode,
            commands::qrcode_tools::parse_qrcode,
            commands::qrcode_tools::generate_barcode,

            // UUID
            commands::uuid_tools::generate_uuid_v4,
            commands::uuid_tools::generate_uuid_v7,
            commands::uuid_tools::generate_uuid_v5,
            commands::uuid_tools::generate_uuid_batch,
            commands::uuid_tools::validate_uuid,
            commands::uuid_tools::uuid_to_base64,
            commands::uuid_tools::base64_to_uuid,
            commands::uuid_tools::get_uuid_version,
            commands::uuid_tools::nil_uuid,

            // Cron 表达式
            commands::cron_tools::generate_cron,
            commands::cron_tools::get_next_cron_time,
            commands::cron_tools::cron_to_natural_language,
            commands::cron_tools::validate_cron,

            // 数字 / 进制
            commands::number_tools::dec_to_hex,
            commands::number_tools::hex_to_dec,
            commands::number_tools::dec_to_binary,
            commands::number_tools::binary_to_dec,
            commands::number_tools::dec_to_octal,
            commands::number_tools::octal_to_dec,
            commands::number_tools::scientific_to_decimal,
            commands::number_tools::to_roman,
            commands::number_tools::from_roman,
            commands::number_tools::fraction_to_decimal,
            commands::number_tools::decimal_to_fraction,

            // 字符集 / 编解码扩展（punycode 复用 encoding 模块）
            commands::charset_tools::detect_encoding,
            commands::charset_tools::convert_encoding,
            commands::charset_tools::url_encode_component,
            commands::charset_tools::url_decode_component,
            commands::charset_tools::html_entity_encode,
            commands::charset_tools::html_entity_decode,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        // Graceful shutdown hook. The embedded LLM engine owns a background
        // thread holding a `LlamaBackend` and, once a chat has run, a
        // `LlamaModel` + `LlamaContext` whose weights live in Metal buffers.
        // Those buffers are registered in ggml's per-device residency-set
        // collection, and ggml frees that collection from a C++ static
        // destructor at process exit. If the model has not been freed by
        // then, the destructor trips
        // `GGML_ASSERT([rsets->data count] == 0)` and `abort()`s the app —
        // the `Abort trap: 6` (SIGABRT via `__cxa_finalize_ranges`) seen in
        // the crash dumps under `~/Library/Logs/DiagnosticReports/tbox-*.ips`.
        //
        // So we ask the engine to free its native resources on its own
        // thread and join it here, before Tauri returns and the static
        // destructors run. `RunEvent::Exit` fires once the event loop
        // unwinds — including Cmd+Q and Quit-from-Dock on macOS.
        .run(|_app, event| {
            if let tauri::RunEvent::Exit = event {
                agent::embedded_engine::engine().shutdown_blocking();
            }
        });
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
