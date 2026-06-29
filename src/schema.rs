//! The recursive schema converter.

use serde_json::{Map, Number, Value};

use crate::consts::{BYTE_PATTERN, DRAFT_04, VALID_OPENAPI_FORMATS, VALID_TYPES};
use crate::error::Error;
use crate::options::ResolvedOptions;
use crate::pattern::default_pattern_properties_handler;

/// True for JSON objects and arrays, matching the source `isObject` helper
/// (`maybeObj !== null && typeof maybeObj === "object"`).
fn is_object(value: &Value) -> bool {
    matches!(value, Value::Object(_) | Value::Array(_))
}

/// Convert a schema and write `$schema` on the root.
///
/// This is the entry point. It runs the recursive converter then sets the
/// draft-04 `$schema` on whatever object the recursion returns.
pub(crate) fn convert_from_schema(
    schema: Value,
    options: &ResolvedOptions,
) -> Result<Value, Error> {
    let mut new_schema = convert_schema(schema, options)?;
    if let Value::Object(map) = &mut new_schema {
        map.insert("$schema".to_string(), Value::String(DRAFT_04.to_string()));
    }
    Ok(new_schema)
}

/// Convert one node. Recurses into struct keywords and properties.
///
/// Step order matches the source: before-transform hook, struct recursion,
/// definition keywords, properties and required pruning, type validation,
/// nullable handling, format handling, pattern properties, keyword stripping,
/// after-transform hook.
fn convert_schema(schema: Value, options: &ResolvedOptions) -> Result<Value, Error> {
    let mut schema = schema;

    if let Some(hook) = &options.before_transform {
        schema = hook(schema, options);
    }

    // Only objects carry the keywords below. Anything else passes through.
    let Value::Object(_) = &schema else {
        return Ok(maybe_after_transform(schema, options));
    };

    recurse_structs(&mut schema, options)?;
    convert_definition_keywords(&mut schema, options)?;
    convert_properties_and_required(&mut schema, options)?;

    if options.strict_mode {
        if let Value::Object(map) = &schema {
            if let Some(type_value) = map.get("type") {
                validate_type(type_value)?;
            }
        }
    }

    convert_types(&mut schema);
    convert_format(&mut schema, options);

    if options.support_pattern_properties {
        let has_xpattern =
            matches!(&schema, Value::Object(m) if m.contains_key("x-patternProperties"));
        if has_xpattern {
            schema = convert_pattern_properties(schema, options);
        }
    }

    if let Value::Object(map) = &mut schema {
        for keyword in &options.not_supported {
            map.remove(*keyword);
        }
    }

    Ok(maybe_after_transform(schema, options))
}

/// Run the after-transform hook if present, otherwise return the node.
fn maybe_after_transform(schema: Value, options: &ResolvedOptions) -> Value {
    match &options.after_transform {
        Some(hook) => hook(schema, options),
        None => schema,
    }
}

/// Recurse into the struct keywords (`allOf`, `anyOf`, `oneOf`, `not`, `items`,
/// `additionalProperties`).
///
/// Array values: convert object elements, drop non-object elements, keep order.
/// Null values: remove the key. Object values: convert in place. Scalars are
/// left untouched, so boolean `additionalProperties` survives.
fn recurse_structs(schema: &mut Value, options: &ResolvedOptions) -> Result<(), Error> {
    for keyword in options.structs() {
        let Value::Object(map) = schema else {
            return Ok(());
        };
        let Some(entry) = map.get_mut(*keyword) else {
            continue;
        };

        if let Value::Array(items) = entry {
            let mut converted = Vec::with_capacity(items.len());
            for item in items.drain(..) {
                if is_object(&item) {
                    converted.push(convert_schema(item, options)?);
                }
                // Non-object elements are dropped.
            }
            *entry = Value::Array(converted);
        } else if entry.is_null() {
            map.remove(*keyword);
        } else if entry.is_object() {
            let taken = std::mem::replace(entry, Value::Null);
            *entry = convert_schema(taken, options)?;
        }
        // Scalars (bool, number, string) pass through untouched.
    }
    Ok(())
}

/// Convert sub-schemas named under each definition keyword path.
///
/// Each keyword is a dotted path resolved with lodash-style get and set. The
/// value at the path, when an object or array, is run through
/// [`convert_properties`] and written back.
fn convert_definition_keywords(schema: &mut Value, options: &ResolvedOptions) -> Result<(), Error> {
    for path in &options.definition_keywords {
        let inner = lodash_get(schema, path);
        // JS uses `typeof innerDef === "object"`, true for arrays and null.
        // A missing path yields undefined and is skipped.
        match inner {
            Some(value) if value.is_object() || value.is_array() || value.is_null() => {
                let converted = convert_properties(value, options)?;
                lodash_set(schema, path, converted);
            }
            _ => {}
        }
    }
    Ok(())
}

/// Convert `properties` and prune `required`.
///
/// `properties` is replaced by the converted map. A non-object or null value
/// converts to an empty map, which is then deleted. `required` keeps only names
/// still present in the converted properties. An empty `required` or empty
/// `properties` is deleted.
fn convert_properties_and_required(
    schema: &mut Value,
    options: &ResolvedOptions,
) -> Result<(), Error> {
    let Value::Object(map) = schema else {
        return Ok(());
    };
    if !map.contains_key("properties") {
        return Ok(());
    }

    let props_value = map.remove("properties").unwrap_or(Value::Null);
    let converted = convert_properties(props_value, options)?;
    map.insert("properties".to_string(), converted);

    // Prune required against the converted properties. Only an array required
    // is touched; any other shape is left as is.
    if let Some(Value::Array(required)) = map.get("required").cloned() {
        let prop_names: Vec<String> = match map.get("properties") {
            Some(Value::Object(p)) => p.keys().cloned().collect(),
            _ => Vec::new(),
        };
        let filtered: Vec<Value> = required
            .into_iter()
            .filter(|key| matches!(key, Value::String(name) if prop_names.contains(name)))
            .collect();
        if filtered.is_empty() {
            map.remove("required");
        } else {
            map.insert("required".to_string(), Value::Array(filtered));
        }
    }

    let props_empty = matches!(map.get("properties"), Some(Value::Object(p)) if p.is_empty());
    if props_empty {
        map.remove("properties");
    }
    Ok(())
}

/// Convert a `properties` map. Returns a fresh object.
///
/// Non-object property values are dropped. Properties flagged with a removal
/// keyword (`readOnly`/`writeOnly` when the matching option is on) are dropped.
/// A scalar or null input converts to an empty object. An array input is walked
/// by index, matching the source `for ... in` over an array, so object elements
/// land under their numeric string keys.
fn convert_properties(value: Value, options: &ResolvedOptions) -> Result<Value, Error> {
    let mut out = Map::new();

    // Build the (key, value) pairs to walk. The source guard
    // `!isObject(properties) || !properties` returns early for scalars and null.
    // Objects walk their keys. Arrays are objects in JS, so they walk index
    // keys.
    let entries: Vec<(String, Value)> = match value {
        Value::Object(props) => props.into_iter().collect(),
        Value::Array(items) => items
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i.to_string(), v))
            .collect(),
        _ => return Ok(Value::Object(out)),
    };

    for (key, property) in entries {
        if !is_object(&property) {
            continue;
        }
        let remove = options.remove_props.iter().any(|flag| {
            matches!(&property, Value::Object(m) if m.get(*flag) == Some(&Value::Bool(true)))
        });
        if remove {
            continue;
        }
        out.insert(key, convert_schema(property, options)?);
    }
    Ok(Value::Object(out))
}

/// Reject `type` values outside the draft-04 set.
///
/// Only truthy values are checked. A falsy value (empty string, 0, false, null)
/// passes. An array value never equals a valid type string, so it is rejected.
fn validate_type(type_value: &Value) -> Result<(), Error> {
    if is_falsy(type_value) {
        return Ok(());
    }
    let valid = matches!(type_value, Value::String(s) if VALID_TYPES.contains(&s.as_str()));
    if valid {
        Ok(())
    } else {
        let rendered = serde_json::to_string(type_value).unwrap_or_default();
        Err(Error::InvalidType(format!(
            "Type {rendered} is not a valid type"
        )))
    }
}

/// JS truthiness for a JSON value. False for null, false, 0, empty string.
fn is_falsy(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::Bool(b) => !b,
        Value::Number(n) => n.as_f64().map(|f| f == 0.0).unwrap_or(false),
        Value::String(s) => s.is_empty(),
        _ => false,
    }
}

/// Apply `nullable` handling. When `nullable` is exactly true and a `type` is
/// present, replace the scalar type with `[type, "null"]` and append `null` to
/// an array `enum` that does not already contain it.
fn convert_types(schema: &mut Value) {
    let Value::Object(map) = schema else {
        return;
    };
    if !map.contains_key("type") {
        return;
    }
    let nullable = map.get("nullable") == Some(&Value::Bool(true));
    if !nullable {
        return;
    }
    // JS checks `type !== undefined`. A present `type: null` still proceeds,
    // since null is not undefined.
    let Some(type_value) = map.get("type").cloned() else {
        return;
    };

    map.insert(
        "type".to_string(),
        Value::Array(vec![type_value, Value::String("null".to_string())]),
    );

    if let Some(Value::Array(items)) = map.get("enum") {
        if !items.iter().any(|v| v.is_null()) {
            let mut new_enum = items.clone();
            new_enum.push(Value::Null);
            map.insert("enum".to_string(), Value::Array(new_enum));
        }
    }
}

/// Apply format conversions for numeric, byte, and date formats.
///
/// Formats in `VALID_OPENAPI_FORMATS` and absent formats pass through. `date`
/// becomes `date-time` only when `date_to_date_time` is on. `int32`, `int64`,
/// `float`, `double` inject `minimum`/`maximum` bounds. `byte` sets a base64
/// pattern. Unknown formats pass through.
fn convert_format(schema: &mut Value, options: &ResolvedOptions) {
    let Value::Object(map) = schema else {
        return;
    };
    let format = match map.get("format") {
        Some(Value::String(s)) => s.clone(),
        _ => return,
    };

    if VALID_OPENAPI_FORMATS.contains(&format.as_str()) {
        return;
    }

    if format == "date" && options.date_to_date_time {
        map.insert("format".to_string(), Value::String("date-time".to_string()));
        return;
    }

    // Bounds computed as f64, matching the source `2 ** n` arithmetic. Large
    // bounds are not exact integers, so a float Value is what JS produces.
    match format.as_str() {
        "int32" => clamp_bounds(map, -(2f64.powi(31)), 2f64.powi(31) - 1.0),
        "int64" => clamp_bounds(map, -(2f64.powi(63)), 2f64.powi(63) - 1.0),
        "float" => clamp_bounds(map, -(2f64.powi(128)), 2f64.powi(128) - 1.0),
        "double" => clamp_bounds(map, -f64::MAX, f64::MAX),
        "byte" => {
            map.insert(
                "pattern".to_string(),
                Value::String(BYTE_PATTERN.to_string()),
            );
        }
        _ => {}
    }
}

/// Set or clamp `minimum` and `maximum` for a numeric format.
///
/// For each bound the source guard is `(!value && value !== 0) || out of range`.
/// A missing, null, or non-number value, or any falsy value other than 0, takes
/// the bound. A present 0 is kept. A present number is clamped only when out of
/// range.
fn clamp_bounds(map: &mut Map<String, Value>, min: f64, max: f64) {
    let current_min = number_field(map, "minimum");
    if falsy_not_zero(current_min) || current_min.map(|v| v < min).unwrap_or(false) {
        map.insert("minimum".to_string(), float_value(min));
    }

    let current_max = number_field(map, "maximum");
    if falsy_not_zero(current_max) || current_max.map(|v| v > max).unwrap_or(false) {
        map.insert("maximum".to_string(), float_value(max));
    }
}

/// Read a numeric field as f64. Returns None when missing or not a number.
fn number_field(map: &Map<String, Value>, key: &str) -> Option<f64> {
    match map.get(key) {
        Some(Value::Number(n)) => n.as_f64(),
        _ => None,
    }
}

/// True when the value is falsy and not exactly 0, matching `!v && v !== 0`.
///
/// A missing or non-number field counts as falsy and not 0, so it takes the
/// bound. A present 0 is not falsy-and-not-zero, so it is kept. Among present
/// numbers only NaN is both falsy and not equal to 0, but NaN cannot appear in
/// JSON input.
fn falsy_not_zero(value: Option<f64>) -> bool {
    match value {
        None => true,
        Some(v) => v.is_nan(),
    }
}

/// Build a float-valued JSON number so impl and goldens use the same Number
/// representation. `from_f64` only fails for NaN or infinity, which the bounds
/// never produce.
fn float_value(v: f64) -> Value {
    Number::from_f64(v)
        .map(Value::Number)
        .unwrap_or(Value::Null)
}

/// Move `x-patternProperties` to `patternProperties` and run the handler.
///
/// When `x-patternProperties` is an object or array, copy it to
/// `patternProperties`. Always delete `x-patternProperties`. Then run the
/// configured handler or the default and return its result.
fn convert_pattern_properties(schema: Value, options: &ResolvedOptions) -> Value {
    let mut schema = schema;
    if let Value::Object(map) = &mut schema {
        let xpattern = map.remove("x-patternProperties");
        if let Some(value) = xpattern {
            if is_object(&value) {
                map.insert("patternProperties".to_string(), value);
            }
        }
    }
    match &options.pattern_properties_handler {
        Some(handler) => handler(schema),
        None => default_pattern_properties_handler(schema),
    }
}

// ---- lodash get/set over dotted paths -------------------------------------

/// Resolve a dotted path against a value. Returns a clone of the value at the
/// path, or None when any segment is missing or not an object.
fn lodash_get(root: &Value, path: &str) -> Option<Value> {
    let mut current = root;
    for segment in path.split('.') {
        match current {
            Value::Object(map) => {
                current = map.get(segment)?;
            }
            _ => return None,
        }
    }
    Some(current.clone())
}

/// Write a value at a dotted path, creating intermediate objects as needed.
fn lodash_set(root: &mut Value, path: &str, value: Value) {
    let segments: Vec<&str> = path.split('.').collect();
    set_recursive(root, &segments, value);
}

fn set_recursive(node: &mut Value, segments: &[&str], value: Value) {
    let (head, rest) = match segments.split_first() {
        Some(parts) => parts,
        None => return,
    };
    if !node.is_object() {
        *node = Value::Object(Map::new());
    }
    let Value::Object(map) = node else {
        return;
    };
    if rest.is_empty() {
        map.insert((*head).to_string(), value);
        return;
    }
    let child = map
        .entry((*head).to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    set_recursive(child, rest, value);
}

#[cfg(test)]
mod guard_tests {
    use super::falsy_not_zero;

    #[test]
    fn zero_is_present() {
        assert!(!falsy_not_zero(Some(0.0)));
    }

    #[test]
    fn missing_is_falsy() {
        assert!(falsy_not_zero(None));
    }

    #[test]
    fn nonzero_is_present() {
        assert!(!falsy_not_zero(Some(500.0)));
        assert!(!falsy_not_zero(Some(-5.0)));
    }
}
