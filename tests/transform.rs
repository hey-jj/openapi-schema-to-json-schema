//! before_transform and after_transform hooks.

mod common;

use common::DRAFT4;
use openapi_to_json_schema::{from_schema, Options};
use serde_json::{json, Value};
use std::sync::Arc;

#[test]
fn handles_conversion_in_transform_hooks() {
    let before = Arc::new(|schema: Value, _: &_| {
        let mut schema = schema;
        if let Value::Object(map) = &mut schema {
            map.insert("type".to_string(), Value::String("string".to_string()));
        }
        schema
    });
    let after = Arc::new(|schema: Value, _: &_| {
        let mut schema = schema;
        if let Value::Object(map) = &mut schema {
            map.insert("examples".to_string(), json!(["foo", "bar"]));
        }
        schema
    });

    let options = Options {
        before_transform: Some(before),
        after_transform: Some(after),
        ..Options::new()
    };

    let result = from_schema(json!({ "type": "boolean" }), &options).unwrap();
    assert_eq!(
        result,
        json!({ "$schema": DRAFT4, "type": "string", "examples": ["foo", "bar"] })
    );
}
