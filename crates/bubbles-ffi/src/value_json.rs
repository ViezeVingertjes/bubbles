//! JSON helpers for [`bubbles::Value`] in the C ABI.

use bubbles::Value;

/// Parses a [`Value`] from JSON (same encoding `serde_json` uses for `Value` with the `serde` feature).
pub(crate) fn value_from_json_slice(s: &[u8]) -> Result<Value, String> {
    let v: serde_json::Value =
        serde_json::from_slice(s).map_err(|e| format!("invalid value JSON: {e}"))?;
    json_to_value(&v)
}

fn json_to_value(v: &serde_json::Value) -> Result<Value, String> {
    match v {
        serde_json::Value::Null => Err("null is not a valid dialogue Value".into()),
        serde_json::Value::Bool(b) => Ok(Value::Bool(*b)),
        serde_json::Value::Number(n) => n
            .as_f64()
            .map(Value::Number)
            .ok_or_else(|| "number out of range".to_string()),
        serde_json::Value::String(s) => Ok(Value::Text(s.clone())),
        // Externally-tagged form produced by `Value`'s own `Serialize` impl.
        serde_json::Value::Object(_) => serde_json::from_value(v.clone())
            .map_err(|_| "unknown Value object shape (expected Number, Text, or Bool key)".into()),
        serde_json::Value::Array(_) => Err("arrays are not valid single Values".into()),
    }
}

/// Serializes a [`Value`] to a JSON string for host exchange.
pub(crate) fn value_to_json_string(v: &Value) -> Result<String, serde_json::Error> {
    serde_json::to_string(v)
}

/// Builds a JSON array of [`Value`] for host function arguments.
pub(crate) fn values_to_json_array(args: &[Value]) -> Result<String, serde_json::Error> {
    serde_json::to_string(args)
}
