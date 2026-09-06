//! Assistant-turn trajectory steps (ordered reasoning / tool / text).

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One ordered step in an assistant turn timeline.
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

/// Prefer parsed `trajectory_json`; otherwise synthesize from flat fields.
pub fn resolve_trajectory(
    trajectory_json: Option<&str>,
    reasoning: Option<&str>,
    tool_calls_json: Option<&str>,
    content: &str,
) -> Vec<TrajectoryStep> {
    if let Some(raw) = trajectory_json.map(str::trim).filter(|s| !s.is_empty()) {
        if let Ok(steps) = serde_json::from_str::<Vec<TrajectoryStep>>(raw) {
            if !steps.is_empty() {
                return steps;
            }
        }
    }
    synthesize_trajectory(reasoning, tool_calls_json, content)
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
    fn resolve_prefers_stored_trajectory() {
        let stored = json!([
            {"type": "text", "text": "先说一句"},
            {"type": "tool", "id": "uuid.generate", "args": {}, "status": "done", "result": "x"},
            {"type": "text", "text": "再说一句"}
        ])
        .to_string();
        let steps = resolve_trajectory(
            Some(&stored),
            Some("ignored"),
            Some("[]"),
            "ignored body",
        );
        assert_eq!(steps.len(), 3);
        assert!(matches!(&steps[0], TrajectoryStep::Text { text } if text == "先说一句"));
        assert!(matches!(&steps[1], TrajectoryStep::Tool { id, .. } if id == "uuid.generate"));
    }

    #[test]
    fn resolve_falls_back_when_trajectory_empty() {
        let steps = resolve_trajectory(Some(""), Some("r"), None, "body");
        assert_eq!(steps.len(), 2);
        assert!(matches!(&steps[0], TrajectoryStep::Reasoning { .. }));
        assert!(matches!(&steps[1], TrajectoryStep::Text { .. }));
    }
}
