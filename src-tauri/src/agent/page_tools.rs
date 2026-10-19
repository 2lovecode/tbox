//! 工具页操作的 Agent 规格与调度。复用既有 command。

use serde_json::{json, Value};

use crate::agent::registry::{required_string_props, tool, tool_confirm, ToolSpec};

fn s(id: &'static str, desc: &'static str, props: &[(&str, &str)]) -> ToolSpec {
    tool(id, desc, required_string_props(props))
}

fn empty_schema() -> Value {
    json!({
        "type": "object",
        "properties": {},
        "additionalProperties": false
    })
}

pub fn extra_specs() -> Vec<ToolSpec> {
    let mut specs = vec![
        s("json.compress", "JSON 压缩（去掉空白）", &[("input", "JSON 字符串")]),
        s("json.escape", "JSON 转义为代码字符串", &[("input", "合法 JSON")]),
        s("json.unescape", "JSON 去转义，不美化", &[("input", "转义文本")]),
        s("json.validate", "校验 JSON", &[("input", "JSON 字符串")]),
        s("json.info", "JSON 结构信息", &[("input", "JSON 字符串")]),
        s("json.diff", "对比两段 JSON", &[("left", "原始 JSON"), ("right", "对比 JSON")]),
        s("json.to_java", "JSON 转 Java 类", &[("input", "JSON"), ("class_name", "类名")]),
        s("json.to_csharp", "JSON 转 C# 类", &[("input", "JSON"), ("class_name", "类名")]),
        s("json.to_go", "JSON 转 Go 结构体", &[("input", "JSON"), ("struct_name", "结构体名")]),
        s("json.to_python", "JSON 转 Python 类", &[("input", "JSON"), ("class_name", "类名")]),
        s("json.to_typescript", "JSON 转 TypeScript 接口", &[("input", "JSON"), ("interface_name", "接口名")]),
        s("encoding.url_decode", "URL 解码", &[("input", "已编码文本")]),
        s("encoding.unicode_encode", "中文转 Unicode 转义", &[("input", "文本")]),
        s("encoding.unicode_decode", "Unicode 转义转中文", &[("input", "\\uXXXX 文本")]),
        s("encoding.base58_encode", "Base58 编码", &[("input", "文本")]),
        s("encoding.base58_decode", "Base58 解码", &[("input", "Base58")]),
        s("encoding.hex_encode", "文本转十六进制", &[("input", "文本")]),
        s("encoding.hex_decode", "十六进制转文本", &[("input", "十六进制")]),
        s("encoding.html_encode", "HTML 实体编码", &[("input", "文本")]),
        s("encoding.html_decode", "HTML 实体解码", &[("input", "实体文本")]),
        s("encoding.punycode_encode", "Punycode 编码", &[("input", "域名或文本")]),
        s("encoding.punycode_decode", "Punycode 解码", &[("input", "Punycode")]),
        s("encoding.binary_encode", "文本转二进制", &[("input", "文本")]),
        s("encoding.binary_decode", "二进制转文本", &[("input", "空格分隔的二进制")]),
        s("encoding.morse_encode", "文本转摩尔斯码", &[("input", "文本")]),
        s("encoding.morse_decode", "摩尔斯码转文本", &[("input", "摩尔斯码")]),
        s("encoding.rot13", "ROT13", &[("input", "文本")]),
        s("encoding.detect", "启发式编码检测", &[("input", "文本")]),
        s("charset.detect", "检测字符集", &[("input", "文本")]),
        s("charset.convert_text", "UTF-8 与 GBK 互转", &[("input", "文本"), ("from", "源字符集"), ("to", "目标字符集")]),
        s("jwt.generate", "生成 JWT", &[("payload", "JSON payload"), ("secret", "签名密钥")]),
        s("gm.sm3", "SM3 摘要", &[("input", "文本")]),
        s("gm.hmac_sm3", "HMAC-SM3", &[("message", "消息"), ("key", "密钥")]),
        s("gm.sm2_sign", "SM2 签名", &[("message", "消息"), ("private_key", "私钥")]),
        s("gm.sm2_verify", "SM2 验签", &[("message", "消息"), ("signature", "签名"), ("public_key", "公钥")]),
        tool("gm.sm4_key", "生成 SM4 密钥", empty_schema()),
        tool("gm.sm2_keypair", "生成 SM2 密钥对", empty_schema()),
        tool("uuid.v7", "生成 UUID v7", empty_schema()),
        tool("uuid.nil", "NIL UUID", empty_schema()),
        s("uuid.v5", "生成 UUID v5", &[("namespace", "命名空间 UUID"), ("name", "名称")]),
        s("uuid.validate", "校验 UUID", &[("input", "UUID")]),
        s("uuid.to_base64", "UUID 转 Base64", &[("input", "UUID")]),
        s("uuid.from_base64", "Base64 转 UUID", &[("input", "Base64")]),
        s("uuid.version", "读取 UUID 版本", &[("input", "UUID")]),
        s("cron.validate", "校验 Cron", &[("input", "5 字段 cron")]),
        s("cron.to_natural", "Cron 转自然语言", &[("input", "5 字段 cron")]),
        s("number.scientific", "科学计数法转小数", &[("input", "科学计数法")]),
        s("xml.minify", "XML 压缩", &[("input", "XML")]),
        s("xml.to_json", "XML 转 JSON", &[("input", "XML")]),
        s("json.to_xml", "JSON 转 XML", &[("input", "JSON")]),
        s("xml.to_yaml", "XML 转 YAML", &[("input", "XML")]),
        s("yaml.to_xml", "YAML 转 XML", &[("input", "YAML")]),
        s("yaml.validate", "校验 YAML", &[("input", "YAML")]),
        s("yaml.to_json", "YAML 转 JSON", &[("input", "YAML")]),
        s("json.to_yaml", "JSON 转 YAML", &[("input", "JSON")]),
        s("yaml.merge", "合并两段 YAML", &[("left", "基础 YAML"), ("right", "覆盖 YAML")]),
        s("sql.format", "SQL 格式化", &[("input", "SQL")]),
        s("sql.minify", "SQL 压缩", &[("input", "SQL")]),
        s("sql.escape", "SQL 转义", &[("input", "文本"), ("db_type", "mysql/postgres/mssql/sqlite")]),
        s("sql.unescape", "SQL 反转义", &[("input", "文本"), ("db_type", "mysql/postgres/mssql/sqlite")]),
        s("csv.stats", "CSV 统计", &[("input", "CSV 文本")]),
        s("log.count_errors", "统计日志错误", &[("input", "日志文本")]),
        s("log.levels", "提取日志级别", &[("input", "日志文本")]),
        s("image.info", "图片文件信息", &[("path", "图片路径")]),
        s("image.to_base64", "图片转 Base64", &[("path", "图片路径")]),
        s("qrcode.parse", "解析二维码", &[("image_data", "图片数据")]),
        tool("color.random", "随机颜色", empty_schema()),
        tool("password.generate", "生成随机密码", json!({
            "type": "object",
            "properties": { "length": { "type": "integer", "description": "长度，默认 16" } },
            "additionalProperties": false
        })),
    ];

    specs.push(tool(
        "encoding.caesar",
        "凯撒密码",
        json!({
            "type": "object",
            "properties": {
                "input": { "type": "string", "description": "文本" },
                "shift": { "type": "integer", "description": "位移，默认 3" },
                "direction": { "type": "string", "description": "encode 或 decode", "enum": ["encode", "decode"] }
            },
            "required": ["input"],
            "additionalProperties": false
        }),
    ));
    specs.push(tool(
        "encoding.case",
        "大小写或反转",
        json!({
            "type": "object",
            "properties": {
                "input": { "type": "string", "description": "文本" },
                "mode": { "type": "string", "description": "upper、lower 或 reverse", "enum": ["upper", "lower", "reverse"] }
            },
            "required": ["input", "mode"],
            "additionalProperties": false
        }),
    ));
    specs.push(tool(
        "gm.sm4_encrypt",
        "SM4 加密",
        json!({
            "type": "object",
            "properties": {
                "plaintext": { "type": "string" },
                "key": { "type": "string" },
                "iv": { "type": "string", "description": "可选 IV" }
            },
            "required": ["plaintext", "key"],
            "additionalProperties": false
        }),
    ));
    specs.push(tool(
        "gm.sm4_decrypt",
        "SM4 解密",
        json!({
            "type": "object",
            "properties": {
                "ciphertext": { "type": "string" },
                "key": { "type": "string" },
                "iv": { "type": "string" }
            },
            "required": ["ciphertext", "key"],
            "additionalProperties": false
        }),
    ));
    specs.push(tool(
        "uuid.batch",
        "批量生成 UUID",
        json!({
            "type": "object",
            "properties": {
                "count": { "type": "integer", "description": "数量 1..=1000" },
                "version": { "type": "string", "description": "v4 或 v7", "enum": ["v4", "v7"] }
            },
            "required": ["count", "version"],
            "additionalProperties": false
        }),
    ));
    specs.push(tool(
        "cron.generate",
        "按字段生成 Cron",
        required_string_props(&[
            ("minute", "分"),
            ("hour", "时"),
            ("day", "日"),
            ("month", "月"),
            ("dow", "周"),
        ]),
    ));
    specs.push(tool(
        "number.roman",
        "整数转罗马数字",
        json!({
            "type": "object",
            "properties": { "input": { "type": "integer", "description": "1..=3999" } },
            "required": ["input"],
            "additionalProperties": false
        }),
    ));
    specs.push(tool(
        "number.fraction",
        "分数与小数互转",
        json!({
            "type": "object",
            "properties": {
                "mode": { "type": "string", "enum": ["to_decimal", "to_fraction"] },
                "numerator": { "type": "integer" },
                "denominator": { "type": "integer" },
                "decimal": { "type": "number" }
            },
            "required": ["mode"],
            "additionalProperties": false
        }),
    ));
    specs.push(tool(
        "code.format",
        "按语言格式化代码",
        json!({
            "type": "object",
            "properties": {
                "input": { "type": "string" },
                "language": { "type": "string", "description": "json/js/python 等页面支持的语言" },
                "indent_size": { "type": "integer" },
                "use_tabs": { "type": "boolean" }
            },
            "required": ["input", "language"],
            "additionalProperties": false
        }),
    ));
    specs.push(tool(
        "regex.test",
        "正则匹配",
        json!({
            "type": "object",
            "properties": {
                "pattern": { "type": "string" },
                "text": { "type": "string" },
                "flags": { "type": "string", "description": "如 i、m，可空" }
            },
            "required": ["pattern", "text"],
            "additionalProperties": false
        }),
    ));
    specs.push(s("text.compare", "文本对比", &[("left", "文本一"), ("right", "文本二")]));
    specs.push(tool(
        "text.dedupe",
        "按行去重",
        json!({
            "type": "object",
            "properties": {
                "input": { "type": "string" },
                "ignore_case": { "type": "boolean" }
            },
            "required": ["input"],
            "additionalProperties": false
        }),
    ));
    specs.push(tool(
        "text.sort",
        "按行排序",
        json!({
            "type": "object",
            "properties": {
                "input": { "type": "string" },
                "order": { "type": "string", "enum": ["asc", "desc"] },
                "ignore_case": { "type": "boolean" }
            },
            "required": ["input", "order"],
            "additionalProperties": false
        }),
    ));
    specs.push(tool_confirm(
        "db.test",
        "测试数据库连接（需批准）",
        json!({
            "type": "object",
            "properties": {
                "engine": { "type": "string", "enum": ["mysql", "postgres", "sqlite"] },
                "host": { "type": "string" },
                "port": { "type": "integer" },
                "username": { "type": "string" },
                "password": { "type": "string" },
                "database": { "type": "string" },
                "file_path": { "type": "string", "description": "sqlite 文件路径" }
            },
            "required": ["engine"],
            "additionalProperties": false
        }),
    ));
    specs.push(tool(
        "csv.to_json",
        "CSV 转 JSON",
        json!({
            "type": "object",
            "properties": {
                "input": { "type": "string" },
                "has_header": { "type": "boolean" }
            },
            "required": ["input", "has_header"],
            "additionalProperties": false
        }),
    ));
    specs.push(tool(
        "json.to_csv",
        "JSON 转 CSV",
        json!({
            "type": "object",
            "properties": {
                "input": { "type": "string" },
                "has_header": { "type": "boolean" }
            },
            "required": ["input", "has_header"],
            "additionalProperties": false
        }),
    ));
    specs.push(s("csv.format", "按分隔符格式化 CSV", &[("input", "CSV"), ("delimiter", "分隔符")]));
    specs.push(s("log.analyze", "按模式分析日志", &[("input", "日志"), ("pattern", "匹配模式")]));
    specs.push(tool(
        "log.filter",
        "按关键词过滤日志",
        json!({
            "type": "object",
            "properties": {
                "input": { "type": "string" },
                "keyword": { "type": "string" },
                "case_sensitive": { "type": "boolean" },
                "invert": { "type": "boolean" }
            },
            "required": ["input", "keyword"],
            "additionalProperties": false
        }),
    ));
    specs.push(tool(
        "qrcode.generate",
        "生成二维码",
        json!({
            "type": "object",
            "properties": {
                "text": { "type": "string" },
                "size": { "type": "integer", "description": "边长像素，默认 200" }
            },
            "required": ["text"],
            "additionalProperties": false
        }),
    ));
    specs.push(s("barcode.generate", "生成条码", &[("text", "内容"), ("barcode_type", "CODE128/EAN13/EAN8/UPC/CODE39")]));
    specs.push(tool(
        "color.rgb_hex",
        "RGB 与 HEX 互转",
        json!({
            "type": "object",
            "properties": {
                "mode": { "type": "string", "enum": ["rgb_to_hex", "hex_to_rgb"] },
                "r": { "type": "integer" },
                "g": { "type": "integer" },
                "b": { "type": "integer" },
                "hex": { "type": "string" }
            },
            "required": ["mode"],
            "additionalProperties": false
        }),
    ));
    specs.push(tool(
        "color.to_hsl",
        "RGB 转 HSL",
        json!({
            "type": "object",
            "properties": {
                "r": { "type": "integer" }, "g": { "type": "integer" }, "b": { "type": "integer" }
            },
            "required": ["r", "g", "b"],
            "additionalProperties": false
        }),
    ));
    specs.push(tool(
        "color.to_hsv",
        "RGB 转 HSV",
        json!({
            "type": "object",
            "properties": {
                "r": { "type": "integer" }, "g": { "type": "integer" }, "b": { "type": "integer" }
            },
            "required": ["r", "g", "b"],
            "additionalProperties": false
        }),
    ));
    specs.push(tool(
        "color.brightness",
        "调整 HEX 亮度",
        json!({
            "type": "object",
            "properties": {
                "hex": { "type": "string" },
                "percent": { "type": "integer" }
            },
            "required": ["hex", "percent"],
            "additionalProperties": false
        }),
    ));
    specs.push(tool(
        "geo.convert",
        "WGS84/GCJ02/BD09 互转",
        json!({
            "type": "object",
            "properties": {
                "lat": { "type": "number" },
                "lng": { "type": "number" },
                "from": { "type": "string", "enum": ["WGS84", "GCJ02", "BD09"] },
                "to": { "type": "string", "enum": ["WGS84", "GCJ02", "BD09"] }
            },
            "required": ["lat", "lng", "from", "to"],
            "additionalProperties": false
        }),
    ));
    specs.push(tool(
        "geo.distance",
        "两点距离",
        json!({
            "type": "object",
            "properties": {
                "lat1": { "type": "number" }, "lng1": { "type": "number" },
                "lat2": { "type": "number" }, "lng2": { "type": "number" },
                "system": { "type": "string", "enum": ["WGS84", "GCJ02", "BD09"] },
                "method": { "type": "string", "enum": ["haversine", "vincenty", "cosine"] }
            },
            "required": ["lat1", "lng1", "lat2", "lng2", "system", "method"],
            "additionalProperties": false
        }),
    ));
    specs.push(tool_confirm(
        "http.request",
        "发送 HTTP 请求（需批准）",
        json!({
            "type": "object",
            "properties": {
                "method": { "type": "string" },
                "url": { "type": "string" },
                "headers": { "type": "object" },
                "body": { "type": "string" },
                "timeout": { "type": "integer", "description": "秒，默认 30" }
            },
            "required": ["method", "url"],
            "additionalProperties": false
        }),
    ));
    specs.push(tool_confirm("password.list", "列出本地密码（需批准）", empty_schema()));
    specs.push(tool_confirm(
        "password.get",
        "读取一条本地密码（需批准）",
        required_string_props(&[("id", "条目 id")]),
    ));
    specs.push(tool_confirm(
        "password.save",
        "保存本地密码（需批准）",
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "更新时填写" },
                "name": { "type": "string" },
                "username": { "type": "string" },
                "password": { "type": "string" },
                "url": { "type": "string" },
                "notes": { "type": "string" }
            },
            "required": ["name", "password"],
            "additionalProperties": false
        }),
    ));
    specs.push(tool_confirm(
        "password.delete",
        "删除本地密码（需批准）",
        required_string_props(&[("id", "条目 id")]),
    ));
    specs
}

fn req<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("参数错误：缺少字符串字段 `{key}`"))
}

fn opt_str<'a>(args: &'a Value, key: &str) -> &'a str {
    args.get(key).and_then(|v| v.as_str()).unwrap_or("")
}

fn req_u8(args: &Value, key: &str) -> Result<u8, String> {
    let n = args
        .get(key)
        .and_then(|v| v.as_u64())
        .ok_or_else(|| format!("参数错误：缺少整数 `{key}`"))?;
    u8::try_from(n).map_err(|_| format!("`{key}` 超出 0..=255"))
}

fn req_f64(args: &Value, key: &str) -> Result<f64, String> {
    args.get(key)
        .and_then(|v| v.as_f64())
        .ok_or_else(|| format!("参数错误：缺少数字 `{key}`"))
}

fn flag(args: &Value, key: &str) -> bool {
    args.get(key).and_then(|v| v.as_bool()).unwrap_or(false)
}

fn show(value: impl serde::Serialize) -> Result<String, String> {
    let v = serde_json::to_value(value).map_err(|e| e.to_string())?;
    match v {
        Value::String(s) => Ok(s),
        other => serde_json::to_string(&other).map_err(|e| e.to_string()),
    }
}

fn pick_string(value: impl serde::Serialize, key: &str) -> Result<String, String> {
    let v = serde_json::to_value(value).map_err(|e| e.to_string())?;
    v.get(key)
        .and_then(|x| x.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| format!("结果缺少 {key}"))
}

pub fn dispatch_extra(id: &str, args: &Value) -> Result<String, String> {
    match id {
        "json.compress" => pick_string(
            crate::commands::json::compress_json(req(args, "input")?.to_string())?,
            "formatted",
        ),
        "json.escape" => pick_string(
            crate::commands::json::escape_json(req(args, "input")?.to_string())?,
            "escaped",
        ),
        "json.unescape" => pick_string(
            crate::commands::json::unescape_json(req(args, "input")?.to_string())?,
            "unescaped",
        ),
        "json.validate" => {
            crate::commands::json::validate_json(req(args, "input")?.to_string())?;
            Ok("JSON格式有效".to_string())
        }
        "json.info" => show(crate::commands::json::get_json_info(req(args, "input")?.to_string())?),
        "json.diff" => show(crate::commands::json::compare_json(
            req(args, "left")?.to_string(),
            req(args, "right")?.to_string(),
        )?),
        "json.to_java" => crate::commands::data_convert::json_to_java(
            req(args, "input")?.to_string(),
            req(args, "class_name")?.to_string(),
        ),
        "json.to_csharp" => crate::commands::data_convert::json_to_csharp(
            req(args, "input")?.to_string(),
            req(args, "class_name")?.to_string(),
        ),
        "json.to_go" => crate::commands::data_convert::json_to_go(
            req(args, "input")?.to_string(),
            req(args, "struct_name")?.to_string(),
        ),
        "json.to_python" => crate::commands::data_convert::json_to_python(
            req(args, "input")?.to_string(),
            req(args, "class_name")?.to_string(),
        ),
        "json.to_typescript" => crate::commands::data_convert::json_to_typescript(
            req(args, "input")?.to_string(),
            req(args, "interface_name")?.to_string(),
        ),
        "encoding.url_decode" => crate::commands::encoding::url_decode(req(args, "input")?.to_string()),
        "encoding.unicode_encode" => {
            crate::commands::encoding::chinese_to_unicode(req(args, "input")?.to_string())
        }
        "encoding.unicode_decode" => {
            crate::commands::encoding::unicode_to_chinese(req(args, "input")?.to_string())
        }
        "encoding.base58_encode" => crate::commands::encoding::base58_encode(req(args, "input")?.to_string()),
        "encoding.base58_decode" => crate::commands::encoding::base58_decode(req(args, "input")?.to_string()),
        "encoding.hex_encode" => crate::commands::encoding::string_to_hex(req(args, "input")?.to_string()),
        "encoding.hex_decode" => crate::commands::encoding::hex_to_string(req(args, "input")?.to_string()),
        "encoding.html_encode" => crate::commands::encoding::html_encode(req(args, "input")?.to_string()),
        "encoding.html_decode" => crate::commands::encoding::html_decode(req(args, "input")?.to_string()),
        "encoding.punycode_encode" => {
            crate::commands::encoding::punycode_encode(req(args, "input")?.to_string())
        }
        "encoding.punycode_decode" => {
            crate::commands::encoding::punycode_decode(req(args, "input")?.to_string())
        }
        "encoding.binary_encode" => {
            let hex = crate::commands::encoding::string_to_hex(req(args, "input")?.to_string())?;
            crate::commands::encoding::hex_to_binary(hex)
        }
        "encoding.binary_decode" => {
            let hex = crate::commands::encoding::binary_to_hex(req(args, "input")?.to_string())?;
            crate::commands::encoding::hex_to_string(hex)
        }
        "encoding.morse_encode" => Ok(crate::commands::encoding::morse_encode(req(args, "input")?.to_string())),
        "encoding.morse_decode" => Ok(crate::commands::encoding::morse_decode(req(args, "input")?.to_string())),
        "encoding.rot13" => Ok(crate::commands::encoding::rot13(req(args, "input")?.to_string())),
        "encoding.caesar" => {
            let shift = args.get("shift").and_then(|v| v.as_i64()).unwrap_or(3) as i32;
            let decode = opt_str(args, "direction") == "decode";
            Ok(crate::commands::encoding::caesar(
                req(args, "input")?.to_string(),
                shift,
                decode,
            ))
        }
        "encoding.case" => Ok(crate::commands::encoding::encoding_case(
            req(args, "input")?.to_string(),
            req(args, "mode")?.to_string(),
        )),
        "encoding.detect" => Ok(crate::commands::encoding::encoding_detect(req(args, "input")?.to_string())),
        "charset.detect" => Ok(crate::commands::charset_tools::detect_encoding(
            req(args, "input")?.to_string(),
        )),
        "charset.convert_text" => crate::commands::charset_tools::convert_encoding(
            req(args, "input")?.to_string(),
            req(args, "from")?.to_string(),
            req(args, "to")?.to_string(),
        ),
        "jwt.generate" => crate::commands::crypto::generate_jwt(
            req(args, "payload")?.to_string(),
            req(args, "secret")?.to_string(),
        ),
        "gm.sm3" => Ok(crate::commands::gm_crypto::sm3_hash(req(args, "input")?.to_string())),
        "gm.sm4_encrypt" => crate::commands::gm_crypto::sm4_encrypt(
            req(args, "plaintext")?.to_string(),
            req(args, "key")?.to_string(),
            opt_str(args, "iv").to_string(),
        ),
        "gm.sm4_decrypt" => crate::commands::gm_crypto::sm4_decrypt(
            req(args, "ciphertext")?.to_string(),
            req(args, "key")?.to_string(),
            opt_str(args, "iv").to_string(),
        ),
        "gm.sm4_key" => Ok(crate::commands::gm_crypto::generate_sm4_key()),
        "gm.sm2_keypair" => {
            let (priv_key, pub_key) = crate::commands::gm_crypto::generate_sm2_keypair()?;
            show(json!({ "private_key": priv_key, "public_key": pub_key }))
        }
        "gm.sm2_sign" => crate::commands::gm_crypto::sm2_sign(
            req(args, "message")?.to_string(),
            req(args, "private_key")?.to_string(),
        ),
        "gm.sm2_verify" => {
            let ok = crate::commands::gm_crypto::sm2_verify(
                req(args, "message")?.to_string(),
                req(args, "signature")?.to_string(),
                req(args, "public_key")?.to_string(),
            )?;
            Ok(if ok { "验签通过".to_string() } else { "验签失败".to_string() })
        }
        "gm.hmac_sm3" => Ok(crate::commands::gm_crypto::hmac_sm3(
            req(args, "message")?.to_string(),
            req(args, "key")?.to_string(),
        )),
        "uuid.v7" => Ok(crate::commands::uuid_tools::generate_uuid_v7()),
        "uuid.v5" => crate::commands::uuid_tools::generate_uuid_v5(
            req(args, "namespace")?.to_string(),
            req(args, "name")?.to_string(),
        ),
        "uuid.batch" => {
            let count = args
                .get("count")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| "参数错误：缺少整数 `count`".to_string())? as u32;
            show(crate::commands::uuid_tools::generate_uuid_batch(
                count,
                req(args, "version")?.to_string(),
            ))
        }
        "uuid.validate" => Ok(crate::commands::uuid_tools::validate_uuid(req(args, "input")?.to_string()).to_string()),
        "uuid.to_base64" => crate::commands::uuid_tools::uuid_to_base64(req(args, "input")?.to_string()),
        "uuid.from_base64" => crate::commands::uuid_tools::base64_to_uuid(req(args, "input")?.to_string()),
        "uuid.version" => Ok(crate::commands::uuid_tools::get_uuid_version(req(args, "input")?.to_string())?.to_string()),
        "uuid.nil" => Ok(crate::commands::uuid_tools::nil_uuid()),
        "cron.generate" => Ok(crate::commands::cron_tools::generate_cron(
            req(args, "minute")?.to_string(),
            req(args, "hour")?.to_string(),
            req(args, "day")?.to_string(),
            req(args, "month")?.to_string(),
            req(args, "dow")?.to_string(),
        )),
        "cron.validate" => {
            let ok = crate::commands::cron_tools::validate_cron(req(args, "input")?.to_string())?;
            Ok(if ok { "Cron 有效".to_string() } else { "Cron 无效".to_string() })
        }
        "cron.to_natural" => crate::commands::cron_tools::cron_to_natural_language(req(args, "input")?.to_string()),
        "number.scientific" => crate::commands::number_tools::scientific_to_decimal(req(args, "input")?.to_string()),
        "number.roman" => {
            let n = args
                .get("input")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| "参数错误：缺少整数 `input`".to_string())?;
            crate::commands::number_tools::to_roman(n as u32)
        }
        "number.fraction" => match req(args, "mode")? {
            "to_decimal" => {
                let n = args.get("numerator").and_then(|v| v.as_i64()).unwrap_or(0);
                let d = args.get("denominator").and_then(|v| v.as_i64()).unwrap_or(1);
                crate::commands::number_tools::fraction_to_decimal(n, d)
            }
            "to_fraction" => {
                let d = req_f64(args, "decimal")?;
                Ok(crate::commands::number_tools::decimal_to_fraction(d))
            }
            other => Err(format!("未知 mode: {other}")),
        },
        "xml.minify" => crate::commands::xml_utils::minify_xml(req(args, "input")?.to_string()),
        "xml.to_json" => crate::commands::xml_utils::xml_to_json(req(args, "input")?.to_string()),
        "json.to_xml" => crate::commands::xml_utils::json_to_xml(req(args, "input")?.to_string()),
        "xml.to_yaml" => crate::commands::xml_utils::xml_to_yaml(req(args, "input")?.to_string()),
        "yaml.to_xml" => crate::commands::xml_utils::yaml_to_xml(req(args, "input")?.to_string()),
        "yaml.validate" => {
            let ok = crate::commands::yaml_utils::validate_yaml(req(args, "input")?.to_string())?;
            Ok(if ok { "YAML 有效".to_string() } else { "YAML 无效".to_string() })
        }
        "yaml.to_json" => crate::commands::yaml_utils::yaml_to_json(req(args, "input")?.to_string()),
        "json.to_yaml" => crate::commands::yaml_utils::json_to_yaml(req(args, "input")?.to_string()),
        "yaml.merge" => crate::commands::yaml_utils::merge_yaml(
            req(args, "left")?.to_string(),
            req(args, "right")?.to_string(),
        ),
        "code.format" => pick_string(
            crate::commands::code::format_code(
                req(args, "input")?.to_string(),
                req(args, "language")?.to_string(),
                args.get("indent_size").and_then(|v| v.as_u64()).map(|n| n as usize),
                args.get("use_tabs").and_then(|v| v.as_bool()),
            )?,
            "formatted",
        ),
        "regex.test" => show(crate::commands::text_utils::regex_test(
            req(args, "pattern")?.to_string(),
            req(args, "text")?.to_string(),
            opt_str(args, "flags").to_string(),
        )?),
        "text.compare" => show(crate::commands::text_utils::text_compare(
            req(args, "left")?.to_string(),
            req(args, "right")?.to_string(),
        )),
        "text.dedupe" => Ok(crate::commands::text_utils::text_deduplicate(
            req(args, "input")?.to_string(),
            flag(args, "ignore_case"),
        )),
        "text.sort" => Ok(crate::commands::text_utils::text_sort(
            req(args, "input")?.to_string(),
            req(args, "order")?.to_string(),
            flag(args, "ignore_case"),
        )),
        "sql.format" => crate::commands::sql_utils::format_sql(req(args, "input")?.to_string()),
        "sql.minify" => Ok(crate::commands::sql_utils::minify_sql(req(args, "input")?.to_string())),
        "sql.escape" => Ok(crate::commands::sql_utils::escape_sql(
            req(args, "input")?.to_string(),
            req(args, "db_type")?.to_string(),
        )),
        "sql.unescape" => Ok(crate::commands::sql_utils::unescape_sql(
            req(args, "input")?.to_string(),
            req(args, "db_type")?.to_string(),
        )),
        "db.test" => dispatch_db(args),
        "csv.to_json" => crate::commands::csv_utils::csv_to_json(
            req(args, "input")?.to_string(),
            flag(args, "has_header"),
        ),
        "json.to_csv" => crate::commands::csv_utils::json_to_csv(
            req(args, "input")?.to_string(),
            flag(args, "has_header"),
        ),
        "csv.format" => crate::commands::csv_utils::format_csv(
            req(args, "input")?.to_string(),
            req(args, "delimiter")?.to_string(),
        ),
        "csv.stats" => show(crate::commands::csv_utils::csv_stats(req(args, "input")?.to_string())?),
        "log.analyze" => show(crate::commands::log_analyzer::analyze_logs(
            req(args, "input")?.to_string(),
            req(args, "pattern")?.to_string(),
        )?),
        "log.count_errors" => show(crate::commands::log_analyzer::count_errors(req(args, "input")?.to_string())?),
        "log.filter" => Ok(crate::commands::log_analyzer::filter_logs(
            req(args, "input")?.to_string(),
            req(args, "keyword")?.to_string(),
            flag(args, "case_sensitive"),
            flag(args, "invert"),
        )),
        "log.levels" => show(crate::commands::log_analyzer::extract_log_levels(
            req(args, "input")?.to_string(),
        )?),
        "image.info" => show(crate::commands::image_utils::get_detailed_image_info(
            req(args, "path")?.to_string(),
        )?),
        "image.to_base64" => crate::commands::image_utils::image_to_base64(req(args, "path")?.to_string()),
        "qrcode.generate" => {
            let size = args.get("size").and_then(|v| v.as_u64()).unwrap_or(200) as u32;
            crate::commands::qrcode_tools::generate_qrcode(req(args, "text")?.to_string(), size)
        }
        "qrcode.parse" => crate::commands::qrcode_tools::parse_qrcode(req(args, "image_data")?.to_string()),
        "barcode.generate" => crate::commands::qrcode_tools::generate_barcode(
            req(args, "text")?.to_string(),
            req(args, "barcode_type")?.to_string(),
        ),
        "color.rgb_hex" => match req(args, "mode")? {
            "rgb_to_hex" => Ok(crate::commands::color_tools::rgb_to_hex(
                req_u8(args, "r")?,
                req_u8(args, "g")?,
                req_u8(args, "b")?,
            )),
            "hex_to_rgb" => {
                let (r, g, b) = crate::commands::color_tools::hex_to_rgb(req(args, "hex")?.to_string())?;
                Ok(format!("R: {r}, G: {g}, B: {b}"))
            }
            other => Err(format!("未知 mode: {other}")),
        },
        "color.to_hsl" => {
            let (h, s, l) = crate::commands::color_tools::rgb_to_hsl(
                req_u8(args, "r")?,
                req_u8(args, "g")?,
                req_u8(args, "b")?,
            );
            Ok(format!("H: {h:.1}, S: {s:.1}%, L: {l:.1}%"))
        }
        "color.to_hsv" => {
            let (h, s, v) = crate::commands::color_tools::rgb_to_hsv(
                req_u8(args, "r")?,
                req_u8(args, "g")?,
                req_u8(args, "b")?,
            );
            Ok(format!("H: {h:.1}, S: {s:.1}%, V: {v:.1}%"))
        }
        "color.random" => Ok(crate::commands::color_tools::random_color()),
        "color.brightness" => crate::commands::color_tools::adjust_brightness(
            req(args, "hex")?.to_string(),
            args.get("percent").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        ),
        "geo.convert" => crate::commands::geo::geo_convert(
            req_f64(args, "lat")?,
            req_f64(args, "lng")?,
            req(args, "from")?.to_string(),
            req(args, "to")?.to_string(),
        ),
        "geo.distance" => show(crate::commands::geo::geo_distance(
            req_f64(args, "lat1")?,
            req_f64(args, "lng1")?,
            req_f64(args, "lat2")?,
            req_f64(args, "lng2")?,
            req(args, "system")?.to_string(),
            req(args, "method")?.to_string(),
        )?),
        "http.request" => dispatch_http(args),
        "password.generate" => Ok(crate::commands::password_store::password_generate(
            args.get("length").and_then(|v| v.as_u64()).map(|n| n as u32),
        )),
        "password.list" => show(crate::commands::password_store::password_list()?),
        "password.get" => show(crate::commands::password_store::password_get(req(args, "id")?.to_string())?),
        "password.save" => show(crate::commands::password_store::password_save(
            args.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()),
            req(args, "name")?.to_string(),
            opt_str(args, "username").to_string(),
            req(args, "password")?.to_string(),
            opt_str(args, "url").to_string(),
            opt_str(args, "notes").to_string(),
        )?),
        "password.delete" => crate::commands::password_store::password_delete(req(args, "id")?.to_string()),
        _ => Err(format!("未注册的工具: {id}（unknown tool）")),
    }
}

fn dispatch_db(args: &Value) -> Result<String, String> {
    match req(args, "engine")? {
        "sqlite" => crate::commands::db_tools::test_sqlite_connection(req(args, "file_path")?.to_string()),
        "postgres" => crate::commands::db_tools::test_postgres_connection(
            req(args, "host")?.to_string(),
            args.get("port").and_then(|v| v.as_u64()).unwrap_or(5432) as u16,
            req(args, "username")?.to_string(),
            opt_str(args, "password").to_string(),
            req(args, "database")?.to_string(),
        ),
        "mysql" => tauri::async_runtime::block_on(crate::commands::db_tools::test_mysql_connection(
            req(args, "host")?.to_string(),
            args.get("port").and_then(|v| v.as_u64()).unwrap_or(3306) as u16,
            req(args, "username")?.to_string(),
            opt_str(args, "password").to_string(),
            req(args, "database")?.to_string(),
        )),
        other => Err(format!("未知数据库: {other}")),
    }
}

fn dispatch_http(args: &Value) -> Result<String, String> {
    let headers = args.get("headers").cloned().unwrap_or_else(|| json!({}));
    let body = args.get("body").and_then(|v| v.as_str()).map(|s| s.to_string());
    let timeout = args.get("timeout").and_then(|v| v.as_u64()).unwrap_or(30);
    let value = tauri::async_runtime::block_on(crate::commands::network::http_request(
        req(args, "method")?.to_string(),
        req(args, "url")?.to_string(),
        headers,
        body,
        timeout,
    ))?;
    show(value)
}
