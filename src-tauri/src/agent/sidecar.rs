//! llama.cpp server sidecar lifecycle (CPU, loopback only).

use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;

/// Sidecar bind config. Port MUST NOT be 11434 (Ollama default).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidecarConfig {
    pub host: String,
    pub port: u16,
}

impl Default for SidecarConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 11435,
        }
    }
}

impl SidecarConfig {
    pub fn base_url(&self) -> String {
        format!("http://{}:{}/v1", self.host, self.port)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.host != "127.0.0.1" && self.host != "localhost" {
            return Err("sidecar 仅允许绑定 127.0.0.1".into());
        }
        if self.port == 11434 {
            return Err("禁止使用 Ollama 默认端口 11434".into());
        }
        Ok(())
    }
}

/// Abstraction over OS process control for unit tests.
pub trait ProcessCtl: Send {
    fn spawn(
        &mut self,
        bin: &Path,
        model: &Path,
        cfg: &SidecarConfig,
    ) -> Result<(), String>;
    fn kill(&mut self) -> Result<(), String>;
    fn is_running(&mut self) -> bool;
}

/// Real process controller wrapping `std::process::Child`.
pub struct OsProcess {
    child: Option<Child>,
}

impl Default for OsProcess {
    fn default() -> Self {
        Self { child: None }
    }
}

impl ProcessCtl for OsProcess {
    fn spawn(
        &mut self,
        bin: &Path,
        model: &Path,
        cfg: &SidecarConfig,
    ) -> Result<(), String> {
        cfg.validate()?;
        if self.is_running() {
            return Ok(());
        }
        let child = Command::new(bin)
            .arg("--host")
            .arg(&cfg.host)
            .arg("--port")
            .arg(cfg.port.to_string())
            .arg("-m")
            .arg(model)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("启动 sidecar 失败: {e}"))?;
        self.child = Some(child);
        Ok(())
    }

    fn kill(&mut self) -> Result<(), String> {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        Ok(())
    }

    fn is_running(&mut self) -> bool {
        match self.child.as_mut() {
            None => false,
            Some(child) => match child.try_wait() {
                Ok(None) => true,
                Ok(Some(_)) => {
                    self.child = None;
                    false
                }
                Err(_) => false,
            },
        }
    }
}

pub struct SidecarHandle<P: ProcessCtl = OsProcess> {
    pub config: SidecarConfig,
    pub model_path: PathBuf,
    process: P,
}

impl<P: ProcessCtl> SidecarHandle<P> {
    pub fn start(
        mut process: P,
        bin: &Path,
        model: &Path,
        config: SidecarConfig,
    ) -> Result<Self, String> {
        config.validate()?;
        process.spawn(bin, model, &config)?;
        Ok(Self {
            config,
            model_path: model.to_path_buf(),
            process,
        })
    }

    pub fn stop(&mut self) -> Result<(), String> {
        self.process.kill()
    }

    pub fn health(&mut self) -> bool {
        if !self.process.is_running() {
            return false;
        }
        let addr = format!("{}:{}", self.config.host, self.config.port);
        TcpStream::connect_timeout(
            &addr.parse().unwrap_or_else(|_| {
                std::net::SocketAddr::from(([127, 0, 0, 1], self.config.port))
            }),
            Duration::from_millis(200),
        )
        .is_ok()
    }
}

/// App-wide sidecar manager (optional running handle).
pub struct SidecarManager {
    inner: Mutex<Option<SidecarHandle>>,
    pub config: SidecarConfig,
}

impl Default for SidecarManager {
    fn default() -> Self {
        Self {
            inner: Mutex::new(None),
            config: SidecarConfig::default(),
        }
    }
}

impl SidecarManager {
    pub fn ensure_started(&self, bin: &Path, model: &Path) -> Result<String, String> {
        let mut guard = self.inner.lock().map_err(|e| e.to_string())?;
        if let Some(handle) = guard.as_mut() {
            if handle.health() {
                return Ok(handle.config.base_url());
            }
            let _ = handle.stop();
            *guard = None;
        }
        let handle = SidecarHandle::start(OsProcess::default(), bin, model, self.config.clone())?;
        let url = handle.config.base_url();
        *guard = Some(handle);
        Ok(url)
    }

    pub fn stop(&self) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(mut handle) = guard.take() {
                let _ = handle.stop();
            }
        }
    }

    pub fn health(&self) -> bool {
        self.inner
            .lock()
            .ok()
            .and_then(|mut g| g.as_mut().map(|h| h.health()))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};

    struct FakeProcess {
        running: AtomicBool,
        last_port: AtomicU16,
        last_host: Mutex<String>,
    }

    impl FakeProcess {
        fn new() -> Self {
            Self {
                running: AtomicBool::new(false),
                last_port: AtomicU16::new(0),
                last_host: Mutex::new(String::new()),
            }
        }
    }

    impl ProcessCtl for FakeProcess {
        fn spawn(
            &mut self,
            _bin: &Path,
            _model: &Path,
            cfg: &SidecarConfig,
        ) -> Result<(), String> {
            cfg.validate()?;
            self.last_port.store(cfg.port, Ordering::SeqCst);
            *self.last_host.lock().unwrap() = cfg.host.clone();
            self.running.store(true, Ordering::SeqCst);
            Ok(())
        }

        fn kill(&mut self) -> Result<(), String> {
            self.running.store(false, Ordering::SeqCst);
            Ok(())
        }

        fn is_running(&mut self) -> bool {
            self.running.load(Ordering::SeqCst)
        }
    }

    #[test]
    fn default_port_is_not_ollama() {
        let cfg = SidecarConfig::default();
        assert_eq!(cfg.port, 11435);
        assert_ne!(cfg.port, 11434);
        assert_eq!(cfg.host, "127.0.0.1");
        cfg.validate().unwrap();
    }

    #[test]
    fn rejects_ollama_port() {
        let cfg = SidecarConfig {
            host: "127.0.0.1".into(),
            port: 11434,
        };
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn start_records_loopback_bind() {
        let fake = FakeProcess::new();
        let cfg = SidecarConfig::default();
        let mut handle = SidecarHandle::start(
            fake,
            Path::new("llama-server"),
            Path::new("model.gguf"),
            cfg,
        )
        .unwrap();
        assert_eq!(handle.process.last_port.load(Ordering::SeqCst), 11435);
        assert_eq!(
            handle.process.last_host.lock().unwrap().as_str(),
            "127.0.0.1"
        );
        // Simulate crash
        handle.process.running.store(false, Ordering::SeqCst);
        assert!(!handle.process.is_running());
        // stop must not panic
        handle.stop().unwrap();
    }
}
