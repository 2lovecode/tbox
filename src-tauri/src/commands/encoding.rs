
// ==================== URL编码/解码 ====================

#[tauri::command]
pub fn url_encode(input: String) -> Result<String, String> {
    Ok(percent_encoding::utf8_percent_encode(&input, percent_encoding::NON_ALPHANUMERIC).to_string())
}

#[tauri::command]
pub fn url_decode(input: String) -> Result<String, String> {
    percent_encoding::percent_decode_str(&input)
        .decode_utf8()
        .map_err(|e| format!("URL解码失败: {}", e))
        .map(|s| s.to_string())
}

// ==================== Unicode转换 ====================

#[tauri::command]
pub fn unicode_to_chinese(input: String) -> Result<String, String> {
    // 处理 \uXXXX 或 \u{XXXX} 格式的Unicode转义
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if chars.peek() == Some(&'u') {
                chars.next(); // consume 'u'

                // 检查是否是 \u{XXXX} 格式（不定长度）
                if chars.peek() == Some(&'{') {
                    chars.next(); // consume '{'
                    let mut hex = String::new();
                    while let Some(&c) = chars.peek() {
                        if c == '}' {
                            chars.next();
                            break;
                        }
                        hex.push(c);
                        chars.next();
                    }
                    if let Ok(code_point) = u32::from_str_radix(&hex, 16) {
                        if let Some(ch) = char::from_u32(code_point) {
                            result.push(ch);
                        } else {
                            return Err(format!("无效的Unicode码点: \\u{{{}}}", hex));
                        }
                    } else {
                        return Err(format!("无效的十六进制数: {}", hex));
                    }
                } else {
                    // \uXXXX 格式（4位十六进制）
                    let mut hex = String::new();
                    for _ in 0..4 {
                        if let Some(c) = chars.next() {
                            hex.push(c);
                        } else {
                            return Err("不完整的Unicode转义序列".to_string());
                        }
                    }
                    if let Ok(code_point) = u32::from_str_radix(&hex, 16) {
                        if let Some(ch) = char::from_u32(code_point) {
                            result.push(ch);
                        } else {
                            return Err(format!("无效的Unicode码点: \\u{}", hex));
                        }
                    } else {
                        return Err(format!("无效的十六进制数: {}", hex));
                    }
                }
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }

    Ok(result)
}

#[tauri::command]
pub fn chinese_to_unicode(input: String) -> Result<String, String> {
    let mut result = String::new();
    for c in input.chars() {
        if c.is_ascii() {
            result.push(c);
        } else {
            // 非ASCII字符转为 \uXXXX 格式
            result.push_str(&format!("\\u{:04x}", c as u32));
        }
    }
    Ok(result)
}

// ==================== Base58 编解码 ====================

const BASE58_ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

#[tauri::command]
pub fn base58_encode(input: String) -> Result<String, String> {
    // 将输入字符串转换为字节
    let bytes = input.as_bytes();

    // 计算结果字符串的长度
    let leading_zeros = bytes.iter().take_while(|&&b| b == 0).count();
    let size = (bytes.len() - leading_zeros) * 138 / 100 + 1;
    let mut buffer = vec![0u8; size];
    let high = buffer.len() - 1;

    // 处理每个字节
    for &byte in bytes {
        let mut carry = byte as i32;
        let mut i = high;

        loop {
            carry += 256 * buffer[i] as i32;
            buffer[i] = (carry % 58) as u8;
            carry /= 58;

            if carry == 0 && i == 0 && byte == 0 {
                break;
            }
            if i == 0 {
                break;
            }
            i -= 1;
        }
    }

    // 找到非零部分的起始位置
    let mut result = String::new();
    let start = buffer.iter().position(|&b| b != 0).unwrap_or(buffer.len());

    // 添加前导的 '1'（对应前导零字节）
    for _ in 0..leading_zeros {
        result.push('1');
    }

    // 转换剩余部分
    for &b in &buffer[start..] {
        result.push(BASE58_ALPHABET[b as usize] as char);
    }

    Ok(result)
}

#[tauri::command]
pub fn base58_decode(input: String) -> Result<String, String> {
    // 检查输入是否为空
    if input.is_empty() {
        return Err("输入不能为空".to_string());
    }

    let bytes = input.as_bytes();
    let leading_ones = bytes.iter().take_while(|&&b| b == b'1').count();
    let size = (bytes.len() - leading_ones) * 138 / 100 + 1;
    let mut buffer = vec![0u8; size];
    let high = buffer.len() - 1;

    for &byte in bytes {
        let c = match byte {
            b'1'..=b'9' => byte - b'1',
            b'A'..=b'H' => byte - b'A' + 9,
            b'J'..=b'N' => byte - b'J' + 17,
            b'P'..=b'Z' => byte - b'P' + 22,
            b'a'..=b'k' => byte - b'a' + 25,
            b'm'..=b'z' => byte - b'm' + 31,
            _ => return Err(format!("无效的Base58字符: {}", byte as char)),
        };

        let mut carry = c as i32;
        let mut i = high;

        loop {
            carry += 58 * buffer[i] as i32;
            buffer[i] = (carry % 256) as u8;
            carry /= 256;

            if carry == 0 && i == 0 && c == 0 {
                break;
            }
            if i == 0 {
                break;
            }
            i -= 1;
        }
    }

    let mut result = Vec::new();
    let start = buffer.iter().position(|&b| b != 0).unwrap_or(buffer.len());

    for _ in 0..leading_ones {
        result.push(0);
    }

    for &b in &buffer[start..] {
        result.push(b);
    }

    String::from_utf8(result).map_err(|e| format!("解码后不是有效的UTF-8字符串: {}", e))
}

// ==================== Base62 编解码 ====================

const BASE62_ALPHABET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

#[tauri::command]
pub fn base62_encode(input: String) -> Result<String, String> {
    let bytes = input.as_bytes();
    let leading_zeros = bytes.iter().take_while(|&&b| b == 0).count();
    let size = (bytes.len() - leading_zeros) * 146 / 100 + 1;
    let mut buffer = vec![0u8; size];
    let high = buffer.len() - 1;

    for &byte in bytes {
        let mut carry = byte as i32;
        let mut i = high;

        loop {
            carry += 256 * buffer[i] as i32;
            buffer[i] = (carry % 62) as u8;
            carry /= 62;

            if carry == 0 && i == 0 && byte == 0 {
                break;
            }
            if i == 0 {
                break;
            }
            i -= 1;
        }
    }

    let mut result = String::new();
    let start = buffer.iter().position(|&b| b != 0).unwrap_or(buffer.len());

    for _ in 0..leading_zeros {
        result.push('0');
    }

    for &b in &buffer[start..] {
        result.push(BASE62_ALPHABET[b as usize] as char);
    }

    Ok(result)
}

#[tauri::command]
pub fn base62_decode(input: String) -> Result<String, String> {
    if input.is_empty() {
        return Err("输入不能为空".to_string());
    }

    let bytes = input.as_bytes();
    let leading_zeros = bytes.iter().take_while(|&&b| b == b'0').count();
    let size = (bytes.len() - leading_zeros) * 146 / 100 + 1;
    let mut buffer = vec![0u8; size];
    let high = buffer.len() - 1;

    for &byte in bytes {
        let c = match byte {
            b'0'..=b'9' => byte - b'0',
            b'A'..=b'Z' => byte - b'A' + 10,
            b'a'..=b'z' => byte - b'a' + 36,
            _ => return Err(format!("无效的Base62字符: {}", byte as char)),
        };

        let mut carry = c as i32;
        let mut i = high;

        loop {
            carry += 62 * buffer[i] as i32;
            buffer[i] = (carry % 256) as u8;
            carry /= 256;

            if carry == 0 && i == 0 && c == 0 {
                break;
            }
            if i == 0 {
                break;
            }
            i -= 1;
        }
    }

    let mut result = Vec::new();
    let start = buffer.iter().position(|&b| b != 0).unwrap_or(buffer.len());

    for _ in 0..leading_zeros {
        result.push(0);
    }

    for &b in &buffer[start..] {
        result.push(b);
    }

    String::from_utf8(result).map_err(|e| format!("解码后不是有效的UTF-8字符串: {}", e))
}

// ==================== 十六进制转换 ====================

#[tauri::command]
pub fn string_to_hex(input: String) -> Result<String, String> {
    // Single-pass build of "AA BB CC ..." from the input bytes.
    let mut out = String::with_capacity(input.len() * 3);
    for (i, byte) in input.as_bytes().iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        out.push_str(&format!("{:02X}", byte));
    }
    Ok(out)
}

#[tauri::command]
pub fn hex_to_string(hex: String) -> Result<String, String> {
    // 移除空格
    let hex = hex.replace(" ", "").replace("\n", "").replace("\r", "");

    // 验证十六进制格式
    if hex.len() % 2 != 0 {
        return Err("十六进制字符串长度必须是偶数".to_string());
    }

    // 验证字符
    for c in hex.chars() {
        if !c.is_ascii_hexdigit() {
            return Err(format!("无效的十六进制字符: {}", c));
        }
    }

    // 解码
    let bytes = hex::decode(&hex).map_err(|e| format!("十六进制解码失败: {}", e))?;
    String::from_utf8(bytes).map_err(|e| format!("解码后不是有效的UTF-8字符串: {}", e))
}

// ==================== HTML实体编解码 ====================

#[tauri::command]
pub fn html_encode(input: String) -> Result<String, String> {
    let mut result = String::with_capacity(input.len() * 6);
    for c in input.chars() {
        match c {
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '&' => result.push_str("&amp;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#39;"),
            _ => result.push(c),
        }
    }
    Ok(result)
}

#[tauri::command]
pub fn html_decode(input: String) -> Result<String, String> {
    // Single-pass scanner. &-sequences are decoded in-place into the output
    // string; everything else is copied byte-for-byte.
    const NAMED: &[(&str, &str)] = &[
        ("&lt;", "<"),
        ("&gt;", ">"),
        ("&amp;", "&"),
        ("&quot;", "\""),
        ("&#39;", "'"),
        ("&nbsp;", " "),
    ];

    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'&' {
            out.push(bytes[i] as char);
            i += 1;
            continue;
        }

        // Try named entities first.
        let mut matched = false;
        for (entity, replacement) in NAMED {
            if bytes[i..].starts_with(entity.as_bytes()) {
                out.push_str(replacement);
                i += entity.len();
                matched = true;
                break;
            }
        }
        if matched {
            continue;
        }

        // Numeric entities: &#NN; (decimal) or &#xNN; (hex).
        if bytes[i..].starts_with(b"&#") {
            let after_amp = i + 2;
            let (digits_start, radix) = match bytes.get(after_amp) {
                Some(b'x') | Some(b'X') => (after_amp + 1, 16),
                _ => (after_amp, 10),
            };
            if let Some(semi) = bytes[digits_start..].iter().position(|&b| b == b';') {
                let num_end = digits_start + semi;
                if let Ok(code) = u32::from_str_radix(
                    std::str::from_utf8(&bytes[digits_start..num_end]).unwrap_or(""),
                    radix,
                ) {
                    if let Some(ch) = char::from_u32(code) {
                        out.push(ch);
                        i = num_end + 1;
                        continue;
                    }
                }
            }
        }

        // Unknown escape — copy the '&' verbatim and move on.
        out.push('&');
        i += 1;
    }

    Ok(out)
}

// ==================== Punycode 编解码 ====================

#[tauri::command]
pub fn punycode_encode(input: String) -> Result<String, String> {
    punycode::encode(&input).map_err(|_| "Punycode 编码失败".to_string())
}

#[tauri::command]
pub fn punycode_decode(input: String) -> Result<String, String> {
    punycode::decode(&input).map_err(|_| "Punycode 解码失败".to_string())
}

// ==================== 进制转换辅助 ====================

#[tauri::command]
pub fn binary_to_hex(input: String) -> Result<String, String> {
    // 移除空格
    let binary = input.replace(" ", "").replace("\n", "").replace("\r", "");

    // 验证二进制格式
    for c in binary.chars() {
        if c != '0' && c != '1' {
            return Err(format!("无效的二进制字符: {}", c));
        }
    }

    // 补齐到8的倍数
    let len = binary.len();
    let padded_len = (len + 7) / 8 * 8;
    let padded = format!("{:0>width$}", binary, width = padded_len);

    let mut result = String::new();
    for chunk in padded.as_bytes().chunks(8) {
        let byte = u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 2)
            .map_err(|e| format!("二进制转换失败: {}", e))?;
        result.push_str(&format!("{:02X} ", byte));
    }

    Ok(result.trim().to_string())
}

#[tauri::command]
pub fn hex_to_binary(hex: String) -> Result<String, String> {
    // 移除空格
    let hex = hex.replace(" ", "").replace("\n", "").replace("\r", "");

    // 验证十六进制格式
    if hex.len() % 2 != 0 {
        return Err("十六进制字符串长度必须是偶数".to_string());
    }

    for c in hex.chars() {
        if !c.is_ascii_hexdigit() {
            return Err(format!("无效的十六进制字符: {}", c));
        }
    }

    let bytes = hex::decode(&hex).map_err(|e| format!("十六进制解码失败: {}", e))?;

    let result = bytes
        .iter()
        .map(|b| format!("{:08b}", b))
        .collect::<Vec<_>>()
        .join(" ");

    Ok(result)
}
