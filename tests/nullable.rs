//! nullable widens type and enum.

mod common;

use common::{assert_schema_default, DRAFT4};
use serde_json::json;

#[test]
fn handles_nullable_without_enum() {
    assert_schema_default(
        json!({ "type": "string", "nullable": true }),
        json!({ "$schema": DRAFT4, "type": ["string", "null"] }),
    );
    assert_schema_default(
        json!({ "type": "string", "nullable": false }),
        json!({ "$schema": DRAFT4, "type": "string" }),
    );
}

#[test]
fn handles_nullable_with_enum() {
    assert_schema_default(
        json!({ "type": "string", "enum": ["a", "b"], "nullable": true }),
        json!({ "$schema": DRAFT4, "type": ["string", "null"], "enum": ["a", "b", null] }),
    );
    assert_schema_default(
        json!({ "type": "string", "enum": ["a", "b", null], "nullable": true }),
        json!({ "$schema": DRAFT4, "type": ["string", "null"], "enum": ["a", "b", null] }),
    );
    assert_schema_default(
        json!({ "type": "string", "enum": ["a", "b"], "nullable": false }),
        json!({ "$schema": DRAFT4, "type": "string", "enum": ["a", "b"] }),
    );
}

#[test]
fn nullable_with_no_type() {
    // convertTypes only acts when a `type` is present. With no type, the
    // nullable keyword is stripped and no type array is created.
    assert_schema_default(json!({ "nullable": true }), json!({ "$schema": DRAFT4 }));
}

#[test]
fn nullable_widens_a_null_type() {
    // A present `type: null` is widened, and strict validation accepts the
    // falsy value.
    assert_schema_default(
        json!({ "type": null, "nullable": true }),
        json!({ "$schema": DRAFT4, "type": [null, "null"] }),
    );
}

#[test]
fn nullable_does_not_duplicate_null_type() {
    assert_schema_default(
        json!({ "type": "null", "nullable": true }),
        json!({ "$schema": DRAFT4, "type": ["null"] }),
    );
}

#[test]
fn nullable_leaves_a_non_array_enum() {
    // The enum widening only applies to an array enum. A non-array enum is left
    // untouched while the type still widens.
    assert_schema_default(
        json!({ "type": "string", "enum": { "a": 1 }, "nullable": true }),
        json!({ "$schema": DRAFT4, "type": ["string", "null"], "enum": { "a": 1 } }),
    );
}
