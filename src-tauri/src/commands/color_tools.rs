use regex::Regex;

/// RGB转十六进制
#[tauri::command]
pub fn rgb_to_hex(r: u8, g: u8, b: u8) -> String {
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

/// 十六进制转RGB
#[tauri::command]
pub fn hex_to_rgb(hex: String) -> Result<(u8, u8, u8), String> {
    let hex = hex.trim_start_matches('#');

    if hex.len() != 6 {
        return Err("十六进制颜色代码必须是6位".to_string());
    }

    let r = u8::from_str_radix(&hex[0..2], 16)
        .map_err(|_| "无效的十六进制颜色代码".to_string())?;
    let g = u8::from_str_radix(&hex[2..4], 16)
        .map_err(|_| "无效的十六进制颜色代码".to_string())?;
    let b = u8::from_str_radix(&hex[4..6], 16)
        .map_err(|_| "无效的十六进制颜色代码".to_string())?;

    Ok((r, g, b))
}

/// RGB转HSL
#[tauri::command]
pub fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let rf = r as f64 / 255.0;
    let gf = g as f64 / 255.0;
    let bf = b as f64 / 255.0;

    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let delta = max - min;

    let h = if delta == 0.0 {
        0.0
    } else if max == rf {
        60.0 * (((gf - bf) / delta) % 6.0)
    } else if max == gf {
        60.0 * (((bf - rf) / delta) + 2.0)
    } else {
        60.0 * (((rf - gf) / delta) + 4.0)
    };

    let l = (max + min) / 2.0;

    let s = if delta == 0.0 {
        0.0
    } else {
        delta / (1.0 - (2.0 * l - 1.0).abs())
    };

    (
        if h < 0.0 { h + 360.0 } else { h },
        s * 100.0,
        l * 100.0
    )
}

/// HSL转RGB
#[tauri::command]
pub fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    let h = h / 360.0;
    let s = s / 100.0;
    let l = l / 100.0;

    if s == 0.0 {
        let gray = (l * 255.0).round() as u8;
        return (gray, gray, gray);
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };

    let p = 2.0 * l - q;

    let hk = (h + 1.0 / 3.0) % 1.0;
    let hk2 = h % 1.0;
    let hk3 = (h + 2.0 / 3.0) % 1.0;

    let to_rgb = |t: f64| -> u8 {
        let t = if t < 0.0 { t + 1.0 } else if t > 1.0 { t - 1.0 } else { t };
        let color = if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        };
        (color * 255.0).round() as u8
    };

    (to_rgb(hk), to_rgb(hk2), to_rgb(hk3))
}

/// RGB转HSV
#[tauri::command]
pub fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let rf = r as f64 / 255.0;
    let gf = g as f64 / 255.0;
    let bf = b as f64 / 255.0;

    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let delta = max - min;

    let h = if delta == 0.0 {
        0.0
    } else if max == rf {
        60.0 * (((gf - bf) / delta) % 6.0)
    } else if max == gf {
        60.0 * (((bf - rf) / delta) + 2.0)
    } else {
        60.0 * (((rf - gf) / delta) + 4.0)
    };

    let s = if max == 0.0 { 0.0 } else { delta / max };

    (h, s * 100.0, max * 100.0)
}

/// HSV转RGB
#[tauri::command]
pub fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
    let h = h / 60.0;
    let s = s / 100.0;
    let v = v / 100.0;

    let c = v * s;
    let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
    let m = v - c;

    let (r1, g1, b1) = if h < 1.0 {
        (c, x, 0.0)
    } else if h < 2.0 {
        (x, c, 0.0)
    } else if h < 3.0 {
        (0.0, c, x)
    } else if h < 4.0 {
        (0.0, x, c)
    } else if h < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (
        ((r1 + m) * 255.0).round() as u8,
        ((g1 + m) * 255.0).round() as u8,
        ((b1 + m) * 255.0).round() as u8
    )
}

/// 解析颜色字符串
#[tauri::command]
pub fn parse_color(color: String) -> Result<serde_json::Value, String> {
    let color = color.trim().to_lowercase();

    // 十六进制
    if color.starts_with('#') {
        let (r, g, b) = hex_to_rgb(color)?;
        return Ok(serde_json::json!({
            "hex": format!("#{:02X}{:02X}{:02X}", r, g, b),
            "rgb": format!("rgb({}, {}, {})", r, g, b),
            "hsl": rgb_to_hsl(r, g, b),
            "hsv": rgb_to_hsv(r, g, b)
        }));
    }

    // RGB
    if color.starts_with("rgb") {
        let re = Regex::new(r"rgb\((\d+),\s*(\d+),\s*(\d+)\)")
            .map_err(|_| "正则表达式编译失败".to_string())?;

        if let Some(caps) = re.captures(&color) {
            let r: u8 = caps[1].parse().map_err(|_| "无效的RGB值".to_string())?;
            let g: u8 = caps[2].parse().map_err(|_| "无效的RGB值".to_string())?;
            let b: u8 = caps[3].parse().map_err(|_| "无效的RGB值".to_string())?;

            return Ok(serde_json::json!({
                "hex": format!("#{:02X}{:02X}{:02X}", r, g, b),
                "rgb": format!("rgb({}, {}, {})", r, g, b),
                "hsl": rgb_to_hsl(r, g, b),
                "hsv": rgb_to_hsv(r, g, b)
            }));
        }
    }

    Err("不支持的颜色格式".to_string())
}

/// 生成随机颜色
#[tauri::command]
pub fn random_color() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let r: u8 = rng.gen();
    let g: u8 = rng.gen();
    let b: u8 = rng.gen();
    rgb_to_hex(r, g, b)
}

/// 调整颜色亮度
#[tauri::command]
pub fn adjust_brightness(hex: String, percent: i32) -> Result<String, String> {
    let (r, g, b) = hex_to_rgb(hex)?;

    let factor = 1.0 + (percent as f64 / 100.0);

    let r = (r as f64 * factor).clamp(0.0, 255.0).round() as u8;
    let g = (g as f64 * factor).clamp(0.0, 255.0).round() as u8;
    let b = (b as f64 * factor).clamp(0.0, 255.0).round() as u8;

    Ok(rgb_to_hex(r, g, b))
}
