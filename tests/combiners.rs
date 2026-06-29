//! Null combiner keywords are deleted.

mod common;

use common::{assert_schema_default, DRAFT4};
use serde_json::json;

#[test]
fn all_of_is_null() {
    assert_schema_default(json!({ "allOf": null }), json!({ "$schema": DRAFT4 }));
}

#[test]
fn any_of_is_null() {
    assert_schema_default(json!({ "anyOf": null }), json!({ "$schema": DRAFT4 }));
}

#[test]
fn one_of_is_null() {
    assert_schema_default(json!({ "oneOf": null }), json!({ "$schema": DRAFT4 }));
}

#[test]
fn object_additional_properties_recurses() {
    // A single-object additionalProperties is recursed, so a nullable inside it
    // widens the type.
    assert_schema_default(
        json!({
            "type": "object",
            "additionalProperties": { "type": "string", "nullable": true }
        }),
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "additionalProperties": { "type": ["string", "null"] }
        }),
    );
}
