//! Fixture golden for a deeply nested schema.
//!
//! `clone_schema: false` has no observable effect here. This crate owns its
//! input, so there is no shared input to mutate. The case is kept as an
//! output-equality assertion.

mod common;

use common::load_fixture;
use openapi_schema_to_json_schema::{from_schema, Options};

#[test]
fn complex_schema() {
    let input = load_fixture("schema-1.json");
    let expected = load_fixture("schema-1-expected.json");
    let result = from_schema(input, &Options::new()).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn converting_complex_schema_in_place() {
    let input = load_fixture("schema-1.json");
    let expected = load_fixture("schema-1-expected.json");
    let options = Options {
        clone_schema: Some(false),
        ..Options::new()
    };
    let result = from_schema(input, &options).unwrap();
    assert_eq!(result, expected);
}
