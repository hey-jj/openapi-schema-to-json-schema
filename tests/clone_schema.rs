//! Clone semantics.
//!
//! There is no circular-reference case. An owned `serde_json::Value` cannot
//! hold a cycle, so JSON input never cycles.

mod common;

use common::DRAFT4;
use openapi_to_json_schema::{from_schema, Options};
use serde_json::json;

#[test]
fn cloning_schema_by_default() {
    let input = json!({
        "type": "string",
        "nullable": true,
        "properties": {
            "foo": true,
            "bar": { "allOf": [null, { "type": "string" }, null] }
        }
    });
    let original = input.clone();
    let result = from_schema(input.clone(), &Options::new()).unwrap();

    assert_eq!(
        result,
        json!({
            "$schema": DRAFT4,
            "type": ["string", "null"],
            "properties": {
                "bar": { "allOf": [{ "type": "string" }] }
            }
        })
    );
    // The input value is unchanged after conversion.
    assert_eq!(input, original);
}

#[test]
fn cloning_schema_with_clone_schema_option() {
    let input = json!({ "type": "string", "nullable": true });
    let original = input.clone();
    let options = Options {
        clone_schema: Some(true),
        ..Options::new()
    };
    let result = from_schema(input.clone(), &options).unwrap();

    assert_eq!(
        result,
        json!({ "$schema": DRAFT4, "type": ["string", "null"] })
    );
    assert_eq!(input, original);
}

#[test]
fn direct_schema_modification() {
    let options = Options {
        clone_schema: Some(false),
        ..Options::new()
    };
    let result = from_schema(json!({ "type": "string", "nullable": true }), &options).unwrap();
    assert_eq!(
        result,
        json!({ "$schema": DRAFT4, "type": ["string", "null"] })
    );
}
