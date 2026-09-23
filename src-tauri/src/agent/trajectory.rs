//! Assistant-turn trajectory.
//!
//! Persistence shape (`messages.trajectory_json`) is a FLAT array of
//! interaction events — one node per interaction, no nesting:
//!
//! ```json
//! [
//!   {"type":"model_call","id":"call1","reasoning":"...","requested_tools":["base64.encode"],"stream_mode":"fallback"},
//!   {"type":"tool_call","id":"base64.encode","args":{...},"result":"...","status":"done"},
//!   {"type":"model_call","id":"call2","response":"最终回复"}
//! ]
//! ```
//!
//! Interaction mapping (each is ONE node in the flat list):
//! - `model_call` = one agent↔model invocation (one loop iteration)
//! - `tool_call`  = one agent↔tool dispatch
//! - user↔agent is the outer conversation turn, rendered from the message
//!   list (not persisted inside `trajectory_json`).
//!
//! Legacy shapes (still readable, converted on read):
//! - nested dev format: `[{type:"model_call", tools:[...], response}]`
//! - legacy flat steps: `[{type:"reasoning"|"tool"|"text"}]`
//! - no stored trajectory: synthesized from `reasoning`/`tool_calls_json`/`content`

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Legacy flat step. Kept for backward compatibility when reading old
/// `trajectory_json` payloads and for in-loop streaming accumulation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TrajectoryStep {
    Reasoning { text: String },
    Tool {
        id: String,
        args: Value,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        result: Option<String>,
        status: String,
    },
    Text { text: String },
}

/// One interaction event. New persistence writes a flat `Vec<TrajectoryEvent>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TrajectoryEvent {
    /// agent ↔ model: one model invocation (one agent-loop iteration).
    ModelCall {
        /// Stable id within the turn (e.g. "call1", "call2").
        id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reasoning: Option<String>,
        /// Tool ids this call requested. The executions follow immediately
        /// as sibling `tool_call` events.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        requested_tools: Vec<String>,
        /// Final user-facing text; set only on the call that ended the turn.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        response: Option<String>,
        /// "live" / "fallback" — mirrors AgentEvent::StreamMeta.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        stream_mode: Option<String>,
    },
    /// agent ↔ tool: one tool dispatch.
    ToolCall {
        id: String,
        args: Value,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        result: Option<String>,
        status: String,
    },
}

/// Nested dev format (written briefly during development). Read-only compat.
#[derive(Debug, Deserialize)]
struct NestedTool {
    id: String,
    args: Value,
    #[serde(default)]
    result: Option<String>,
    status: String,
}

#[derive(Debug, Deserialize)]
struct NestedModelCall {
    #[serde(default)]
    id: String,
    #[serde(default)]
    reasoning: Option<String>,
    #[serde(default)]
    tools: Vec<NestedTool>,
    #[serde(default)]
    response: Option<String>,
    #[serde(default)]
    stream_mode: Option<String>,
}

/// Build a display trajectory from flat message fields when `trajectory_json` is absent.
///
/// Order: reasoning → tools → text (legacy messages never interleaved mid-turn).
pub fn synthesize_trajectory(
    reasoning: Option<&str>,
    tool_calls_json: Option<&str>,
    content: &str,
) -> Vec<TrajectoryStep> {
    let mut steps = Vec::new();
    if let Some(r) = reasoning.map(str::trim).filter(|s| !s.is_empty()) {
        steps.push(TrajectoryStep::Reasoning {
            text: r.to_string(),
        });
    }
    if let Some(raw) = tool_calls_json.map(str::trim).filter(|s| !s.is_empty()) {
        if let Ok(Value::Array(arr)) = serde_json::from_str::<Value>(raw) {
            for item in arr {
                let id = item
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("tool")
                    .to_string();
                let args = item.get("args").cloned().unwrap_or(Value::Null);
                let result = item
                    .get("result")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let status = item
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or(if result.is_some() { "done" } else { "running" })
                    .to_string();
                steps.push(TrajectoryStep::Tool {
                    id,
                    args,
                    result,
                    status,
                });
            }
        }
    }
    let body = content.trim();
    if !body.is_empty() {
        steps.push(TrajectoryStep::Text {
            text: body.to_string(),
        });
    }
    steps
}

/// Streaming-event builder that converts a flat `Vec<TrajectoryStep>` (+
/// iteration boundaries) into flat `Vec<TrajectoryEvent>`.
///
/// `boundaries` MUST be the `trajectory.len()` snapshot taken at the start of
/// each loop iteration (ascending, including 0 for the first call). Each
/// window becomes one `model_call` event followed by its `tool_call` events.
pub fn build_trajectory_events_from_steps(
    steps: &[TrajectoryStep],
    boundaries: &[usize],
    stream_mode: Option<&str>,
) -> Vec<TrajectoryEvent> {
    if steps.is_empty() {
        return Vec::new();
    }
    let mut cuts: Vec<usize> = Vec::with_capacity(boundaries.len() + 2);
    cuts.push(0);
    for b in boundaries {
        let clamped = (*b).min(steps.len());
        if clamped >= *cuts.last().unwrap() {
            cuts.push(clamped);
        }
    }
    if *cuts.last().unwrap() != steps.len() {
        cuts.push(steps.len());
    }
    cuts.dedup();

    let mut events: Vec<TrajectoryEvent> = Vec::new();
    for window in cuts.windows(2) {
        let slice = &steps[window[0]..window[1]];
        let mut reasoning = String::new();
        let mut tools: Vec<TrajectoryEvent> = Vec::new();
        let mut tool_ids: Vec<String> = Vec::new();
        let mut response = String::new();
        for step in slice {
            match step {
                TrajectoryStep::Reasoning { text } => {
                    if !reasoning.is_empty() {
                        reasoning.push('\n');
                    }
                    reasoning.push_str(text);
                }
                TrajectoryStep::Tool {
                    id,
                    args,
                    result,
                    status,
                } => {
                    tool_ids.push(id.clone());
                    tools.push(TrajectoryEvent::ToolCall {
                        id: id.clone(),
                        args: args.clone(),
                        result: result.clone(),
                        status: status.clone(),
                    });
                }
                TrajectoryStep::Text { text } => {
                    if !response.is_empty() {
                        response.push('\n');
                    }
                    response.push_str(text);
                }
            }
        }
        let call_no = events
            .iter()
            .filter(|e| matches!(e, TrajectoryEvent::ModelCall { .. }))
            .count()
            + 1;
        events.push(TrajectoryEvent::ModelCall {
            id: format!("call{call_no}"),
            reasoning: if reasoning.is_empty() { None } else { Some(reasoning) },
            requested_tools: tool_ids,
            response: if response.is_empty() { None } else { Some(response) },
            stream_mode: stream_mode.map(|s| s.to_string()),
        });
        events.extend(tools);
    }
    events
}

/// Incremental state machine converting legacy flat steps into events.
/// Heuristic: each `reasoning` starts a new model call; `tool` steps attach to
/// the current call (emitted as siblings after it); a trailing `text` becomes
/// the final call's response (or folds into the current call when it has no
/// tools).
struct EventBuilder {
    events: Vec<TrajectoryEvent>,
    call_no: usize,
    cur_reasoning: Option<String>,
    cur_started: bool,
    cur_tools: Vec<TrajectoryEvent>,
    cur_tool_ids: Vec<String>,
    stream_mode: Option<String>,
}

impl EventBuilder {
    fn new(stream_mode: Option<&str>) -> Self {
        Self {
            events: Vec::new(),
            call_no: 0,
            cur_reasoning: None,
            cur_started: false,
            cur_tools: Vec::new(),
            cur_tool_ids: Vec::new(),
            stream_mode: stream_mode.map(|s| s.to_string()),
        }
    }

    fn start_call(&mut self, reasoning: Option<String>) {
        self.flush();
        self.cur_started = true;
        self.cur_reasoning = reasoning;
    }

    fn flush(&mut self) {
        if self.cur_started {
            self.call_no += 1;
            let reasoning = self
                .cur_reasoning
                .take()
                .filter(|s| !s.trim().is_empty());
            self.events.push(TrajectoryEvent::ModelCall {
                id: format!("call{}", self.call_no),
                reasoning,
                requested_tools: std::mem::take(&mut self.cur_tool_ids),
                response: None,
                stream_mode: self.stream_mode.clone(),
            });
            let tools = std::mem::take(&mut self.cur_tools);
            self.events.extend(tools);
            self.cur_started = false;
        }
    }

    fn push_tool(&mut self, id: &str, args: &Value, result: &Option<String>, status: &str) {
        if !self.cur_started {
            self.cur_started = true;
        }
        self.cur_tool_ids.push(id.to_string());
        self.cur_tools.push(TrajectoryEvent::ToolCall {
            id: id.to_string(),
            args: args.clone(),
            result: result.clone(),
            status: status.to_string(),
        });
    }

    fn push_text(&mut self, text: &str) {
        let has_tools = !self.cur_tools.is_empty();
        let reasoning = if has_tools {
            self.flush();
            None
        } else {
            self.cur_reasoning
                .take()
                .filter(|s| !s.trim().is_empty())
        };
        self.call_no += 1;
        self.events.push(TrajectoryEvent::ModelCall {
            id: format!("call{}", self.call_no),
            reasoning,
            requested_tools: Vec::new(),
            response: Some(text.to_string()).filter(|s| !s.trim().is_empty()),
            stream_mode: self.stream_mode.clone(),
        });
        self.cur_started = false;
    }
}

fn steps_to_events(steps: &[TrajectoryStep], stream_mode: Option<&str>) -> Vec<TrajectoryEvent> {
    let mut b = EventBuilder::new(stream_mode);
    for step in steps {
        match step {
            TrajectoryStep::Reasoning { text } => {
                b.start_call(Some(text.clone()));
            }
            TrajectoryStep::Tool {
                id,
                args,
                result,
                status,
            } => b.push_tool(id, args, result, status),
            TrajectoryStep::Text { text } => b.push_text(text),
        }
    }
    b.flush();
    b.events
}

fn nested_to_events(arr: &[Value]) -> Option<Vec<TrajectoryEvent>> {
    let mut events = Vec::new();
    for (i, v) in arr.iter().enumerate() {
        if v.get("type").and_then(|t| t.as_str()) != Some("model_call") {
            continue;
        }
        let nested: NestedModelCall = serde_json::from_value(v.clone()).ok()?;
        let id = if nested.id.is_empty() {
            format!("call{}", i + 1)
        } else {
            nested.id
        };
        events.push(TrajectoryEvent::ModelCall {
            id,
            reasoning: nested.reasoning.filter(|s| !s.trim().is_empty()),
            requested_tools: nested.tools.iter().map(|t| t.id.clone()).collect(),
            response: nested.response.filter(|s| !s.trim().is_empty()),
            stream_mode: nested.stream_mode,
        });
        for t in nested.tools {
            events.push(TrajectoryEvent::ToolCall {
                id: t.id,
                args: t.args,
                result: t.result,
                status: t.status,
            });
        }
    }
    if events.is_empty() {
        None
    } else {
        Some(events)
    }
}

fn value_array_to_events(arr: &[Value], stream_mode: Option<&str>) -> Option<Vec<TrajectoryEvent>> {
    let first = arr.first()?;
    let ty = first.get("type")?.as_str()?;
    match ty {
        "model_call" => {
            // Nested dev format carries a `tools` array of objects.
            let is_nested = first
                .get("tools")
                .map(|t| t.is_array())
                .unwrap_or(false);
            if is_nested {
                nested_to_events(arr)
            } else {
                // New flat format.
                let mut events = Vec::with_capacity(arr.len());
                for (i, v) in arr.iter().enumerate() {
                    let ty = v.get("type").and_then(|t| t.as_str())?;
                    match ty {
                        "model_call" => {
                            let id = v
                                .get("id")
                                .and_then(|x| x.as_str())
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| format!("call{}", i + 1));
                            events.push(TrajectoryEvent::ModelCall {
                                id,
                                reasoning: v
                                    .get("reasoning")
                                    .and_then(|x| x.as_str())
                                    .map(|s| s.to_string()),
                                requested_tools: v
                                    .get("requested_tools")
                                    .and_then(|x| x.as_array())
                                    .map(|a| {
                                        a.iter()
                                            .filter_map(|x| x.as_str().map(|s| s.to_string()))
                                            .collect()
                                    })
                                    .unwrap_or_default(),
                                response: v
                                    .get("response")
                                    .and_then(|x| x.as_str())
                                    .map(|s| s.to_string()),
                                stream_mode: v
                                    .get("stream_mode")
                                    .and_then(|x| x.as_str())
                                    .map(|s| s.to_string()),
                            });
                        }
                        "tool_call" => {
                            events.push(TrajectoryEvent::ToolCall {
                                id: v.get("id").and_then(|x| x.as_str())?.to_string(),
                                args: v.get("args").cloned().unwrap_or(Value::Null),
                                result: v
                                    .get("result")
                                    .and_then(|x| x.as_str())
                                    .map(|s| s.to_string()),
                                status: v
                                    .get("status")
                                    .and_then(|x| x.as_str())
                                    .unwrap_or("done")
                                    .to_string(),
                            });
                        }
                        _ => return None,
                    }
                }
                Some(events)
            }
        }
        "reasoning" | "tool" | "text" => {
            // Legacy flat steps.
            let mut steps = Vec::with_capacity(arr.len());
            for v in arr {
                let ty = v.get("type").and_then(|t| t.as_str())?;
                match ty {
                    "reasoning" => steps.push(TrajectoryStep::Reasoning {
                        text: v.get("text").and_then(|x| x.as_str())?.to_string(),
                    }),
                    "tool" => steps.push(TrajectoryStep::Tool {
                        id: v.get("id").and_then(|x| x.as_str())?.to_string(),
                        args: v.get("args").cloned().unwrap_or(Value::Null),
                        result: v
                            .get("result")
                            .and_then(|x| x.as_str())
                            .map(|s| s.to_string()),
                        status: v
                            .get("status")
                            .and_then(|x| x.as_str())
                            .unwrap_or("done")
                            .to_string(),
                    }),
                    "text" => steps.push(TrajectoryStep::Text {
                        text: v.get("text").and_then(|x| x.as_str())?.to_string(),
                    }),
                    _ => return None,
                }
            }
            if steps.is_empty() {
                return None;
            }
            Some(steps_to_events(&steps, stream_mode))
        }
        _ => None,
    }
}

/// Resolve the trajectory for one assistant message into a flat
/// `Vec<TrajectoryEvent>` (one node per interaction).
///
/// Precedence:
/// 1. `trajectory_json` in new flat event format → use it.
/// 2. Nested dev format (`model_call` with `tools` array) → flatten.
/// 3. Legacy flat steps (`reasoning`/`tool`/`text`) → heuristic conversion.
/// 4. Synthesize from `reasoning` / `tool_calls_json` / `content`.
pub fn resolve_trajectory(
    trajectory_json: Option<&str>,
    reasoning: Option<&str>,
    tool_calls_json: Option<&str>,
    content: &str,
    stream_mode: Option<&str>,
) -> Vec<TrajectoryEvent> {
    if let Some(raw) = trajectory_json.map(str::trim).filter(|s| !s.is_empty()) {
        if let Ok(Value::Array(arr)) = serde_json::from_str::<Value>(raw) {
            if !arr.is_empty() {
                if let Some(events) = value_array_to_events(&arr, stream_mode) {
                    return events;
                }
            }
        }
    }
    let synth = synthesize_trajectory(reasoning, tool_calls_json, content);
    if synth.is_empty() {
        return Vec::new();
    }
    steps_to_events(&synth, stream_mode)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn synthesize_orders_reasoning_tools_text() {
        let tools = json!([
            {
                "id": "base64.encode",
                "args": {"input": "hi"},
                "result": "aGk=",
                "status": "done"
            }
        ])
        .to_string();
        let steps = synthesize_trajectory(Some("想一想"), Some(&tools), "结果是 aGk=");
        assert_eq!(steps.len(), 3);
        assert!(matches!(&steps[0], TrajectoryStep::Reasoning { text } if text == "想一想"));
        assert!(matches!(
            &steps[1],
            TrajectoryStep::Tool { id, status, .. } if id == "base64.encode" && status == "done"
        ));
        assert!(matches!(&steps[2], TrajectoryStep::Text { text } if text == "结果是 aGk="));
    }

    #[test]
    fn build_with_boundaries_emits_flat_model_then_tools() {
        // Iteration 1: reasoning + 1 tool. Iteration 2: reasoning + text.
        let steps = vec![
            TrajectoryStep::Reasoning { text: "step 1".into() },
            TrajectoryStep::Tool {
                id: "base64.encode".into(),
                args: json!({"input": "hi"}),
                result: Some("aGk=".into()),
                status: "done".into(),
            },
            TrajectoryStep::Reasoning { text: "step 2".into() },
            TrajectoryStep::Text { text: "done".into() },
        ];
        let events = build_trajectory_events_from_steps(&steps, &[0, 2], Some("fallback"));
        // Expect: MC1, TC, MC2  (flat, no nesting)
        assert_eq!(events.len(), 3);
        match &events[0] {
            TrajectoryEvent::ModelCall {
                id,
                reasoning,
                requested_tools,
                response,
                stream_mode,
            } => {
                assert_eq!(id, "call1");
                assert_eq!(reasoning.as_deref(), Some("step 1"));
                assert_eq!(requested_tools, &vec!["base64.encode".to_string()]);
                assert!(response.is_none());
                assert_eq!(stream_mode.as_deref(), Some("fallback"));
            }
            e => panic!("expected model_call, got {e:?}"),
        }
        match &events[1] {
            TrajectoryEvent::ToolCall { id, status, .. } => {
                assert_eq!(id, "base64.encode");
                assert_eq!(status, "done");
            }
            e => panic!("expected tool_call, got {e:?}"),
        }
        match &events[2] {
            TrajectoryEvent::ModelCall {
                id,
                reasoning,
                requested_tools,
                response,
                ..
            } => {
                assert_eq!(id, "call2");
                assert_eq!(reasoning.as_deref(), Some("step 2"));
                assert!(requested_tools.is_empty());
                assert_eq!(response.as_deref(), Some("done"));
            }
            e => panic!("expected model_call, got {e:?}"),
        }
    }

    #[test]
    fn steps_to_events_reasoning_then_text_folds_into_one_call() {
        let steps = vec![
            TrajectoryStep::Reasoning { text: "想".into() },
            TrajectoryStep::Text { text: "答".into() },
        ];
        let events = steps_to_events(&steps, None);
        assert_eq!(events.len(), 1);
        match &events[0] {
            TrajectoryEvent::ModelCall { reasoning, response, .. } => {
                assert_eq!(reasoning.as_deref(), Some("想"));
                assert_eq!(response.as_deref(), Some("答"));
            }
            e => panic!("expected model_call, got {e:?}"),
        }
    }

    #[test]
    fn steps_to_events_interleaved_reasoning_starts_new_call() {
        let steps = vec![
            TrajectoryStep::Reasoning { text: "r1".into() },
            TrajectoryStep::Tool {
                id: "t1".into(),
                args: json!({}),
                result: Some("x".into()),
                status: "done".into(),
            },
            TrajectoryStep::Reasoning { text: "r2".into() },
            TrajectoryStep::Tool {
                id: "t2".into(),
                args: json!({}),
                result: Some("y".into()),
                status: "done".into(),
            },
            TrajectoryStep::Text { text: "final".into() },
        ];
        let events = steps_to_events(&steps, None);
        // MC1, TC1, MC2, TC2, MC3(response)
        assert_eq!(events.len(), 5);
        assert!(matches!(&events[0], TrajectoryEvent::ModelCall { reasoning, .. } if reasoning.as_deref() == Some("r1")));
        assert!(matches!(&events[1], TrajectoryEvent::ToolCall { id, .. } if id == "t1"));
        assert!(matches!(&events[2], TrajectoryEvent::ModelCall { reasoning, .. } if reasoning.as_deref() == Some("r2")));
        assert!(matches!(&events[3], TrajectoryEvent::ToolCall { id, .. } if id == "t2"));
        assert!(matches!(&events[4], TrajectoryEvent::ModelCall { response, .. } if response.as_deref() == Some("final")));
    }

    #[test]
    fn resolve_prefers_new_flat_format() {
        let stored = json!([
            {"type": "model_call", "id": "call1", "reasoning": "think", "requested_tools": ["uuid.generate"], "stream_mode": "live"},
            {"type": "tool_call", "id": "uuid.generate", "args": {}, "result": "x", "status": "done"},
            {"type": "model_call", "id": "call2", "response": "answer"}
        ])
        .to_string();
        let events = resolve_trajectory(Some(&stored), None, None, "", None);
        assert_eq!(events.len(), 3);
        assert!(matches!(&events[0], TrajectoryEvent::ModelCall { id, .. } if id == "call1"));
        assert!(matches!(&events[1], TrajectoryEvent::ToolCall { id, .. } if id == "uuid.generate"));
        assert!(matches!(&events[2], TrajectoryEvent::ModelCall { response, .. } if response.as_deref() == Some("answer")));
    }

    #[test]
    fn resolve_flattens_nested_dev_format() {
        let stored = json!([
            {
                "type": "model_call",
                "id": "call1",
                "reasoning": "think",
                "tools": [{"id": "uuid.generate", "args": {}, "result": "x", "status": "done"}],
                "stream_mode": "live"
            },
            {
                "type": "model_call",
                "id": "call2",
                "tools": [],
                "response": "answer"
            }
        ])
        .to_string();
        let events = resolve_trajectory(Some(&stored), None, None, "", None);
        // Nested MC(tools) → MC + TC
        assert_eq!(events.len(), 3);
        assert!(matches!(&events[0], TrajectoryEvent::ModelCall { id, requested_tools, .. }
            if id == "call1" && requested_tools == &vec!["uuid.generate".to_string()]));
        assert!(matches!(&events[1], TrajectoryEvent::ToolCall { id, .. } if id == "uuid.generate"));
        assert!(matches!(&events[2], TrajectoryEvent::ModelCall { response, .. } if response.as_deref() == Some("answer")));
    }

    #[test]
    fn resolve_falls_back_to_legacy_flat_steps() {
        let stored = json!([
            {"type": "reasoning", "text": "先想"},
            {"type": "tool", "id": "uuid.generate", "args": {}, "status": "done", "result": "x"},
            {"type": "text", "text": "回答"}
        ])
        .to_string();
        let events = resolve_trajectory(Some(&stored), None, None, "", None);
        // Legacy heuristic: MC(reasoning+requested) + TC + MC(response)
        assert_eq!(events.len(), 3);
        assert!(matches!(&events[0], TrajectoryEvent::ModelCall { reasoning, requested_tools, .. }
            if reasoning.as_deref() == Some("先想") && requested_tools.len() == 1));
        assert!(matches!(&events[1], TrajectoryEvent::ToolCall { id, .. } if id == "uuid.generate"));
        assert!(matches!(&events[2], TrajectoryEvent::ModelCall { response, .. } if response.as_deref() == Some("回答")));
    }

    #[test]
    fn resolve_synthesizes_when_no_stored() {
        let events = resolve_trajectory(
            None,
            Some("think"),
            Some("[{\"id\":\"x\",\"args\":{}}]"),
            "ok",
            Some("fallback"),
        );
        assert_eq!(events.len(), 3);
        assert!(matches!(&events[0], TrajectoryEvent::ModelCall { reasoning, .. } if reasoning.as_deref() == Some("think")));
        assert!(matches!(&events[1], TrajectoryEvent::ToolCall { id, .. } if id == "x"));
        assert!(matches!(&events[2], TrajectoryEvent::ModelCall { response, .. } if response.as_deref() == Some("ok")));
    }
}
