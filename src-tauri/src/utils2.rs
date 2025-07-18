use std::path::PathBuf;
use std::fs;
pub fn get_db_path() -> PathBuf {
    let mut db_path = dirs::home_dir().expect("无法获取用户主目录");
    db_path.push(".toolbox"); // 可自定义子目录名
    fs::create_dir_all(&db_path).expect("创建目录失败");
    db_path.push("tools.db");
    db_path
}