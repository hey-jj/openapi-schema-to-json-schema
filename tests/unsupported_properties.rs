//! NOT_SUPPORTED keyword stripping and keepNotSupported.

mod common;

use common::{assert_schema, assert_schema_default, DRAFT4};
use openapi_schema_to_json_schema::Options;
use serde_json::json;

#[test]
fn remove_discriminator_by_default() {
    let branch = json!({
        "type": "object",
        "required": ["foo"],
        "properties": { "foo": { "type": "string" } }
    });
    assert_schema_default(
        json!({
            "oneOf": [branch, branch],
            "discriminator": { "propertyName": "foo" }
        }),
        json!({ "$schema": DRAFT4, "oneOf": [branch, branch] }),
    );
}

#[test]
fn remove_read_only_by_default() {
    assert_schema_default(
        json!({
            "type": "object",
            "properties": { "readOnly": { "type": "string", "readOnly": true } }
        }),
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "properties": { "readOnly": { "type": "string" } }
        }),
    );
}

#[test]
fn remove_write_only_by_default() {
    assert_schema_default(
        json!({
            "type": "object",
            "properties": { "test": { "type": "string", "writeOnly": true } }
        }),
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "properties": { "test": { "type": "string" } }
        }),
    );
}

#[test]
fn remove_xml_by_default() {
    assert_schema_default(
        json!({
            "type": "object",
            "properties": { "foo": { "type": "string", "xml": { "attribute": true } } }
        }),
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "properties": { "foo": { "type": "string" } }
        }),
    );
}

#[test]
fn remove_external_docs_by_default() {
    assert_schema_default(
        json!({
            "type": "object",
            "properties": { "foo": { "type": "string" } },
            "externalDocs": { "url": "http://foo.bar" }
        }),
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "properties": { "foo": { "type": "string" } }
        }),
    );
}

#[test]
fn remove_example_by_default() {
    assert_schema_default(
        json!({ "type": "string", "example": "foo" }),
        json!({ "$schema": DRAFT4, "type": "string" }),
    );
}

#[test]
fn remove_deprecated_by_default() {
    assert_schema_default(
        json!({ "type": "string", "deprecated": true }),
        json!({ "$schema": DRAFT4, "type": "string" }),
    );
}

#[test]
fn retaining_fields() {
    let options = Options {
        keep_not_supported: Some(vec!["readOnly".to_string(), "discriminator".to_string()]),
        ..Options::new()
    };
    assert_schema(
        json!({
            "type": "object",
            "properties": {
                "readOnly": { "type": "string", "readOnly": true, "example": "foo" },
                "anotherProp": {
                    "type": "object",
                    "properties": { "writeOnly": { "type": "string", "writeOnly": true } }
                }
            },
            "discriminator": "bar"
        }),
        &options,
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "properties": {
                "readOnly": { "type": "string", "readOnly": true },
                "anotherProp": {
                    "type": "object",
                    "properties": { "writeOnly": { "type": "string" } }
                }
            },
            "discriminator": "bar"
        }),
    );
}
