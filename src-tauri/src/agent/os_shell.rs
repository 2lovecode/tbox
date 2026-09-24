//! 受限 OS shell：黑名单、超时、输出截断、命令探测与默认 cwd。

use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

const TIMEOUT_SECS: u64 = 15;
const MAX_OUTPUT_BYTES: usize = 64 * 1024;
const SHELL_PREFS: &str = "shell.json";

/// 危险可执行文件 basename（小写比较）。
const DENYLIST: &[&str] = &[
    "rm", "rmdir", "del", "erase", "rd", "sudo", "su", "doas", "mkfs", "dd",
    "shutdown", "reboot", "poweroff", "halt", "passwd", "chown", "chmod",
    "chgrp", "mkfs.ext4", "mkfs.ntfs", "diskpart", "format", "cipher",
    "powershell", "pwsh", "cmdlet", "reg", "regedit", "launchctl", "systemctl",
    "service", "kill", "killall", "pkill", "taskkill", "curl", "wget", "nc",
    "ncat", "netcat", "ssh", "scp", "sftp", "ftp", "telnet",
];

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellPrefs {
    #[serde(default)]
    pub shell_default_cwd: Option<String>,
}

fn prefs_path() -> PathBuf {
    crate::agent::llama_log::toolbox_dir().join(SHELL_PREFS)
}

pub fn load_shell_prefs() -> ShellPrefs {
    let raw = match std::fs::read(prefs_path()) {
        Ok(b) => b,
        Err(_) => return ShellPrefs::default(),
    };
    serde_json::from_slice(&raw).unwrap_or_default()
}

pub fn save_shell_prefs(prefs: ShellPrefs) -> Result<ShellPrefs, String> {
    let dir = crate::agent::llama_log::toolbox_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建配置目录失败: {e}"))?;
    let json = serde_json::to_vec_pretty(&prefs).map_err(|e| e.to_string())?;
    std::fs::write(prefs_path(), json).map_err(|e| format!("写入 shell 配置失败: {e}"))?;
    Ok(prefs)
}

pub fn default_cwd() -> PathBuf {
    let prefs = load_shell_prefs();
    if let Some(cwd) = prefs.shell_default_cwd.filter(|s| !s.trim().is_empty()) {
        return PathBuf::from(cwd);
    }
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

/// 从命令串提取首 token 的 basename（同类键）。
pub fn similar_key(command: &str) -> String {
    let token = first_token(command);
    Path::new(&token)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(token.as_str())
        .to_lowercase()
}

fn first_token(command: &str) -> String {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    // 简单：跳过 env 赋值前缀 FOO=bar
    let mut parts = trimmed.split_whitespace();
    while let Some(p) = parts.next() {
        if p.contains('=') && !p.starts_with('-') && !p.contains('/') {
            continue;
        }
        return p.trim_matches(|c| c == '"' || c == '\'').to_string();
    }
    String::new()
}

pub fn is_denied(command: &str) -> Option<String> {
    let key = similar_key(command);
    if key.is_empty() {
        return Some("命令为空".into());
    }
    if DENYLIST.iter().any(|d| *d == key) {
        return Some(format!("拒绝执行危险命令: {key}"));
    }
    None
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandAvailability {
    pub name: String,
    pub available: bool,
    pub alternative: Option<String>,
}

/// 探测常用命令可用性与替代建议。
pub fn probe_commands() -> Vec<CommandAvailability> {
    let candidates: &[(&str, Option<&str>)] = &[
        ("rg", Some("grep -R")),
        ("grep", None),
        ("cat", Some("type (Windows) / Get-Content")),
        ("head", Some("Get-Content -TotalCount")),
        ("tail", Some("Get-Content -Tail")),
        ("wc", Some("Measure-Object")),
        ("ls", Some("dir / Get-ChildItem")),
        ("dir", Some("ls")),
        ("find", Some("Get-ChildItem -Recurse")),
        ("which", Some("where")),
        ("where", Some("which")),
    ];
    candidates
        .iter()
        .map(|(name, alt)| CommandAvailability {
            name: (*name).to_string(),
            available: command_on_path(name),
            alternative: if command_on_path(name) {
                None
            } else {
                alt.map(|s| s.to_string())
            },
        })
        .collect()
}

fn command_on_path(name: &str) -> bool {
    #[cfg(windows)]
    {
        Command::new("where")
            .arg(name)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
    #[cfg(not(windows))]
    {
        Command::new("which")
            .arg(name)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

fn truncate_output(mut s: String) -> String {
    if s.len() <= MAX_OUTPUT_BYTES {
        return s;
    }
    s.truncate(MAX_OUTPUT_BYTES);
    s.push_str("\n…[输出已截断]");
    s
}

/// 执行受限 shell 命令。调用方须已完成审批。
pub fn execute(command: &str, cwd: Option<&str>) -> Result<String, String> {
    if let Some(reason) = is_denied(command) {
        return Err(reason);
    }
    let workdir = match cwd.map(str::trim).filter(|s| !s.is_empty()) {
        Some(c) => PathBuf::from(c),
        None => default_cwd(),
    };
    if !workdir.is_dir() {
        return Err(format!("工作目录不存在: {}", workdir.display()));
    }

    #[cfg(windows)]
    let mut child = Command::new("cmd")
        .args(["/C", command])
        .current_dir(&workdir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("启动 shell 失败: {e}"))?;

    #[cfg(not(windows))]
    let mut child = Command::new("sh")
        .args(["-c", command])
        .current_dir(&workdir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("启动 shell 失败: {e}"))?;

    let timeout = Duration::from_secs(TIMEOUT_SECS);
    let start = std::time::Instant::now();
    let status = loop {
        match child.try_wait().map_err(|e| e.to_string())? {
            Some(status) => break status,
            None if start.elapsed() > timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("命令超时（>{TIMEOUT_SECS}s）"));
            }
            None => std::thread::sleep(Duration::from_millis(40)),
        }
    };

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    if let Some(mut out) = child.stdout.take() {
        let _ = out.read_to_end(&mut stdout);
    }
    if let Some(mut err) = child.stderr.take() {
        let _ = err.read_to_end(&mut stderr);
    }

    let mut combined = String::new();
    if !stdout.is_empty() {
        combined.push_str(&String::from_utf8_lossy(&stdout));
    }
    if !stderr.is_empty() {
        if !combined.is_empty() {
            combined.push('\n');
        }
        combined.push_str(&String::from_utf8_lossy(&stderr));
    }
    if !status.success() {
        let code = status.code().unwrap_or(-1);
        if combined.is_empty() {
            combined = format!("退出码 {code}");
        } else {
            combined = format!("退出码 {code}\n{combined}");
        }
    }
    if combined.is_empty() {
        combined = "(无输出)".into();
    }
    Ok(truncate_output(combined))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn denylist_blocks_rm() {
        assert!(is_denied("rm -rf /").is_some());
        assert!(is_denied("sudo ls").is_some());
    }

    #[test]
    fn similar_key_basename() {
        assert_eq!(similar_key("/usr/bin/rg foo"), "rg");
        assert_eq!(similar_key("grep -R x"), "grep");
    }

    #[test]
    fn echo_ok() {
        let out = execute("echo hello-shell", None).unwrap();
        assert!(out.contains("hello-shell"), "{out}");
    }
}
