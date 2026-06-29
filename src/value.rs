//! JSON value predicates shared across the converters.
//!
//! These match the JavaScript semantics the conversion rules are written
//! against. Keeping one definition of each rule stops the modules from drifting
//! apart.

use serde_json::Value;

/// JS truthiness for a JSON value. False for null, false, 0, and empty string.
/// True for every other number, every non-empty string, and any object or
/// array.
pub(crate) fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(true),
        Value::String(s) => !s.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

/// The inverse of [`is_truthy`]. True for null, false, 0, and empty string.
pub(crate) fn is_falsy(value: &Value) -> bool {
    !is_truthy(value)
}

/// True for JSON objects and arrays, matching the JS `isObject` helper
/// (`maybeObj !== null && typeof maybeObj === "object"`). Arrays count as
/// objects in JavaScript, so they count here too.
pub(crate) fn is_object(value: &Value) -> bool {
    matches!(value, Value::Object(_) | Value::Array(_))
}
