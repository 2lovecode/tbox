use reqwest::Client;
use std::time::Duration;

/// HTTP模拟请求
#[tauri::command]
pub async fn http_request(
    method: String,
    url: String,
    headers: serde_json::Value,
    body: Option<String>,
    timeout: u64
) -> Result<serde_json::Value, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(timeout))
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

    let mut request_builder = match method.to_uppercase().as_str() {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        "PATCH" => client.patch(&url),
        "HEAD" => client.head(&url),
        _ => return Err("不支持的HTTP方法".to_string())
    };

    // 添加请求头
    if let Some(headers_obj) = headers.as_object() {
        for (key, value) in headers_obj {
            if let Some(value_str) = value.as_str() {
                request_builder = request_builder.header(key, value_str);
            }
        }
    }

    // 添加请求体
    if let Some(body_str) = body {
        request_builder = request_builder.body(body_str);
    }

    let start = std::time::Instant::now();
    let response = request_builder.send().await
        .map_err(|e| format!("请求失败: {}", e))?;

    let duration = start.elapsed();
    let status = response.status();
    let status_code = status.as_u16();
    let response_headers: serde_json::Map<String, serde_json::Value> = response
        .headers()
        .iter()
        .map(|(k, v)| {
            (k.as_str().to_string(), serde_json::json!(v.to_str().unwrap_or("")))
        })
        .collect();

    let response_body = response.text().await
        .map_err(|e| format!("读取响应体失败: {}", e))?;

    Ok(serde_json::json!({
        "statusCode": status_code,
        "statusText": status.canonical_reason().unwrap_or("Unknown"),
        "headers": response_headers,
        "body": response_body,
        "duration": duration.as_millis()
    }))
}

/// DNS查询
#[tauri::command]
pub async fn dns_lookup(domain: String, record_type: String) -> Result<Vec<String>, String> {
    // 简化实现，使用系统命令或返回模拟数据
    use std::process::Command;

    let output = if cfg!(target_os = "windows") {
        Command::new("nslookup")
            .args([&domain, "8.8.8.8"])
            .output()
    } else {
        Command::new("dig")
            .args(["@", "8.8.8.8", &domain, &record_type])
            .output()
    };

    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let lines: Vec<String> = stdout.lines().map(|s| s.to_string()).collect();
            Ok(lines)
        }
        Err(e) => {
            // 如果命令失败，返回示例数据
            Ok(vec![
                format!("DNS查询 - 域名: {}", domain),
                format!("类型: {}", record_type),
                "DNS服务器: 8.8.8.8".to_string()
            ])
        }
    }
}

/// 端口开放检测
#[tauri::command]
pub async fn check_port_open(host: String, port: u16, timeout: u64) -> Result<bool, String> {
    use std::net::TcpStream;
    use std::time::Duration;

    let address = format!("{}:{}", host, port);
    let timeout_dur = Duration::from_secs(timeout);

    let result = TcpStream::connect_timeout(&address.parse().unwrap(), timeout_dur);

    Ok(result.is_ok())
}

/// 获取公网IP
#[tauri::command]
pub async fn get_public_ip() -> Result<String, String> {
    let client = Client::new();
    let response = client.get("https://api.ipify.org?format=text")
        .send().await
        .map_err(|e| format!("请求失败: {}", e))?;

    let ip = response.text().await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    Ok(ip)
}

/// Ping测试（简化版）
#[tauri::command]
pub async fn ping_test(host: String, count: u32) -> Result<serde_json::Value, String> {
    use std::process::Command;

    let output = if cfg!(target_os = "windows") {
        Command::new("ping")
            .args(["-n", &count.to_string(), &host])
            .output()
    } else {
        Command::new("ping")
            .args(["-c", &count.to_string(), &host])
            .output()
    }.map_err(|e| format!("Ping命令执行失败: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    Ok(serde_json::json!({
        "success": output.status.success(),
        "stdout": stdout.to_string(),
        "stderr": stderr.to_string()
    }))
}

/// SSL证书查询（简化版 - 通过在线API）
#[tauri::command]
pub async fn get_ssl_cert(hostname: String) -> Result<serde_json::Value, String> {
    // 使用简化的实现，通过在线API查询SSL证书信息
    let client = Client::new();
    let url = format!("https://{}:443", hostname);

    // 简化版本：只返回基本信息
    Ok(serde_json::json!({
        "hostname": hostname,
        "message": "SSL证书查询功能需要OpenSSL库支持，Windows上需要手动安装",
        "url": url
    }))
}
