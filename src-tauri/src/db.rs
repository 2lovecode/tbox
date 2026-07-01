use std::path::PathBuf;
use std::{fs, sync::OnceLock};

use rusqlite::Connection;

static DB_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Resolve and cache the absolute path to the SQLite database file.
pub fn get_db_path() -> &'static PathBuf {
    DB_PATH.get_or_init(|| {
        let mut db_path = dirs::home_dir().expect("无法获取用户主目录");
        db_path.push(".toolbox");
        fs::create_dir_all(&db_path).expect("创建目录失败");
        db_path.push("tools.db");
        db_path
    })
}

/// Open a new connection to the SQLite database.
///
/// Each command gets its own connection so failures stay isolated. Use
/// [`with_connection`] when you need a borrowed handle for transactions or
/// multiple queries in the same scope.
pub fn open_connection() -> Result<Connection, String> {
    Connection::open(get_db_path()).map_err(|e| e.to_string())
}

/// Convenience wrapper that opens a connection, runs `f`, and propagates the
/// error as a `String` so it can be returned directly from a Tauri command.
pub fn with_connection<T, F>(f: F) -> Result<T, String>
where
    F: FnOnce(&Connection) -> Result<T, rusqlite::Error>,
{
    let conn = open_connection()?;
    f(&conn).map_err(|e| e.to_string())
}
