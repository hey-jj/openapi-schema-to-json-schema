//! Null combiner keywords are deleted.

mod common;

use common::{assert_schema_default, DRAFT4};
use serde_json::json;

#[test]
fn all_of_is_null() {
    assert_schema_default(json!({ "allOf": null }), json!({ "$schema": DRAFT4 }));
}

#[test]
fn any_of_is_null() {
    assert_schema_default(json!({ "anyOf": null }), json!({ "$schema": DRAFT4 }));
}

#[test]
fn one_of_is_null() {
    assert_schema_default(json!({ "oneOf": null }), json!({ "$schema": DRAFT4 }));
}
