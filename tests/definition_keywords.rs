//! definitionKeywords convert named sub-schemas via dotted paths.

mod common;

use common::{assert_schema, assert_schema_default, DRAFT4};
use openapi_to_json_schema::Options;
use serde_json::json;

fn shared_definitions() -> serde_json::Value {
    json!({
        "definitions": {
            "sharedDefinition": {
                "type": "object",
                "properties": {
                    "foo": { "type": "string", "nullable": true }
                }
            }
        }
    })
}

#[test]
fn handles_conversion_in_definition_keywords() {
    let options = Options {
        definition_keywords: Some(vec!["definitions".to_string()]),
        ..Options::new()
    };
    assert_schema(
        shared_definitions(),
        &options,
        json!({
            "$schema": DRAFT4,
            "definitions": {
                "sharedDefinition": {
                    "type": "object",
                    "properties": {
                        "foo": { "type": ["string", "null"] }
                    }
                }
            }
        }),
    );
}

#[test]
fn does_not_convert_without_definition_keywords() {
    assert_schema_default(
        shared_definitions(),
        json!({
            "$schema": DRAFT4,
            "definitions": {
                "sharedDefinition": {
                    "type": "object",
                    "properties": {
                        "foo": { "type": "string", "nullable": true }
                    }
                }
            }
        }),
    );
}

#[test]
fn handles_nested_definition_keywords() {
    let input = json!({
        "schema": {
            "definitions": {
                "sharedDefinition": {
                    "type": "object",
                    "properties": {
                        "foo": { "type": "string", "nullable": true }
                    }
                }
            }
        }
    });
    let options = Options {
        definition_keywords: Some(vec!["schema.definitions".to_string()]),
        ..Options::new()
    };
    assert_schema(
        input,
        &options,
        json!({
            "$schema": DRAFT4,
            "schema": {
                "definitions": {
                    "sharedDefinition": {
                        "type": "object",
                        "properties": {
                            "foo": { "type": ["string", "null"] }
                        }
                    }
                }
            }
        }),
    );
}

fn definitions_option() -> Options {
    Options {
        definition_keywords: Some(vec!["definitions".to_string()]),
        ..Options::new()
    }
}

#[test]
fn missing_path_is_left_alone() {
    // The path resolves to nothing, so the node is unchanged.
    assert_schema(
        json!({ "type": "object" }),
        &definitions_option(),
        json!({ "type": "object", "$schema": DRAFT4 }),
    );
}

#[test]
fn null_path_becomes_empty_object() {
    // A null value at the path enters conversion. Converting null yields an
    // empty object, which is written back.
    assert_schema(
        json!({ "definitions": null }),
        &definitions_option(),
        json!({ "definitions": {}, "$schema": DRAFT4 }),
    );
}

#[test]
fn scalar_path_is_left_alone() {
    // A scalar at the path is not an object, so it stays as is.
    assert_schema(
        json!({ "definitions": 5 }),
        &definitions_option(),
        json!({ "definitions": 5, "$schema": DRAFT4 }),
    );
}

#[test]
fn handles_bracket_notation_in_path() {
    // A bracketed segment indexes an array. `defs[0]` reaches the first element
    // and its definitions are converted in place.
    let input = json!({
        "defs": [
            {
                "sharedDefinition": {
                    "type": "object",
                    "properties": {
                        "foo": { "type": "string", "nullable": true }
                    }
                }
            }
        ]
    });
    let options = Options {
        definition_keywords: Some(vec!["defs[0]".to_string()]),
        ..Options::new()
    };
    assert_schema(
        input,
        &options,
        json!({
            "$schema": DRAFT4,
            "defs": [
                {
                    "sharedDefinition": {
                        "type": "object",
                        "properties": {
                            "foo": { "type": ["string", "null"] }
                        }
                    }
                }
            ]
        }),
    );
}
