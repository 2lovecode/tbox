# Tasks

## 1. Backend

- [x] 1.1 重写 `unescape_json`：明文合法 JSON 直接返回；否则循环调用 `unescape_json_text` 至稳定或达上限；不 pretty；设置 `is_valid`
- [x] 1.2 单测：简单 `{\"a\":1}`、含 `\\u0026` / `\\n` 的 ES query 样例、已是明文的对象

## 2. Verify

- [x] 2.1 `cargo test` 覆盖 `commands::json` 相关用例通过
