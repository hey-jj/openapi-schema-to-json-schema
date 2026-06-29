//! Array `items` as a single schema and as a tuple array.

mod common;

use common::{assert_schema_default, DRAFT4};
use serde_json::json;

#[test]
fn items() {
    assert_schema_default(
        json!({
            "type": "array",
            "items": { "type": "string", "example": "2017-01-01T12:34:56Z" }
        }),
        json!({
            "$schema": DRAFT4,
            "type": "array",
            "items": { "type": "string" }
        }),
    );
}

#[test]
fn single_object_items_recurses() {
    // A single-object items branch recurses into the schema, so nullable widens
    // the type and example is stripped.
    assert_schema_default(
        json!({
            "type": "array",
            "items": { "type": "string", "nullable": true, "example": "x" }
        }),
        json!({
            "$schema": DRAFT4,
            "type": "array",
            "items": { "type": ["string", "null"] }
        }),
    );
}

#[test]
fn handles_items_with_invalid_values() {
    assert_schema_default(
        json!({
            "type": "array",
            "items": [
                { "type": "string" },
                2,
                null,
                { "type": "number" },
                "foo",
                { "type": "array" }
            ]
        }),
        json!({
            "$schema": DRAFT4,
            "type": "array",
            "items": [
                { "type": "string" },
                { "type": "number" },
                { "type": "array" }
            ]
        }),
    );
}
