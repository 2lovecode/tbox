use uuid::Uuid;
use base64::{Engine as _, engine::general_purpose};

/// 生成UUID v4
#[tauri::command]
pub fn generate_uuid_v4() -> String {
    Uuid::new_v4().to_string()
}

/// 生成UUID v7
#[tauri::command]
pub fn generate_uuid_v7() -> String {
    // 使用now()创建时间戳的UUID v7
    Uuid::now_v7().to_string()
}

/// 批量生成UUID
#[tauri::command]
pub fn generate_uuid_batch(count: u32, version: String) -> Vec<String> {
    (0..count)
        .map(|_| {
            match version.to_lowercase().as_str() {
                "v4" => Uuid::new_v4().to_string(),
                "v7" => Uuid::now_v7().to_string(),
                _ => Uuid::new_v4().to_string()
            }
        })
        .collect()
}

/// 验证UUID
#[tauri::command]
pub fn validate_uuid(uuid_str: String) -> bool {
    Uuid::parse_str(&uuid_str).is_ok()
}

/// UUID转Base64
#[tauri::command]
pub fn uuid_to_base64(uuid_str: String) -> Result<String, String> {
    let uuid = Uuid::parse_str(&uuid_str)
        .map_err(|_| "无效的UUID".to_string())?;

    Ok(general_purpose::STANDARD.encode(uuid.as_bytes()))
}

/// Base64转UUID
#[tauri::command]
pub fn base64_to_uuid(base64_str: String) -> Result<String, String> {
    let bytes = general_purpose::STANDARD.decode(&base64_str)
        .map_err(|_| "无效的Base64".to_string())?;

    let uuid = Uuid::from_slice(&bytes)
        .map_err(|_| "Base64数据不是有效的UUID".to_string())?;

    Ok(uuid.to_string())
}

/// 获取UUID版本
#[tauri::command]
pub fn get_uuid_version(uuid_str: String) -> Result<u8, String> {
    let uuid = Uuid::parse_str(&uuid_str)
        .map_err(|_| "无效的UUID".to_string())?;

    match uuid.get_version() {
        Some(version) => Ok(version as u8),
        None => Ok(0)
    }
}

/// 生成NIL UUID
#[tauri::command]
pub fn nil_uuid() -> String {
    Uuid::nil().to_string()
}

/// 生成命名UUID v5
#[tauri::command]
pub fn generate_uuid_v5(namespace: String, name: String) -> Result<String, String> {
    let ns_uuid = Uuid::parse_str(&namespace)
        .map_err(|_| "无效的命名空间UUID".to_string())?;

    let uuid = Uuid::new_v5(&ns_uuid, name.as_bytes());

    Ok(uuid.to_string())
}
