//! 工具审批闸门：Process 副作用工具在执行前等待用户决策。

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalDecision {
    Deny,
    Allow,
    AllowSimilar,
}

struct Pending {
    tx: std::sync::mpsc::Sender<ApprovalDecision>,
    conversation_id: String,
}

struct ApprovalState {
    pending: HashMap<String, Pending>,
    /// conversation_id -> allowed similar keys
    session_allow: HashMap<String, HashSet<String>>,
}

fn state() -> &'static Mutex<ApprovalState> {
    static STATE: OnceLock<Mutex<ApprovalState>> = OnceLock::new();
    STATE.get_or_init(|| {
        Mutex::new(ApprovalState {
            pending: HashMap::new(),
            session_allow: HashMap::new(),
        })
    })
}

pub fn clear_session(conversation_id: &str) {
    if let Ok(mut g) = state().lock() {
        g.session_allow.remove(conversation_id);
        let to_deny: Vec<String> = g
            .pending
            .iter()
            .filter(|(_, p)| p.conversation_id == conversation_id)
            .map(|(id, _)| id.clone())
            .collect();
        for id in to_deny {
            if let Some(p) = g.pending.remove(&id) {
                let _ = p.tx.send(ApprovalDecision::Deny);
            }
        }
    }
}

pub fn is_session_allowed(conversation_id: &str, similar_key: &str) -> bool {
    state()
        .lock()
        .ok()
        .map(|g| {
            g.session_allow
                .get(conversation_id)
                .map(|s| s.contains(similar_key))
                .unwrap_or(false)
        })
        .unwrap_or(false)
}

/// 注册待审批请求并阻塞等待决策。cancel 为 true 或超时则 Deny。
pub fn wait_for_approval(
    request_id: &str,
    conversation_id: &str,
    similar_key: &str,
    cancel: &AtomicBool,
) -> ApprovalDecision {
    if is_session_allowed(conversation_id, similar_key) {
        return ApprovalDecision::Allow;
    }

    let (tx, rx) = std::sync::mpsc::channel();
    {
        let mut g = state().lock().expect("approval lock");
        g.pending.insert(
            request_id.to_string(),
            Pending {
                tx,
                conversation_id: conversation_id.to_string(),
            },
        );
    }

    let deadline = std::time::Instant::now() + Duration::from_secs(300);
    loop {
        if cancel.load(Ordering::SeqCst) {
            let _ = resolve(request_id, ApprovalDecision::Deny);
            return ApprovalDecision::Deny;
        }
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(decision) => {
                if decision == ApprovalDecision::AllowSimilar {
                    if let Ok(mut g) = state().lock() {
                        g.session_allow
                            .entry(conversation_id.to_string())
                            .or_default()
                            .insert(similar_key.to_string());
                    }
                    return ApprovalDecision::Allow;
                }
                return decision;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if std::time::Instant::now() > deadline {
                    let _ = resolve(request_id, ApprovalDecision::Deny);
                    return ApprovalDecision::Deny;
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                return ApprovalDecision::Deny;
            }
        }
    }
}

pub fn resolve(request_id: &str, decision: ApprovalDecision) -> Result<(), String> {
    let mut g = state().lock().map_err(|e| e.to_string())?;
    let pending = g
        .pending
        .remove(request_id)
        .ok_or_else(|| format!("未知或已结束的审批请求: {request_id}"))?;
    pending
        .tx
        .send(decision)
        .map_err(|_| "审批等待方已断开".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn allow_similar_persists_in_session() {
        let cancel = AtomicBool::new(false);
        let conv = format!("test-conv-{}", uuid::Uuid::new_v4());
        let key = "rg";
        let req = format!("req-{}", uuid::Uuid::new_v4());

        let conv2 = conv.clone();
        let req2 = req.clone();
        let handle = thread::spawn(move || wait_for_approval(&req2, &conv2, key, &cancel));

        thread::sleep(Duration::from_millis(50));
        resolve(&req, ApprovalDecision::AllowSimilar).unwrap();
        assert_eq!(handle.join().unwrap(), ApprovalDecision::Allow);
        assert!(is_session_allowed(&conv, key));

        let cancel2 = AtomicBool::new(false);
        // Second wait should short-circuit without resolve
        assert_eq!(
            wait_for_approval("unused", &conv, key, &cancel2),
            ApprovalDecision::Allow
        );
        clear_session(&conv);
    }
}
