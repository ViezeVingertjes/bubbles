//! Serialize [`DialogueEvent`] to JSON for the C ABI without adding serde to the core crate.

use bubbles::{DialogueEvent, MarkupSpan};
use serde_json::{Value, json};

fn markup_span_to_json(span: &MarkupSpan) -> Value {
    json!({
        "name": span.name,
        "start": span.start,
        "length": span.length,
        "properties": span.properties,
    })
}

/// Serializes a runtime event to a JSON object string for the C ABI.
#[must_use]
pub(crate) fn dialogue_event_to_json(ev: &DialogueEvent) -> String {
    match ev {
        DialogueEvent::NodeStarted(node) => json!({
            "kind": "NodeStarted",
            "node": node,
        })
        .to_string(),
        DialogueEvent::Line {
            speaker,
            text,
            line_id,
            tags,
            line_mode,
            spans,
        } => json!({
            "kind": "Line",
            "speaker": speaker,
            "text": text,
            "line_id": line_id,
            "tags": tags,
            "line_mode": match line_mode {
                bubbles::LineMode::Normal => "normal",
                bubbles::LineMode::Narration => "narration",
                bubbles::LineMode::Debug => "debug",
            },
            "spans": spans.iter().map(markup_span_to_json).collect::<Vec<_>>(),
        })
        .to_string(),
        DialogueEvent::Options(opts) => json!({
            "kind": "Options",
            "options": opts.iter().map(|o| {
                json!({
                    "text": o.text,
                    "available": o.available,
                    "line_id": o.line_id,
                    "tags": o.tags,
                    "group": o.group,
                    "spans": o.spans.iter().map(markup_span_to_json).collect::<Vec<_>>(),
                })
            }).collect::<Vec<_>>(),
        })
        .to_string(),
        DialogueEvent::Command { name, args, tags } => json!({
            "kind": "Command",
            "name": name,
            "args": args,
            "tags": tags,
        })
        .to_string(),
        DialogueEvent::NodeComplete(node) => json!({
            "kind": "NodeComplete",
            "node": node,
        })
        .to_string(),
        DialogueEvent::DialogueComplete => json!({ "kind": "DialogueComplete" }).to_string(),
        // `DialogueEvent` is `#[non_exhaustive]`; keep the C ABI forward-compatible.
        #[allow(unreachable_patterns)]
        _ => json!({ "kind": "Unknown" }).to_string(),
    }
}
