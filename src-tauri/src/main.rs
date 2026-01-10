// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod utils2;

fn main() {
    if let Err(e) = commands::tool::init_db_if_needed() {
        eprintln!("初始化数据库失败: {}", e);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // 工具管理
            commands::tool::get_categories,
            commands::tool::get_all_tools,

            // 文件操作
            commands::file::download_file,
            commands::file_ops::list_directory,
            commands::file_ops::get_file_size,
            commands::file_ops::file_exists,

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

            // 数据转换工具
            commands::data_convert::json_to_java,
            commands::data_convert::json_to_csharp,
            commands::data_convert::json_to_go,
            commands::data_convert::json_to_python,
            commands::data_convert::json_to_typescript,

            // JSON对比工具
            commands::json_diff::compare_json,
            commands::json_diff::format_json_diff,
            commands::json_diff::query_json_path,

            // 加密工具
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
            commands::encoding::binary_to_hex,
            commands::encoding::hex_to_binary,

            // 文本工具
            commands::text_utils::regex_test,
            commands::text_utils::regex_replace,
            commands::text_utils::text_compare,
            commands::text_utils::text_deduplicate,
            commands::text_utils::text_sort,
            commands::text_utils::text_reverse,
            commands::text_utils::text_statistics,
            commands::text_utils::convert_naming,
            commands::text_utils::convert_case,

            // 时间工具
            commands::datetime::timestamp_to_datetime,
            commands::datetime::datetime_to_timestamp,
            commands::datetime::current_timestamp,
            commands::datetime::batch_timestamp_convert,
            commands::datetime::parse_cron,
            commands::datetime::date_calculation,
            commands::datetime::date_diff,
            commands::datetime::format_duration,

            // 网络工具
            commands::network::http_request,
            commands::network::dns_lookup,
            commands::network::check_port_open,
            commands::network::get_public_ip,
            commands::network::ping_test,
            commands::network::get_ssl_cert,

            // 系统工具
            commands::system::check_port_usage,
            commands::system::kill_process,
            commands::system::list_processes,
            commands::system::get_system_info,
            commands::system::find_files,
            commands::system::calculate_file_hash,

            // XML工具
            commands::xml_utils::format_xml,
            commands::xml_utils::minify_xml,
            commands::xml_utils::xml_to_json,
            commands::xml_utils::json_to_xml,
            commands::xml_utils::xml_to_yaml,
            commands::xml_utils::yaml_to_xml,
            commands::xml_utils::xpath_query,

            // YAML工具
            commands::yaml_utils::format_yaml,
            commands::yaml_utils::yaml_to_json,
            commands::yaml_utils::json_to_yaml,
            commands::yaml_utils::validate_yaml,
            commands::yaml_utils::merge_yaml,

            // 国密算法
            commands::gm_crypto::sm3_hash,
            commands::gm_crypto::sm4_encrypt,
            commands::gm_crypto::sm4_decrypt,
            commands::gm_crypto::generate_sm2_keypair,
            commands::gm_crypto::sm2_sign,
            commands::gm_crypto::sm2_verify,
            commands::gm_crypto::hmac_sm3,
            commands::gm_crypto::generate_sm4_key,

            // SQL工具
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

            // 图片工具
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

            // CSV工具
            commands::csv_utils::csv_to_json,
            commands::csv_utils::json_to_csv,
            commands::csv_utils::format_csv,
            commands::csv_utils::csv_to_excel,
            commands::csv_utils::csv_stats,

            // 日志分析工具
            commands::log_analyzer::analyze_logs,
            commands::log_analyzer::extract_log_levels,
            commands::log_analyzer::filter_logs,
            commands::log_analyzer::count_errors,
            commands::log_analyzer::extract_logs_by_time,
            commands::log_analyzer::highlight_logs,
            commands::log_analyzer::find_duplicate_logs,
            commands::log_analyzer::generate_log_report,

            // 颜色工具
            commands::color_tools::rgb_to_hex,
            commands::color_tools::hex_to_rgb,
            commands::color_tools::rgb_to_hsl,
            commands::color_tools::hsl_to_rgb,
            commands::color_tools::rgb_to_hsv,
            commands::color_tools::hsv_to_rgb,
            commands::color_tools::parse_color,
            commands::color_tools::random_color,
            commands::color_tools::adjust_brightness,

            // 二维码工具
            commands::qrcode_tools::generate_qrcode,
            commands::qrcode_tools::parse_qrcode,
            commands::qrcode_tools::generate_barcode,

            // UUID工具
            commands::uuid_tools::generate_uuid_v4,
            commands::uuid_tools::generate_uuid_v7,
            commands::uuid_tools::generate_uuid_batch,
            commands::uuid_tools::validate_uuid,
            commands::uuid_tools::uuid_to_base64,
            commands::uuid_tools::base64_to_uuid,
            commands::uuid_tools::get_uuid_version,
            commands::uuid_tools::nil_uuid,
            commands::uuid_tools::generate_uuid_v5,

            // Cron工具
            commands::cron_tools::generate_cron,
            commands::cron_tools::get_next_cron_time,
            commands::cron_tools::cron_to_natural_language,
            commands::cron_tools::validate_cron,

            // 数值转换工具
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

            // 字符集工具
            commands::charset_tools::detect_encoding,
            commands::charset_tools::convert_encoding,
            commands::charset_tools::url_encode_component,
            commands::charset_tools::url_decode_component,
            commands::charset_tools::html_entity_encode,
            commands::charset_tools::html_entity_decode,
            commands::charset_tools::punycode_encode,
            commands::charset_tools::punycode_decode,
        ]
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
