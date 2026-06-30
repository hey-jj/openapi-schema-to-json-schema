//! Object properties and additionalProperties.

mod common;

use common::{assert_schema_default, DRAFT4};
use serde_json::json;

#[test]
fn properties() {
    assert_schema_default(
        json!({
            "type": "object",
            "required": ["bar"],
            "properties": {
                "foo": { "type": "string", "example": "2017-01-01T12:34:56Z" },
                "bar": { "type": "string", "nullable": true }
            }
        }),
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "required": ["bar"],
            "properties": {
                "foo": { "type": "string" },
                "bar": { "type": ["string", "null"] }
            }
        }),
    );
}

#[test]
fn required_name_without_property_is_kept() {
    // A required name with no `properties` entry is kept. It was never a
    // property, so conversion did not remove it.
    assert_schema_default(
        json!({
            "type": "object",
            "required": ["a", "b"],
            "properties": { "a": { "type": "string" } }
        }),
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "required": ["a", "b"],
            "properties": { "a": { "type": "string" } }
        }),
    );
}

#[test]
fn properties_value_is_null() {
    assert_schema_default(
        json!({ "type": "object", "properties": null }),
        json!({ "$schema": DRAFT4, "type": "object" }),
    );
}

#[test]
fn properties_as_array_walks_index_keys() {
    // An array `properties` is walked by index. Object elements land under
    // their numeric string keys. Non-object elements are dropped.
    assert_schema_default(
        json!({
            "type": "object",
            "properties": [{ "type": "string", "example": "x" }, 2, { "type": "integer" }]
        }),
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "properties": { "0": { "type": "string" }, "2": { "type": "integer" } }
        }),
    );
}

#[test]
fn strips_malformed_properties_children() {
    assert_schema_default(
        json!({
            "type": "object",
            "required": ["bar"],
            "properties": {
                "foo": { "type": "string", "example": "2017-01-01T12:34:56Z" },
                "foobar": 2,
                "bar": { "type": "string", "nullable": true },
                "baz": null
            }
        }),
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "required": ["bar"],
            "properties": {
                "foo": { "type": "string" },
                "bar": { "type": ["string", "null"] }
            }
        }),
    );
}

#[test]
fn additional_properties_is_false() {
    assert_schema_default(
        json!({
            "type": "object",
            "properties": { "foo": { "type": "string", "example": "x" } },
            "additionalProperties": false
        }),
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "properties": { "foo": { "type": "string" } },
            "additionalProperties": false
        }),
    );
}

#[test]
fn additional_properties_is_true() {
    assert_schema_default(
        json!({
            "type": "object",
            "properties": { "foo": { "type": "string", "example": "x" } },
            "additionalProperties": true
        }),
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "properties": { "foo": { "type": "string" } },
            "additionalProperties": true
        }),
    );
}

#[test]
fn additional_properties_is_an_object() {
    assert_schema_default(
        json!({
            "type": "object",
            "properties": { "foo": { "type": "string", "example": "x" } },
            "additionalProperties": {
                "type": "object",
                "properties": { "foo": { "type": "string" } }
            }
        }),
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "properties": { "foo": { "type": "string" } },
            "additionalProperties": {
                "type": "object",
                "properties": { "foo": { "type": "string" } }
            }
        }),
    );
}
