//! Tauri library entry point.
//!
//! Per the Tauri 2.x convention, all builder configuration lives here and
//! `main.rs` simply calls [`run`]. This keeps the entry point tiny and lets
//! the same setup be reused by the mobile entry point and integration tests.

use tauri::Emitter;
use tauri_plugin_global_shortcut::ShortcutState;

mod commands;
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
        .setup(|app| {
            use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

            // Default shortcuts registered at startup. Ctrl+Space works on
            // every platform; on macOS we also try Cmd+Shift+Space because
            // Cmd+Space is already claimed by the system Spotlight. The
            // plugin logs and skips any shortcut that fails to register
            // (e.g. due to OS conflicts).
            let defaults: Vec<Shortcut> = if cfg!(target_os = "macos") {
                vec![
                    Shortcut::new(Some(Modifiers::CONTROL), Code::Space),
                    Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::Space),
                ]
            } else {
                vec![Shortcut::new(Some(Modifiers::CONTROL), Code::Space)]
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
            // 角色管理
            commands::role::get_roles,
            commands::role::get_tools_by_role,
            commands::role::set_user_role,
            commands::role::get_user_role,
            // 搜索
            commands::search::search_tools,

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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
