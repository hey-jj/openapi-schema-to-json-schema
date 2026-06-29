//! removeReadOnly and removeWriteOnly with required pruning.

mod common;

use common::{assert_schema, assert_schema_default, DRAFT4};
use openapi_to_json_schema::Options;
use serde_json::json;

fn remove_read_only() -> Options {
    Options {
        remove_read_only: Some(true),
        ..Options::new()
    }
}

fn remove_write_only() -> Options {
    Options {
        remove_write_only: Some(true),
        ..Options::new()
    }
}

#[test]
fn removing_read_only_prop() {
    assert_schema(
        json!({
            "type": "object",
            "properties": {
                "prop1": { "type": "string", "readOnly": true },
                "prop2": { "type": "string" }
            }
        }),
        &remove_read_only(),
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "properties": { "prop2": { "type": "string" } }
        }),
    );
}

#[test]
fn removing_read_only_prop_even_if_keeping() {
    let options = Options {
        remove_read_only: Some(true),
        keep_not_supported: Some(vec!["readOnly".to_string()]),
        ..Options::new()
    };
    assert_schema(
        json!({
            "type": "object",
            "properties": {
                "prop1": { "type": "string", "readOnly": true },
                "prop2": { "type": "string" }
            }
        }),
        &options,
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "properties": { "prop2": { "type": "string" } }
        }),
    );
}

#[test]
fn removing_write_only_prop_and_required() {
    assert_schema(
        json!({
            "type": "object",
            "required": ["prop1", "prop2", "prop3", "prop4"],
            "properties": {
                "prop1": { "type": "string", "writeOnly": true },
                "prop2": { "type": "string", "writeOnly": true },
                "prop3": { "type": "string", "writeOnly": true },
                "prop4": { "type": "string" }
            }
        }),
        &remove_write_only(),
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "required": ["prop4"],
            "properties": { "prop4": { "type": "string" } }
        }),
    );
}

#[test]
fn removing_read_only_from_required() {
    assert_schema(
        json!({
            "type": "object",
            "required": ["prop1", "prop2", "prop3", "prop4"],
            "properties": {
                "prop1": { "type": "string" },
                "prop2": { "type": "string", "readOnly": true },
                "prop3": { "type": "string", "readOnly": true },
                "prop4": { "type": "string" }
            }
        }),
        &remove_read_only(),
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "required": ["prop1", "prop4"],
            "properties": {
                "prop1": { "type": "string" },
                "prop4": { "type": "string" }
            }
        }),
    );
}

#[test]
fn deleting_required_if_empty() {
    assert_schema(
        json!({
            "type": "object",
            "required": ["prop1"],
            "properties": {
                "prop1": { "type": "string", "readOnly": true },
                "prop2": { "type": "string" }
            }
        }),
        &remove_read_only(),
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "properties": { "prop2": { "type": "string" } }
        }),
    );
}

#[test]
fn deleting_properties_if_empty() {
    assert_schema(
        json!({
            "type": "object",
            "required": ["prop1"],
            "properties": { "prop1": { "type": "string", "readOnly": true } }
        }),
        &remove_read_only(),
        json!({ "$schema": DRAFT4, "type": "object" }),
    );
}

#[test]
fn not_removing_read_only_props_by_default() {
    assert_schema_default(
        json!({
            "type": "object",
            "required": ["prop1", "prop2"],
            "properties": {
                "prop1": { "type": "string", "readOnly": true },
                "prop2": { "type": "string" }
            }
        }),
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "required": ["prop1", "prop2"],
            "properties": {
                "prop1": { "type": "string" },
                "prop2": { "type": "string" }
            }
        }),
    );
}

#[test]
fn not_removing_write_only_props_by_default() {
    assert_schema_default(
        json!({
            "type": "object",
            "required": ["prop1", "prop2"],
            "properties": {
                "prop1": { "type": "string", "writeOnly": true },
                "prop2": { "type": "string" }
            }
        }),
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "required": ["prop1", "prop2"],
            "properties": {
                "prop1": { "type": "string" },
                "prop2": { "type": "string" }
            }
        }),
    );
}

#[test]
fn non_true_read_only_value_keeps_the_property() {
    // The removal check compares against boolean true. A string readOnly does
    // not remove the property, and the readOnly key is then stripped.
    assert_schema(
        json!({
            "type": "object",
            "properties": { "a": { "type": "string", "readOnly": "yes" } }
        }),
        &remove_read_only(),
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "properties": { "a": { "type": "string" } }
        }),
    );
}

#[test]
fn remove_read_only_and_write_only_together() {
    let options = Options {
        remove_read_only: Some(true),
        remove_write_only: Some(true),
        ..Options::new()
    };
    assert_schema(
        json!({
            "type": "object",
            "properties": {
                "a": { "type": "string", "readOnly": true },
                "b": { "type": "string", "writeOnly": true },
                "c": { "type": "string" }
            }
        }),
        &options,
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "properties": { "c": { "type": "string" } }
        }),
    );
}

#[test]
fn deep_schema() {
    assert_schema(
        json!({
            "type": "object",
            "required": ["prop1", "prop2"],
            "properties": {
                "prop1": { "type": "string", "readOnly": true },
                "prop2": {
                    "allOf": [
                        {
                            "type": "object",
                            "required": ["prop3"],
                            "properties": { "prop3": { "type": "object", "readOnly": true } }
                        },
                        {
                            "type": "object",
                            "properties": { "prop4": { "type": "object", "readOnly": true } }
                        }
                    ]
                }
            }
        }),
        &remove_read_only(),
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "required": ["prop2"],
            "properties": {
                "prop2": {
                    "allOf": [
                        { "type": "object" },
                        { "type": "object" }
                    ]
                }
            }
        }),
    );
}
