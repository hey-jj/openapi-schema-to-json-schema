//! The recursive schema converter.

use serde_json::{Map, Number, Value};

use crate::consts::{BYTE_PATTERN, DRAFT_04, VALID_OPENAPI_FORMATS, VALID_TYPES};
use crate::error::Error;
use crate::options::ResolvedOptions;
use crate::pattern::default_pattern_properties_handler;
use crate::value::{is_falsy, is_object};

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
/// Step order: before-transform hook, struct recursion, definition keywords,
/// properties and required pruning, type validation, nullable handling, format
/// handling, pattern properties, keyword stripping, after-transform hook.
fn convert_schema(schema: Value, options: &ResolvedOptions) -> Result<Value, Error> {
    let mut schema = schema;

    if let Some(hook) = &options.before_transform {
        schema = hook(schema, options);
    }

    // Keyword processing reads object members. An array carries no keywords, so
    // it passes through. A scalar or null is not a schema and stops conversion.
    match &schema {
        Value::Object(_) => {}
        Value::Array(_) => return Ok(maybe_after_transform(schema, options)),
        other => {
            let rendered = serde_json::to_string(other).unwrap_or_default();
            return Err(Error::InvalidInput(format!(
                "schema must be an object, got {rendered}"
            )));
        }
    }

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
    for keyword in ResolvedOptions::STRUCTS {
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
/// Each keyword is a dotted path. The value at the path, when an object or
/// array, is run through [`convert_properties`] and written back.
fn convert_definition_keywords(schema: &mut Value, options: &ResolvedOptions) -> Result<(), Error> {
    for path in &options.definition_keywords {
        let inner = path_get(schema, path);
        // Objects, arrays, and null all enter conversion. A null carries no
        // sub-schemas, so it converts to an empty object. A missing path is
        // skipped.
        match inner {
            Some(value) if value.is_object() || value.is_array() || value.is_null() => {
                let converted = convert_properties(value, options)?;
                path_set(schema, path, converted);
            }
            _ => {}
        }
    }
    Ok(())
}

/// Convert `properties` and prune `required`.
///
/// `properties` is replaced by the converted map. A non-object or null value
/// converts to an empty map, which is then deleted. `required` drops only names
/// whose property was present in the input but removed during conversion. A name
/// with no property entry is kept. An empty `required` or empty `properties` is
/// deleted.
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
    let input_names: Vec<String> = match &props_value {
        Value::Object(p) => p.keys().cloned().collect(),
        _ => Vec::new(),
    };
    let converted = convert_properties(props_value, options)?;
    map.insert("properties".to_string(), converted);

    // Only an array required is touched. Drop a name only when it was a declared
    // property that did not survive conversion.
    if let Some(Value::Array(required)) = map.get("required").cloned() {
        let kept_names: Vec<String> = match map.get("properties") {
            Some(Value::Object(p)) => p.keys().cloned().collect(),
            _ => Vec::new(),
        };
        let filtered: Vec<Value> = required
            .into_iter()
            .filter(|key| match key {
                Value::String(name) => kept_names.contains(name) || !input_names.contains(name),
                _ => false,
            })
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
/// by index, so object elements land under their numeric string keys.
fn convert_properties(value: Value, options: &ResolvedOptions) -> Result<Value, Error> {
    let mut out = Map::new();

    // Build the (key, value) pairs to walk. Scalars and null return early.
    // Objects walk their keys. Arrays walk index keys.
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

/// Apply `nullable` handling. When `nullable` is exactly true and a `type` is
/// present, replace the scalar type with `[type, "null"]` and append `null` to
/// an array `enum` that does not already contain it.
fn convert_types(schema: &mut Value) {
    let Value::Object(map) = schema else {
        return;
    };
    let nullable = map.get("nullable") == Some(&Value::Bool(true));
    if !nullable {
        return;
    }
    // A missing `type` ends the function. A present `type: null` proceeds.
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

    // Bounds computed as f64, matching the `2 ** n` arithmetic. Integral bounds
    // within i64 range serialize as integers, the rest as floats.
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
/// Write the bound when the current value is falsy and not the number 0, or
/// when it is a number out of range. A present 0, a present in-range number, or
/// any present non-number value is kept. Falsy here follows the truthiness
/// rules in [`crate::value`]: null, false, 0, and the empty string.
fn clamp_bounds(map: &mut Map<String, Value>, min: f64, max: f64) {
    if take_bound(map.get("minimum"), |v| v < min) {
        map.insert("minimum".to_string(), bound_value(min));
    }
    if take_bound(map.get("maximum"), |v| v > max) {
        map.insert("maximum".to_string(), bound_value(max));
    }
}

/// Decide whether to overwrite a bound with the format default.
///
/// `out_of_range` tests a present number against the format limit. A missing
/// value, or a falsy value other than the number 0, takes the default. A
/// present non-number that is not falsy is kept.
fn take_bound(current: Option<&Value>, out_of_range: impl Fn(f64) -> bool) -> bool {
    match current {
        None => true,
        Some(Value::Number(n)) => match n.as_f64() {
            Some(0.0) => false,
            Some(v) => out_of_range(v),
            None => false,
        },
        Some(other) => is_falsy(other),
    }
}

/// Build a JSON number for a bound. An integral bound inside i64 range becomes
/// an integer Number, so it serializes without a fractional part. Bounds beyond
/// i64 range, such as the int64 maximum and the float and double limits, stay
/// floats.
fn bound_value(v: f64) -> Value {
    if v.fract() == 0.0 && v >= -(2f64.powi(63)) && v < 2f64.powi(63) {
        return Value::Number(Number::from(v as i64));
    }
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

// ---- path get/set over object and array paths -----------------------------

/// Split a path into segments.
///
/// Handles dotted keys and bracket notation: `a.b`, `a[0]`, `a["x"]`, and
/// `a['x']` resolve to the same segments. A bracketed quoted key keeps its
/// literal text. A bracketed bare token (an array index) becomes its digits.
/// Each segment is one object key or one array index.
fn parse_path(path: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut chars = path.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '.' => {
                segments.push(std::mem::take(&mut current));
            }
            '[' => {
                if !current.is_empty() {
                    segments.push(std::mem::take(&mut current));
                }
                let quote = matches!(chars.peek(), Some('"') | Some('\'')).then(|| chars.next());
                let mut inner = String::new();
                for ic in chars.by_ref() {
                    match quote {
                        Some(Some(q)) if ic == q => break,
                        None if ic == ']' => break,
                        _ => inner.push(ic),
                    }
                }
                // Drop the closing bracket after a quoted key.
                if quote.is_some() {
                    for ic in chars.by_ref() {
                        if ic == ']' {
                            break;
                        }
                    }
                }
                segments.push(inner);
            }
            _ => current.push(c),
        }
    }
    if !current.is_empty() || segments.is_empty() {
        segments.push(current);
    }
    segments
}

/// Resolve a path against a value. Returns a clone of the value at the path, or
/// None when any segment is missing. Objects index by key, arrays index by a
/// numeric segment.
fn path_get(root: &Value, path: &str) -> Option<Value> {
    let mut current = root;
    for segment in parse_path(path) {
        current = match current {
            Value::Object(map) => map.get(&segment)?,
            Value::Array(items) => items.get(segment.parse::<usize>().ok()?)?,
            _ => return None,
        };
    }
    Some(current.clone())
}

/// Write a value at a path, creating intermediate objects as needed. An
/// existing array along the path is indexed by a numeric segment.
fn path_set(root: &mut Value, path: &str, value: Value) {
    let segments = parse_path(path);
    set_recursive(root, &segments, value);
}

fn set_recursive(node: &mut Value, segments: &[String], value: Value) {
    let (head, rest) = match segments.split_first() {
        Some(parts) => parts,
        None => return,
    };

    // Index into an existing array when the segment is numeric.
    if let Value::Array(items) = node {
        if let Ok(index) = head.parse::<usize>() {
            if let Some(slot) = items.get_mut(index) {
                if rest.is_empty() {
                    *slot = value;
                } else {
                    set_recursive(slot, rest, value);
                }
                return;
            }
        }
    }

    if !node.is_object() {
        *node = Value::Object(Map::new());
    }
    let Value::Object(map) = node else {
        return;
    };
    if rest.is_empty() {
        map.insert(head.clone(), value);
        return;
    }
    let child = map
        .entry(head.clone())
        .or_insert_with(|| Value::Object(Map::new()));
    set_recursive(child, rest, value);
}

#[cfg(test)]
mod guard_tests {
    use super::{bound_value, take_bound};
    use serde_json::json;

    // out_of_range for a minimum bound of -100.
    fn below_min(v: f64) -> bool {
        v < -100.0
    }

    #[test]
    fn missing_takes_the_bound() {
        assert!(take_bound(None, below_min));
    }

    #[test]
    fn present_zero_is_kept() {
        assert!(!take_bound(Some(&json!(0)), below_min));
    }

    #[test]
    fn in_range_number_is_kept() {
        assert!(!take_bound(Some(&json!(-50)), below_min));
    }

    #[test]
    fn out_of_range_number_takes_the_bound() {
        assert!(take_bound(Some(&json!(-200)), below_min));
    }

    #[test]
    fn falsy_non_number_takes_the_bound() {
        assert!(take_bound(Some(&json!(null)), below_min));
        assert!(take_bound(Some(&json!(false)), below_min));
        assert!(take_bound(Some(&json!("")), below_min));
    }

    #[test]
    fn truthy_non_number_is_kept() {
        assert!(!take_bound(Some(&json!("abc")), below_min));
        assert!(!take_bound(Some(&json!([])), below_min));
        assert!(!take_bound(Some(&json!(true)), below_min));
    }

    #[test]
    fn small_integral_bound_is_integer() {
        assert_eq!(bound_value(-2147483648.0), json!(-2147483648_i64));
    }

    #[test]
    fn large_bound_stays_float() {
        // 2**63 exceeds i64 range, so it stays a float Number.
        assert_eq!(bound_value(2f64.powi(63)), json!(2f64.powi(63)));
    }
}
