// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod utils2;

fn main() {
    commands::tool::init_db_if_needed().unwrap();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::tool::get_categories, 
            commands::tool::get_all_tools,
            commands::file::download_file,
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
            // 文件操作
            commands::file_ops::list_directory,
            commands::file_ops::get_file_size,
            commands::file_ops::file_exists,
        ]
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
