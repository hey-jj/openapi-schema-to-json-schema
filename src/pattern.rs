//! The default `patternProperties` handler.

use serde_json::Value;

/// Disallow extra properties when one already matches a pattern.
///
/// When `additionalProperties` is an object or array and equals one of the
/// `patternProperties` schemas, set `additionalProperties` to false. An
/// `additionalProperties` that is not an object short-circuits and returns the
/// schema unchanged, so a boolean `additionalProperties` survives.
pub(crate) fn default_pattern_properties_handler(schema: Value) -> Value {
    let mut schema = schema;
    let Value::Object(map) = &mut schema else {
        return schema;
    };

    // JS guard `typeof additProps !== "object"`. Objects and arrays pass; null
    // also passes the typeof check but never deep-equals a pattern value here.
    let addit = map.get("additionalProperties").cloned();
    let is_object_like = matches!(
        &addit,
        Some(Value::Object(_)) | Some(Value::Array(_)) | Some(Value::Null)
    );
    if !is_object_like {
        return schema;
    }
    let Some(addit) = addit else {
        return schema;
    };

    let matches_pattern = match map.get("patternProperties") {
        Some(Value::Object(patterns)) => patterns.values().any(|p| *p == addit),
        _ => false,
    };
    if matches_pattern {
        map.insert("additionalProperties".to_string(), Value::Bool(false));
    }
    schema
}
