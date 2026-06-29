//! Parameter and response conversion.

use serde_json::{Map, Value};

use crate::error::Error;
use crate::options::ResolvedOptions;
use crate::schema::convert_from_schema;

/// Convert a parameter or response object.
///
/// Precedence, first match wins:
///
/// 1. A truthy `schema` member converts that schema.
/// 2. A truthy `content` member converts each MIME schema into a map keyed by
///    MIME type. The map itself has no `$schema`.
/// 3. Strict mode raises [`Error::InvalidInput`].
/// 4. Otherwise an empty schema is converted.
///
/// A truthy `description` on the parameter is copied onto each result,
/// overwriting any description the schema produced.
pub(crate) fn convert_from_parameter(
    parameter: Value,
    options: &ResolvedOptions,
) -> Result<Value, Error> {
    let object = match &parameter {
        Value::Object(map) => map,
        _ => &Map::new(),
    };

    let description = object.get("description").cloned();

    // 1. schema present and truthy.
    if let Some(schema) = object.get("schema") {
        if is_truthy(schema) {
            return convert_parameter_schema(schema.clone(), description, options);
        }
    }

    // 2. content present and truthy.
    if let Some(content) = object.get("content") {
        if is_truthy(content) {
            return convert_from_contents(content, description, options);
        }
    }

    // 3. strict mode.
    if options.strict_mode {
        return Err(Error::InvalidInput(
            "OpenAPI parameter must have either a 'schema' or a 'content' property".to_string(),
        ));
    }

    // 4. lenient fallback.
    convert_parameter_schema(Value::Object(Map::new()), description, options)
}

/// Convert one schema and copy a truthy description.
fn convert_parameter_schema(
    schema: Value,
    description: Option<Value>,
    options: &ResolvedOptions,
) -> Result<Value, Error> {
    let schema = if is_truthy(&schema) {
        schema
    } else {
        Value::Object(Map::new())
    };
    let mut json_schema = convert_from_schema(schema, options)?;
    if let Some(desc) = description {
        if is_truthy(&desc) {
            if let Value::Object(map) = &mut json_schema {
                map.insert("description".to_string(), desc);
            }
        }
    }
    Ok(json_schema)
}

/// Convert each MIME schema into a map keyed by MIME type.
fn convert_from_contents(
    content: &Value,
    description: Option<Value>,
    options: &ResolvedOptions,
) -> Result<Value, Error> {
    let mut schemas = Map::new();
    if let Value::Object(content_map) = content {
        for (mime, entry) in content_map {
            let schema = match entry {
                Value::Object(m) => m.get("schema").cloned().unwrap_or(Value::Null),
                _ => Value::Null,
            };
            let converted = convert_parameter_schema(schema, description.clone(), options)?;
            schemas.insert(mime.clone(), converted);
        }
    }
    Ok(Value::Object(schemas))
}

/// JS truthiness for a JSON value. False for null, false, 0, empty string.
fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(true),
        Value::String(s) => !s.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}
