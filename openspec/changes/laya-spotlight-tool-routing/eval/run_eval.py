#!/usr/bin/env python3
"""Zero-shot 评测：laya-multilingual 能否把中文自然语言查询路由到正确工具。

用法（仓库根目录）:
  HF_HOME=.venv-laya/hf HF_ENDPOINT=https://hf-mirror.com HF_HUB_DISABLE_XET=1 \
    .venv-laya/bin/python openspec/changes/laya-spotlight-tool-routing/eval/run_eval.py
"""

import json
import sys
import time
from pathlib import Path

from laya import Router

HERE = Path(__file__).parent
QUERIES = json.loads((HERE / "queries.json").read_text(encoding="utf-8"))["queries"]

# 与 Rust laya.rs build_tool_candidates 相同的候选文本格式
TOOLS = [
    (1, "图片压缩工具", "快速压缩图片大小而不损失质量，支持JPG、PNG等格式。", "图片处理,优化"),
    (2, "视频格式转换", "支持多种视频格式转换，包括MP4, AVI, MOV等常见格式。", "优化,视频处理"),
    (3, "密码管理器", "安全存储和管理所有密码，自动生成强密码，保护您的账户安全。", "安全工具,加密"),
    (4, "PDF工具箱", "合并、分割、压缩PDF文件，添加水印和密码保护等实用功能。", "文件管理,PDF"),
    (5, "屏幕标尺", "在屏幕上测量元素尺寸，支持像素、厘米、英寸等多种单位。", "PDF"),
    (6, "代码格式化", "美化您的代码，支持多种编程语言，提高代码可读性和规范性。", "设计工具,测量"),
    (7, "文件恢复工具", "恢复误删除的文件，支持多种文件系统和存储设备。", "文件管理,恢复"),
    (8, "网络测速", "测试您的网络下载、上传速度和延迟，提供详细分析报告。", "网络工具,测速"),
    (9, "JSON处理工具", "JSON美化、压缩、转义、去转义、验证和信息查看等实用功能。", "开发工具,编程"),
    (10, "Base64工具", "Base64编码和解码工具，支持中文字符和各种格式。", "转换,编码"),
    (11, "哈希生成器", "生成MD5、SHA-1、SHA-256等多种哈希值，用于数据校验和安全验证。", "安全工具,哈希"),
    (12, "JSON转实体类", "将JSON自动转换为Java、C#、Go、Python、TypeScript实体类。", "开发工具,编程"),
    (13, "JSON对比工具", "对比两个JSON的差异，高亮显示新增、删除和修改的字段。", "开发工具,编程"),
    (14, "JWT工具", "解析和生成JWT Token，支持签名验证。", "安全工具,加密"),
    (15, "正则表达式测试", "实时测试正则表达式，查看匹配结果。", "开发工具,编程"),
    (16, "时间戳转换", "Unix时间戳与日期时间互转，支持多种时间单位。", "开发工具,编程"),
    (17, "HTTP请求工具", "发送HTTP请求，支持GET/POST/PUT/DELETE等方法。", "开发工具,网络工具"),
    (18, "文本处理工具", "文本对比、去重、排序、正则替换等文本处理功能。", "开发工具,编程"),
    (19, "编码转换工具", "URL、Unicode、Base58/62等多种编码转换。", "转换,编码"),
    (20, "XML工具", "XML格式化、压缩、JSON/YAML转换等功能。", "开发工具,编程"),
    (21, "YAML工具", "YAML格式化、验证、JSON转换和合并。", "开发工具,编程"),
    (22, "国密算法工具", "SM2/SM3/SM4国密算法加密解密和签名。", "安全工具,加密"),
    (23, "SQL工具", "SQL格式化和转义，支持多种数据库。", "开发工具,编程"),
    (24, "数据库工具", "MySQL、PostgreSQL、SQLite连接测试。", "开发工具,编程"),
    (25, "图片工具", "图片格式转换、Base64编码、图片信息查看。", "图片处理,优化"),
    (26, "CSV工具", "CSV与JSON转换、格式化和统计。", "开发工具,编程"),
    (27, "日志分析工具", "日志分析、错误统计、过滤和级别提取。", "开发工具,编程"),
    (28, "颜色工具", "RGB/HEX/HSL/HSV颜色格式转换，生成随机颜色，调整亮度。", "设计工具,测量"),
    (29, "二维码工具", "生成二维码，解析二维码，生成条形码（CODE128、EAN13等）。", "转换,开发工具"),
    (30, "UUID工具", "生成v4/v7 UUID，批量生成，验证，Base64转换，版本检测。", "开发工具,编程"),
    (31, "Cron工具", "Cron表达式构建、验证和自然语言解释。", "开发工具,编程"),
    (32, "数字工具", "进制转换（十进制、十六进制、二进制、八进制），科学计数法，罗马数字。", "开发工具,编程"),
    (33, "字符编码工具", "编码检测，UTF-8/GBK转换，URL编解码，HTML实体，Punycode。", "转换,编码"),
    (34, "JSON转Query参数", "将POST的JSON对象转换为GET请求的URL查询参数。", "转换,编码"),
    (35, "坐标距离计算", "计算经纬度之间的距离，支持WGS84、GCJ02、BD09坐标系统，以及多种距离计算方法。", "测量,开发工具"),
    (36, "坐标可视化", "在地图上标注多个坐标点，支持批量粘贴坐标并展示。", "测量,开发工具"),
]

CRITERIA = {}
for tid, name, desc, tags in TOOLS:
    CRITERIA[str(tid)] = f"{name}|{desc}|{tags}"

QUESTIONS = {
    "tool": {
        "type": "choice",
        "instructions": "用户想使用哪个开发者工具？选择最匹配的一项。",
        "criteria": CRITERIA,
    }
}


def main() -> int:
    router = Router(preload=True)
    threshold = 0.7
    correct = 0
    hits_at_threshold = 0
    latencies = []
    misses = []
    for item in QUERIES:
        start = time.perf_counter()
        result = router.predict(item["query"], QUESTIONS, model="multilingual", max_len=8192)
        latencies.append((time.perf_counter() - start) * 1000)
        answer = result["answers"]["tool"]
        choice = answer["choice"]
        confidence = answer.get("answer_confidence", answer.get("confidence", 0.0))
        ok = int(choice) == item["expected_tool"]
        if ok:
            correct += 1
        if ok and confidence >= threshold:
            hits_at_threshold += 1
        if not ok or confidence < threshold:
            misses.append({
                "query": item["query"],
                "expected": item["expected_tool"],
                "choice": choice,
                "confidence": round(confidence, 3),
            })
        flag = "OK " if ok else "MISS"
        print(f"[{flag}] conf={confidence:.3f} got={choice} want={item['expected_tool']}  {item['query']}")

    n = len(QUERIES)
    latencies.sort()
    print("\n==== 汇总 ====")
    print(f"样本数: {n}")
    print(f"choice 准确率（不看阈值）: {correct}/{n} = {correct / n:.3f}")
    print(f"准确且 conf>={threshold}（可上线命中率）: {hits_at_threshold}/{n} = {hits_at_threshold / n:.3f}")
    print(f"延迟 ms: p50={latencies[n // 2]:.0f} p95={latencies[int(n * 0.95)]:.0f}")
    (HERE / "misses.json").write_text(
        json.dumps(misses, ensure_ascii=False, indent=2), encoding="utf-8"
    )
    print(f"失败/低置信明细已写入 {HERE / 'misses.json'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
