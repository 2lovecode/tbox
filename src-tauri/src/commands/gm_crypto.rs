use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};

/// SM3哈希（简化实现，使用SHA256）
#[tauri::command]
pub fn sm3_hash(input: String) -> String {
    // 简化实现：使用SHA256代替SM3
    // 实际项目中应使用libsm库的SM3实现
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// SM4加密（简化实现）
#[tauri::command]
pub fn sm4_encrypt(plaintext: String, key: String, _iv: String) -> Result<String, String> {
    // 简化实现：使用简单的XOR加密
    // 实际项目中应使用libsm库的SM4实现
    let mut key_bytes = key.as_bytes().to_vec();
    key_bytes.resize(16, 0);

    let data = plaintext.as_bytes();
    let encrypted: Vec<u8> = data.iter()
        .enumerate()
        .map(|(i, &b)| b ^ key_bytes[i % key_bytes.len()])
        .collect();

    Ok(general_purpose::STANDARD.encode(encrypted))
}

/// SM4解密（简化实现）
#[tauri::command]
pub fn sm4_decrypt(ciphertext: String, key: String, _iv: String) -> Result<String, String> {
    // 简化实现：使用简单的XOR解密
    let mut key_bytes = key.as_bytes().to_vec();
    key_bytes.resize(16, 0);

    let encrypted = general_purpose::STANDARD.decode(&ciphertext)
        .map_err(|e| format!("Base64解码失败: {}", e))?;

    let decrypted: Vec<u8> = encrypted.iter()
        .enumerate()
        .map(|(i, &b)| b ^ key_bytes[i % key_bytes.len()])
        .collect();

    String::from_utf8(decrypted).map_err(|e| format!("UTF-8转换失败: {}", e))
}

/// SM2密钥对生成（简化实现）
#[tauri::command]
pub fn generate_sm2_keypair() -> Result<(String, String), String> {
    // 简化实现：返回示例密钥
    // 实际项目中应使用libsm库的SM2实现
    Ok((
        "SM2私钥已生成（示例）".to_string(),
        "SM2公钥已生成（示例）".to_string()
    ))
}

/// SM2签名（简化实现）
#[tauri::command]
pub fn sm2_sign(message: String, _private_key: String) -> Result<String, String> {
    // 简化实现：返回SM3哈希作为签名
    Ok(sm3_hash(message))
}

/// SM2验签（简化实现）
#[tauri::command]
pub fn sm2_verify(message: String, signature: String, _public_key: String) -> Result<bool, String> {
    // 简化实现：验证签名是否等于SM3哈希
    let expected = sm3_hash(message);
    Ok(signature == expected)
}

/// HMAC-SM3
#[tauri::command]
pub fn hmac_sm3(message: String, key: String) -> String {
    // 简化实现：返回 key + message 的SM3哈希
    let combined = format!("{}{}", key, message);
    sm3_hash(combined)
}

/// SM4生成密钥
#[tauri::command]
pub fn generate_sm4_key() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let key: Vec<u8> = (0..16).map(|_| rng.gen()).collect();
    hex::encode(key)
}
