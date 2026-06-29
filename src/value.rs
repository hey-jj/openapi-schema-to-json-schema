//! JSON value predicates shared across the converters.
//!
//! The conversion rules test values for truthiness and object-ness. One
//! definition of each rule keeps the modules in step.

use serde_json::Value;

/// Truthiness for a JSON value. False for null, false, 0, and empty string.
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

/// True for JSON objects and arrays. The conversion rules treat both as
/// container values that can hold sub-schemas, so arrays count here too.
pub(crate) fn is_object(value: &Value) -> bool {
    matches!(value, Value::Object(_) | Value::Array(_))
}
