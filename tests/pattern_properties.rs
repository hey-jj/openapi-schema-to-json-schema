//! x-patternProperties move plus the default and custom handlers.

mod common;

use common::DRAFT4;
use openapi_schema_to_json_schema::{from_schema, Options, PatternPropertiesHandler};
use serde_json::{json, Value};
use std::sync::Arc;

fn support() -> Options {
    Options {
        support_pattern_properties: Some(true),
        ..Options::new()
    }
}

fn convert(input: Value, options: &Options) -> Value {
    from_schema(input, options).unwrap()
}

#[test]
fn same_type_string() {
    let result = convert(
        json!({
            "type": "object",
            "additionalProperties": { "type": "string" },
            "x-patternProperties": { "^[a-z]*$": { "type": "string" } }
        }),
        &support(),
    );
    assert_eq!(
        result,
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "additionalProperties": false,
            "patternProperties": { "^[a-z]*$": { "type": "string" } }
        })
    );
}

#[test]
fn same_type_number() {
    let result = convert(
        json!({
            "type": "object",
            "additionalProperties": { "type": "number" },
            "x-patternProperties": { "^[a-z]*$": { "type": "number" } }
        }),
        &support(),
    );
    assert_eq!(
        result,
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "additionalProperties": false,
            "patternProperties": { "^[a-z]*$": { "type": "number" } }
        })
    );
}

#[test]
fn one_of_pattern_property_types() {
    let result = convert(
        json!({
            "type": "object",
            "additionalProperties": { "type": "number" },
            "x-patternProperties": {
                "^[a-z]*$": { "type": "string" },
                "^[A-Z]*$": { "type": "number" }
            }
        }),
        &support(),
    );
    assert_eq!(
        result,
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "additionalProperties": false,
            "patternProperties": {
                "^[a-z]*$": { "type": "string" },
                "^[A-Z]*$": { "type": "number" }
            }
        })
    );
}

#[test]
fn matching_objects() {
    let result = convert(
        json!({
            "type": "object",
            "additionalProperties": {
                "type": "object",
                "properties": { "test": { "type": "string" } }
            },
            "x-patternProperties": {
                "^[a-z]*$": { "type": "string" },
                "^[A-Z]*$": {
                    "type": "object",
                    "properties": { "test": { "type": "string" } }
                }
            }
        }),
        &support(),
    );
    assert_eq!(
        result,
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "additionalProperties": false,
            "patternProperties": {
                "^[a-z]*$": { "type": "string" },
                "^[A-Z]*$": {
                    "type": "object",
                    "properties": { "test": { "type": "string" } }
                }
            }
        })
    );
}

#[test]
fn null_x_pattern_properties() {
    let result = convert(
        json!({
            "type": "object",
            "additionalProperties": {
                "type": "object",
                "properties": { "test": { "type": "string" } }
            },
            "x-patternProperties": null
        }),
        &support(),
    );
    assert!(result.get("x-patternProperties").is_none());
    assert_eq!(
        result,
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "additionalProperties": {
                "type": "object",
                "properties": { "test": { "type": "string" } }
            }
        })
    );
}

#[test]
fn non_matching_objects() {
    let result = convert(
        json!({
            "type": "object",
            "additionalProperties": {
                "type": "object",
                "properties": { "test": { "type": "string" } }
            },
            "x-patternProperties": {
                "^[a-z]*$": { "type": "string" },
                "^[A-Z]*$": {
                    "type": "object",
                    "properties": { "test": { "type": "integer" } }
                }
            }
        }),
        &support(),
    );
    assert_eq!(
        result,
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "additionalProperties": {
                "type": "object",
                "properties": { "test": { "type": "string" } }
            },
            "patternProperties": {
                "^[a-z]*$": { "type": "string" },
                "^[A-Z]*$": {
                    "type": "object",
                    "properties": { "test": { "type": "integer" } }
                }
            }
        })
    );
}

#[test]
fn matching_array() {
    let result = convert(
        json!({
            "type": "object",
            "additionalProperties": { "type": "array", "items": { "type": "string" } },
            "x-patternProperties": {
                "^[a-z]*$": { "type": "string" },
                "^[A-Z]*$": { "type": "array", "items": { "type": "string" } }
            }
        }),
        &support(),
    );
    assert_eq!(
        result,
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "additionalProperties": false,
            "patternProperties": {
                "^[a-z]*$": { "type": "string" },
                "^[A-Z]*$": { "type": "array", "items": { "type": "string" } }
            }
        })
    );
}

#[test]
fn composition_types() {
    let result = convert(
        json!({
            "type": "object",
            "additionalProperties": {
                "oneOf": [{ "type": "string" }, { "type": "integer" }]
            },
            "x-patternProperties": {
                "^[a-z]*$": { "oneOf": [{ "type": "string" }, { "type": "integer" }] }
            }
        }),
        &support(),
    );
    assert_eq!(
        result,
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "additionalProperties": false,
            "patternProperties": {
                "^[a-z]*$": { "oneOf": [{ "type": "string" }, { "type": "integer" }] }
            }
        })
    );
}

#[test]
fn not_supporting_pattern_properties() {
    let options = Options {
        support_pattern_properties: Some(false),
        ..Options::new()
    };
    let input = json!({
        "type": "object",
        "additionalProperties": { "type": "string" },
        "x-patternProperties": { "^[a-z]*$": { "type": "string" } }
    });
    let result = convert(input.clone(), &options);
    let mut expected = input;
    expected["$schema"] = json!(DRAFT4);
    assert_eq!(result, expected);
}

#[test]
fn not_supporting_pattern_properties_by_default() {
    let input = json!({
        "type": "object",
        "additionalProperties": { "type": "string" },
        "x-patternProperties": { "^[a-z]*$": { "type": "string" } }
    });
    let result = convert(input.clone(), &Options::new());
    let mut expected = input;
    expected["$schema"] = json!(DRAFT4);
    assert_eq!(result, expected);
}

#[test]
fn custom_pattern_properties_handler() {
    let handler: PatternPropertiesHandler = Arc::new(|schema: Value| {
        let mut schema = schema;
        if let Value::Object(map) = &mut schema {
            map.insert("patternProperties".to_string(), Value::Bool(false));
        }
        schema
    });
    let options = Options {
        support_pattern_properties: Some(true),
        pattern_properties_handler: Some(handler),
        ..Options::new()
    };
    let result = convert(
        json!({
            "type": "object",
            "additionalProperties": { "type": "string" },
            "x-patternProperties": { "^[a-z]*$": { "type": "string" } }
        }),
        &options,
    );
    assert_eq!(
        result,
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "additionalProperties": { "type": "string" },
            "patternProperties": false
        })
    );
}

#[test]
fn additional_properties_not_modified_if_true() {
    let result = convert(
        json!({
            "type": "object",
            "additionalProperties": true,
            "x-patternProperties": { "^[a-z]*$": { "type": "string" } }
        }),
        &support(),
    );
    assert_eq!(
        result,
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "additionalProperties": true,
            "patternProperties": { "^[a-z]*$": { "type": "string" } }
        })
    );
}
