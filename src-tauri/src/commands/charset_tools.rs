use encoding_rs::GBK;

/// 检测文本编码
#[tauri::command]
pub fn detect_encoding(text: String) -> String {
    let bytes = text.as_bytes();

    // 简单的编码检测逻辑
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return "UTF-8 (with BOM)".to_string();
    }

    if bytes.starts_with(&[0xFF, 0xFE]) {
        return "UTF-16 LE".to_string();
    }

    if bytes.starts_with(&[0xFE, 0xFF]) {
        return "UTF-16 BE".to_string();
    }

    // 尝试UTF-8解码
    if let Ok(_) = std::str::from_utf8(bytes) {
        // 检查是否包含ASCII范围外的字符
        if bytes.iter().all(|&b| b <= 0x7F) {
            return "ASCII / UTF-8".to_string();
        }
        return "UTF-8".to_string();
    }

    // 检测GBK
    let (decoded_text, _, _) = GBK.decode(bytes);
    if decoded_text.len() > 0 || bytes.len() == 0 {
        return "GBK".to_string();
    }

    "Unknown".to_string()
}

/// 转换编码
#[tauri::command]
pub fn convert_encoding(text: String, from: String, to: String) -> Result<String, String> {
    let bytes = text.as_bytes();

    let decoded = match from.to_lowercase().as_str() {
        "utf-8" | "utf8" => {
            std::str::from_utf8(bytes)
                .map_err(|_| "UTF-8解码失败".to_string())?
                .to_string()
        }
        "gbk" => {
            let (decoded_text, _, _) = GBK.decode(bytes);
            decoded_text.to_string()
        }
        _ => return Err(format!("不支持的源编码: {}", from))
    };

    let encoded = match to.to_lowercase().as_str() {
        "utf-8" | "utf8" => decoded,
        "gbk" => {
            let (encoded_bytes, _, _) = GBK.encode(&decoded);
            String::from_utf8_lossy(&encoded_bytes).to_string()
        }
        _ => return Err(format!("不支持的目标编码: {}", to))
    };

    Ok(encoded)
}

/// URL编码
#[tauri::command]
pub fn url_encode_component(input: String) -> String {
    percent_encoding::utf8_percent_encode(&input, percent_encoding::NON_ALPHANUMERIC).to_string()
}

/// URL解码
#[tauri::command]
pub fn url_decode_component(input: String) -> Result<String, String> {
    percent_encoding::percent_decode(input.as_bytes())
        .decode_utf8()
        .map(|s| s.to_string())
        .map_err(|_| "URL解码失败".to_string())
}

/// HTML实体编码
#[tauri::command]
pub fn html_entity_encode(input: String) -> String {
    input.chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            '\'' => "&apos;".to_string(),
            c if c > '\x7F' => format!("&#{};", c as u32),
            _ => c.to_string(),
        })
        .collect()
}

/// HTML实体解码
#[tauri::command]
pub fn html_entity_decode(input: String) -> String {
    let mut result = input;

    // 常见HTML实体
    result = result.replace("&amp;", "&");
    result = result.replace("&lt;", "<");
    result = result.replace("&gt;", ">");
    result = result.replace("&quot;", "\"");
    result = result.replace("&apos;", "'");

    // 数字实体 &#xxx;
    let re = regex::Regex::new(r"&#(\d+);").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        if let Ok(code_point) = caps[1].parse::<u32>() {
            if let Some(c) = char::from_u32(code_point) {
                return c.to_string();
            }
        }
        caps[0].to_string()
    }).to_string();

    result
}

