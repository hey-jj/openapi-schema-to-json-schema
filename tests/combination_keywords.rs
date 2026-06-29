//! allOf, anyOf, oneOf, not recursion and the `not` property vs struct case.

mod common;

use common::{assert_schema_default, DRAFT4};
use serde_json::json;

#[test]
fn iterates_all_ofs() {
    let input = json!({
        "allOf": [
            {
                "type": "object",
                "required": ["foo"],
                "properties": { "foo": { "type": "integer" } }
            },
            { "allOf": [{ "type": "number" }] }
        ]
    });
    let mut expected = input.clone();
    expected["$schema"] = json!(DRAFT4);
    assert_schema_default(input, expected);
}

#[test]
fn iterates_any_ofs() {
    let input = json!({
        "anyOf": [
            {
                "type": "object",
                "required": ["foo"],
                "properties": { "foo": { "type": "integer" } }
            },
            {
                "anyOf": [
                    { "type": "object", "properties": { "bar": { "type": "number" } } }
                ]
            }
        ]
    });
    let mut expected = input.clone();
    expected["$schema"] = json!(DRAFT4);
    assert_schema_default(input, expected);
}

#[test]
fn iterates_one_ofs() {
    let input = json!({
        "oneOf": [
            {
                "type": "object",
                "required": ["foo"],
                "properties": { "foo": { "type": "integer" } }
            },
            {
                "oneOf": [
                    { "type": "object", "properties": { "bar": { "type": "number" } } }
                ]
            }
        ]
    });
    let mut expected = input.clone();
    expected["$schema"] = json!(DRAFT4);
    assert_schema_default(input, expected);
}

#[test]
fn not_as_property_name() {
    // `not` under properties is an ordinary property, not the struct keyword.
    assert_schema_default(
        json!({
            "type": "object",
            "properties": { "not": { "type": "string", "minLength": 8 } }
        }),
        json!({
            "$schema": DRAFT4,
            "type": "object",
            "properties": { "not": { "type": "string", "minLength": 8 } }
        }),
    );
}

#[test]
fn not_as_struct_keyword() {
    assert_schema_default(
        json!({ "not": { "type": "string", "minLength": 8 } }),
        json!({ "$schema": DRAFT4, "not": { "type": "string", "minLength": 8 } }),
    );
}

#[test]
fn nested_combination_keywords() {
    assert_schema_default(
        json!({
            "anyOf": [
                {
                    "allOf": [
                        {
                            "type": "object",
                            "properties": { "foo": { "type": "string", "nullable": true } }
                        },
                        {
                            "type": "object",
                            "properties": { "bar": { "type": "integer", "nullable": true } }
                        }
                    ]
                },
                { "type": "object", "properties": { "foo": { "type": "string" } } },
                { "not": { "type": "string", "example": "foobar" } }
            ]
        }),
        json!({
            "$schema": DRAFT4,
            "anyOf": [
                {
                    "allOf": [
                        {
                            "type": "object",
                            "properties": { "foo": { "type": ["string", "null"] } }
                        },
                        {
                            "type": "object",
                            "properties": { "bar": { "type": ["integer", "null"] } }
                        }
                    ]
                },
                { "type": "object", "properties": { "foo": { "type": "string" } } },
                { "not": { "type": "string" } }
            ]
        }),
    );
}
