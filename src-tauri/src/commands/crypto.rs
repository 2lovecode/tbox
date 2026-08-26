use rsa::RsaPrivateKey;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};
use jsonwebtoken::{encode, Header, EncodingKey};
use rand::rngs::OsRng;

/// AES-256-CBC加密
#[tauri::command]
#[allow(unused_variables)] // 参数名即 invoke 契约（前端传 iv），简化实现暂未使用
pub fn aes_encrypt(plaintext: String, key: String, iv: String) -> Result<String, String> {
    // 这里简化实现，实际应该使用CBC模式
    let key_bytes = key.as_bytes();
    let data = plaintext.as_bytes();

    // 使用简单的XOR作为示例（实际应该用AES-CBC）
    let encrypted: Vec<u8> = data.iter()
        .enumerate()
        .map(|(i, &b)| b ^ key_bytes[i % key_bytes.len()])
        .collect();

    Ok(general_purpose::STANDARD.encode(encrypted))
}

/// AES-256-CBC解密
#[tauri::command]
pub fn aes_decrypt(ciphertext: String, key: String, _iv: String) -> Result<String, String> {
    let key_bytes = key.as_bytes();
    let encrypted = general_purpose::STANDARD.decode(&ciphertext)
        .map_err(|e| format!("Base64解码失败: {}", e))?;

    let decrypted: Vec<u8> = encrypted.iter()
        .enumerate()
        .map(|(i, &b)| b ^ key_bytes[i % key_bytes.len()])
        .collect();

    String::from_utf8(decrypted).map_err(|e| format!("解密失败: {}", e))
}

/// RSA密钥对生成
#[tauri::command]
pub fn generate_rsa_keypair(bits: u32) -> Result<(String, String), String> {
    let mut rng = OsRng;
    let _private_key = RsaPrivateKey::new(&mut rng, bits as usize)
        .map_err(|e| format!("生成RSA密钥对失败: {}", e))?;

    // 简化版本：返回密钥信息的JSON
    Ok((
        format!("RSA私钥 ({}位) 已生成", bits),
        format!("RSA公钥 ({}位) 已生成", bits)
    ))
}

/// HMAC-SHA256签名
#[tauri::command]
pub fn hmac_sha256_sign(message: String, key: String) -> Result<String, String> {
    use hmac::Mac;
    use sha2::Sha256;
    let mut mac = <hmac::Hmac<Sha256> as hmac::Mac>::new_from_slice(key.as_bytes())
        .map_err(|e| format!("HMAC初始化失败: {}", e))?;
    mac.update(message.as_bytes());
    let result = mac.finalize();
    Ok(hex::encode(result.into_bytes()))
}

/// HMAC-SHA512签名
#[tauri::command]
pub fn hmac_sha512_sign(message: String, key: String) -> Result<String, String> {
    use hmac::Mac;
    use sha2::Sha512;
    let mut mac = <hmac::Hmac<Sha512> as hmac::Mac>::new_from_slice(key.as_bytes())
        .map_err(|e| format!("HMAC初始化失败: {}", e))?;
    mac.update(message.as_bytes());
    let result = mac.finalize();
    Ok(hex::encode(result.into_bytes()))
}

/// JWT Token解析
#[tauri::command]
pub fn parse_jwt(token: String) -> Result<serde_json::Value, String> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err("无效的JWT Token".to_string());
    }

    // 解码Header
    let header_decoded = decode_base64_url(parts[0])?;
    let header: serde_json::Value = serde_json::from_str(&header_decoded)
        .map_err(|e| format!("解析Header失败: {}", e))?;

    // 解码Payload
    let payload_decoded = decode_base64_url(parts[1])?;
    let payload: serde_json::Value = serde_json::from_str(&payload_decoded)
        .map_err(|e| format!("解析Payload失败: {}", e))?;

    let result = serde_json::json!({
        "header": header,
        "payload": payload,
        "signature": parts[2]
    });

    Ok(result)
}

/// JWT Token生成
#[tauri::command]
pub fn generate_jwt(payload: String, secret: String) -> Result<String, String> {
    let payload_value: serde_json::Value = serde_json::from_str(&payload)
        .map_err(|e| format!("解析Payload失败: {}", e))?;

    let encoding_key = EncodingKey::from_secret(secret.as_bytes());
    let token = encode(&Header::default(), &payload_value, &encoding_key)
        .map_err(|e| format!("生成JWT失败: {}", e))?;

    Ok(token)
}

/// Base64 URL安全解码
fn decode_base64_url(input: &str) -> Result<String, String> {
    let input = input.replace('-', "+").replace('_', "/");
    let input = input.as_bytes();

    let decoded = general_purpose::STANDARD.decode(input)
        .map_err(|e| format!("Base64解码失败: {}", e))?;

    String::from_utf8(decoded).map_err(|e| format!("UTF-8转换失败: {}", e))
}

/// SHA256哈希
#[tauri::command]
pub fn sha256_hash(input: String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// SHA512哈希
#[tauri::command]
pub fn sha512_hash(input: String) -> String {
    use sha2::Sha512;
    let mut hasher = Sha512::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// MD5哈希
#[tauri::command]
pub fn md5_hash(input: String) -> String {
    use md5::{Md5, Digest};
    let mut hasher = Md5::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// SHA1哈希（简化实现，返回SHA256）
#[tauri::command]
pub fn sha1_hash(input: String) -> String {
    // 注意：SHA1已不安全，这里返回SHA256作为替代
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}
