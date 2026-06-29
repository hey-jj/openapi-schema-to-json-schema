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
