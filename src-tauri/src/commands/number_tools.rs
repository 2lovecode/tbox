use num_bigint::BigUint;
use num_traits::Num;

/// 十进制转十六进制
#[tauri::command]
pub fn dec_to_hex(num: String) -> String {
    if let Ok(n) = num.parse::<u128>() {
        format!("{:X}", n)
    } else if let Ok(n) = BigUint::from_str_radix(&num, 10) {
        format!("{:X}", n)
    } else {
        "无效的数字".to_string()
    }
}

/// 十六进制转十进制
#[tauri::command]
pub fn hex_to_dec(hex: String) -> String {
    if let Ok(n) = u128::from_str_radix(&hex.trim_start_matches("0x"), 16) {
        n.to_string()
    } else if let Ok(n) = BigUint::from_str_radix(&hex.trim_start_matches("0x"), 16) {
        n.to_string()
    } else {
        "无效的十六进制".to_string()
    }
}

/// 十进制转二进制
#[tauri::command]
pub fn dec_to_binary(num: String) -> String {
    if let Ok(n) = num.parse::<u128>() {
        format!("{:b}", n)
    } else if let Ok(n) = BigUint::from_str_radix(&num, 10) {
        format!("{:b}", n)
    } else {
        "无效的数字".to_string()
    }
}

/// 二进制转十进制
#[tauri::command]
pub fn binary_to_dec(binary: String) -> String {
    let binary = binary.trim_start_matches("0b");

    if let Ok(n) = u128::from_str_radix(binary, 2) {
        n.to_string()
    } else if let Ok(n) = BigUint::from_str_radix(binary, 2) {
        n.to_string()
    } else {
        "无效的二进制".to_string()
    }
}

/// 十进制转八进制
#[tauri::command]
pub fn dec_to_octal(num: String) -> String {
    if let Ok(n) = num.parse::<u128>() {
        format!("{:o}", n)
    } else if let Ok(n) = BigUint::from_str_radix(&num, 10) {
        format!("{:o}", n)
    } else {
        "无效的数字".to_string()
    }
}

/// 八进制转十进制
#[tauri::command]
pub fn octal_to_dec(octal: String) -> String {
    let octal = octal.trim_start_matches("0o");

    if let Ok(n) = u128::from_str_radix(octal, 8) {
        n.to_string()
    } else if let Ok(n) = BigUint::from_str_radix(octal, 8) {
        n.to_string()
    } else {
        "无效的八进制".to_string()
    }
}

/// 科学计数法转换
#[tauri::command]
pub fn scientific_to_decimal(num: String) -> Result<String, String> {
    if let Ok(n) = num.parse::<f64>() {
        Ok(n.to_string())
    } else {
        Err("无效的科学计数法数字".to_string())
    }
}

/// 数字转罗马数字
#[tauri::command]
pub fn to_roman(num: u32) -> Result<String, String> {
    if num > 3999 {
        return Err("数字超出范围（最大3999）".to_string());
    }

    let values = vec![
        (1000, "M"), (900, "CM"), (500, "D"), (400, "CD"),
        (100, "C"), (90, "XC"), (50, "L"), (40, "XL"),
        (10, "X"), (9, "IX"), (5, "V"), (4, "IV"), (1, "I")
    ];

    let mut result = String::new();
    let mut num = num;

    for (value, symbol) in values {
        while num >= value {
            result.push_str(symbol);
            num -= value;
        }
    }

    Ok(result)
}

/// 罗马数字转数字
#[tauri::command]
pub fn from_roman(roman: String) -> Result<u32, String> {
    let roman = roman.to_uppercase();
    let values: std::collections::HashMap<char, u32> = [
        ('I', 1), ('V', 5), ('X', 10), ('L', 50),
        ('C', 100), ('D', 500), ('M', 1000)
    ].iter().cloned().collect();

    let mut result = 0;
    let mut prev_value = 0;

    for c in roman.chars().rev() {
        let value = *values.get(&c).ok_or_else(|| format!("无效的罗马数字字符: {}", c))?;

        if value < prev_value {
            result -= value;
        } else {
            result += value;
        }

        prev_value = value;
    }

    Ok(result)
}

/// 分数转小数
#[tauri::command]
pub fn fraction_to_decimal(numerator: i64, denominator: i64) -> Result<String, String> {
    if denominator == 0 {
        return Err("分母不能为零".to_string());
    }

    let result = numerator as f64 / denominator as f64;
    Ok(result.to_string())
}

/// 小数转分数（简化实现）
#[tauri::command]
pub fn decimal_to_fraction(decimal: f64) -> String {
    let tolerance = 1.0e-9;
    let mut numerator = 1;
    let mut denominator = 1;

    while ((numerator as f64) / (denominator as f64) - decimal).abs() > tolerance {
        if (numerator as f64) / (denominator as f64) < decimal {
            numerator += 1;
        } else {
            denominator += 1;
        }
    }

    format!("{}/{}", numerator, denominator)
}
