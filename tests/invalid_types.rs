//! Strict type validation.
//!
//! Covers both modes. With `strict_mode` on, an unknown `type` is an error.
//! With `strict_mode` off, the same `type` passes through unchanged.

mod common;

use common::{assert_schema, assert_schema_err, load_fixture, DRAFT4};
use openapi_schema_to_json_schema::{from_schema, Options};
use serde_json::json;

#[test]
fn invalid_types() {
    let opts = Options::new();
    assert_schema_err(json!({ "type": "dateTime" }), &opts, "is not a valid type");
    assert_schema_err(json!({ "type": "foo" }), &opts, "is not a valid type");
    assert_schema_err(
        json!({ "type": ["string", null] }),
        &opts,
        "is not a valid type",
    );

    let nested = load_fixture("schema-2-invalid-type.json");
    let err = from_schema(nested, &opts).unwrap_err();
    assert!(err.to_string().contains("is not a valid type"));
}

#[test]
fn valid_types() {
    let types = [
        "integer", "number", "string", "boolean", "object", "array", "null",
    ];
    for ty in types {
        assert_schema(
            json!({ "type": ty }),
            &Options::new(),
            json!({ "$schema": DRAFT4, "type": ty }),
        );
    }
}

#[test]
fn falsy_type_passes_strict_validation() {
    // validateType guards on a truthy type, so an empty string is accepted and
    // left unchanged.
    assert_schema(
        json!({ "type": "" }),
        &Options::new(),
        json!({ "$schema": DRAFT4, "type": "" }),
    );
}

#[test]
fn invalid_type_allowed_when_strict_mode_false() {
    let options = Options {
        strict_mode: Some(false),
        ..Options::new()
    };
    assert_schema(
        json!({ "type": "nonsense" }),
        &options,
        json!({ "$schema": DRAFT4, "type": "nonsense" }),
    );
}
