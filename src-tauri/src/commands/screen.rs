use serde::Serialize;

#[derive(Serialize)]
pub struct MousePosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize)]
pub struct WindowInfo {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize)]
pub struct GlobalMouseResult {
    pub screen_x: i32,
    pub screen_y: i32,
    pub window_x: i32,
    pub window_y: i32,
    pub relative_x: i32,
    pub relative_y: i32,
}

#[tauri::command]
pub fn get_window_info(window: tauri::Window) -> Result<WindowInfo, String> {
    let position = window
        .outer_position()
        .map_err(|e| format!("获取窗口位置失败: {}", e))?;
    let size = window
        .outer_size()
        .map_err(|e| format!("获取窗口尺寸失败: {}", e))?;

    Ok(WindowInfo {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
    })
}

#[tauri::command]
pub fn get_global_mouse_position(window: tauri::Window) -> Result<GlobalMouseResult, String> {
    // 获取窗口在屏幕上的位置
    let window_pos = window
        .outer_position()
        .map_err(|e| format!("获取窗口位置失败: {}", e))?;

    // 获取鼠标相对于窗口的位置
    let cursor_pos = window
        .cursor_position()
        .map_err(|e| format!("获取鼠标位置失败: {}", e))?;

    // 计算鼠标在屏幕上的绝对位置
    let screen_x = window_pos.x + cursor_pos.x as i32;
    let screen_y = window_pos.y + cursor_pos.y as i32;

    Ok(GlobalMouseResult {
        screen_x,
        screen_y,
        window_x: window_pos.x,
        window_y: window_pos.y,
        relative_x: cursor_pos.x as i32,
        relative_y: cursor_pos.y as i32,
    })
}

#[tauri::command]
pub fn calculate_global_position(
    window: tauri::Window,
    client_x: i32,
    client_y: i32,
) -> Result<MousePosition, String> {
    // 获取窗口在屏幕上的位置
    let window_pos = window
        .outer_position()
        .map_err(|e| format!("获取窗口位置失败: {}", e))?;

    // 计算鼠标在屏幕上的绝对位置
    Ok(MousePosition {
        x: window_pos.x + client_x,
        y: window_pos.y + client_y,
    })
}
