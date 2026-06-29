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
fn properties_value_is_null() {
    assert_schema_default(
        json!({ "type": "object", "properties": null }),
        json!({ "$schema": DRAFT4, "type": "object" }),
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
