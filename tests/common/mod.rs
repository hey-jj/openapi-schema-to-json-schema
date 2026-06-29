//! Shared helpers for the conversion test suite.

#![allow(dead_code)]

use openapi_to_json_schema::{from_schema, Error, Options};
use serde_json::Value;

/// The draft-04 `$schema` value every converted root carries.
pub const DRAFT4: &str = "http://json-schema.org/draft-04/schema#";

/// Load a fixture from `tests/fixtures` and parse it.
pub fn load_fixture(name: &str) -> Value {
    let path = format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {path}: {e}"))
}

/// Assert that converting `input` with `options` deep-equals `expected`.
///
/// This mirrors the source `expect(convert(input, opts)).toEqual(expected)`.
/// Comparison is on `Value`, which is structural and key-order insensitive for
/// objects.
pub fn assert_schema(input: Value, options: &Options, expected: Value) {
    let got = from_schema(input, options).expect("conversion should succeed");
    assert_eq!(got, expected);
}

/// Convert with default options and assert deep equality.
pub fn assert_schema_default(input: Value, expected: Value) {
    assert_schema(input, &Options::new(), expected);
}

/// Assert that converting `input` fails with a message containing `substr`.
pub fn assert_schema_err(input: Value, options: &Options, substr: &str) {
    let err = from_schema(input, options).expect_err("conversion should fail");
    assert!(
        err.to_string().contains(substr),
        "error {err:?} should contain {substr:?}"
    );
}

/// Helper to check an error is the invalid-type variant.
pub fn is_invalid_type(err: &Error) -> bool {
    matches!(err, Error::InvalidType(_))
}
