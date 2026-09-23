//! In-process agent run registry: one in-flight turn per conversation.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

pub const TURN_IN_PROGRESS: &str = "turn_in_progress";

pub struct RunEntry {
    pub cancel: Arc<AtomicBool>,
    pub started_at: u64,
}

pub struct RunRegistry {
    inner: Mutex<HashMap<String, RunEntry>>,
}

impl Default for RunRegistry {
    fn default() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

impl RunRegistry {
    /// Begin a turn for `conversation_id`. Returns the cancel flag for the worker.
    /// Err(`turn_in_progress`) if that conversation already has an in-flight turn.
    pub fn try_begin(&self, conversation_id: &str) -> Result<Arc<AtomicBool>, String> {
        let mut map = self.inner.lock().map_err(|e| e.to_string())?;
        if map.contains_key(conversation_id) {
            return Err(TURN_IN_PROGRESS.to_string());
        }
        let cancel = Arc::new(AtomicBool::new(false));
        map.insert(
            conversation_id.to_string(),
            RunEntry {
                cancel: Arc::clone(&cancel),
                started_at: now_secs(),
            },
        );
        Ok(cancel)
    }

    /// Request cancel for a conversation. Unknown / idle ids are no-ops.
    pub fn cancel(&self, conversation_id: &str) {
        if let Ok(map) = self.inner.lock() {
            if let Some(entry) = map.get(conversation_id) {
                entry.cancel.store(true, Ordering::SeqCst);
            }
        }
    }

    /// Mark the conversation idle (remove registry entry).
    pub fn finish(&self, conversation_id: &str) {
        if let Ok(mut map) = self.inner.lock() {
            map.remove(conversation_id);
        }
    }

    pub fn is_running(&self, conversation_id: &str) -> bool {
        self.inner
            .lock()
            .map(|m| m.contains_key(conversation_id))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_begin_then_second_fails() {
        let reg = RunRegistry::default();
        let flag = reg.try_begin("c1").expect("first begin");
        assert!(!flag.load(Ordering::SeqCst));
        assert!(reg.is_running("c1"));
        let err = reg.try_begin("c1").unwrap_err();
        assert_eq!(err, "turn_in_progress");
    }

    #[test]
    fn parallel_conversations_ok() {
        let reg = RunRegistry::default();
        reg.try_begin("a").unwrap();
        reg.try_begin("b").unwrap();
        assert!(reg.is_running("a") && reg.is_running("b"));
    }

    #[test]
    fn cancel_sets_flag_finish_clears() {
        let reg = RunRegistry::default();
        let flag = reg.try_begin("c1").unwrap();
        reg.cancel("c1");
        assert!(flag.load(Ordering::SeqCst));
        reg.finish("c1");
        assert!(!reg.is_running("c1"));
        reg.cancel("missing"); // no panic
    }
}
