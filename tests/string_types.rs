//! String format handling.

mod common;

use common::{assert_schema, assert_schema_default, DRAFT4};
use openapi_schema_to_json_schema::Options;
use serde_json::json;

#[test]
fn plain_string_is_untouched() {
    assert_schema_default(
        json!({ "type": "string" }),
        json!({ "$schema": DRAFT4, "type": "string" }),
    );
}

#[test]
fn handles_date() {
    assert_schema_default(
        json!({ "type": "string", "format": "date" }),
        json!({ "$schema": DRAFT4, "type": "string", "format": "date" }),
    );

    let options = Options {
        date_to_date_time: Some(true),
        ..Options::new()
    };
    assert_schema(
        json!({ "type": "string", "format": "date" }),
        &options,
        json!({ "$schema": DRAFT4, "type": "string", "format": "date-time" }),
    );
}

#[test]
fn handles_byte_format() {
    assert_schema_default(
        json!({ "type": "string", "format": "byte" }),
        json!({
            "$schema": DRAFT4,
            "type": "string",
            "format": "byte",
            "pattern": "^[\\w\\d+\\/=]*$"
        }),
    );
}

#[test]
fn retaining_custom_formats() {
    assert_schema_default(
        json!({ "type": "string", "format": "custom_email" }),
        json!({ "$schema": DRAFT4, "type": "string", "format": "custom_email" }),
    );
}

#[test]
fn retain_password_format() {
    assert_schema_default(
        json!({ "type": "string", "format": "password" }),
        json!({ "$schema": DRAFT4, "type": "string", "format": "password" }),
    );
}

#[test]
fn retain_binary_format() {
    assert_schema_default(
        json!({ "type": "string", "format": "binary" }),
        json!({ "$schema": DRAFT4, "type": "string", "format": "binary" }),
    );
}

#[test]
fn date_time_passes_through() {
    // date-time is a valid draft-04 format. It is left alone even without
    // date_to_date_time.
    assert_schema_default(
        json!({ "type": "string", "format": "date-time" }),
        json!({ "$schema": DRAFT4, "type": "string", "format": "date-time" }),
    );
}

#[test]
fn byte_pattern_applies_regardless_of_type() {
    // The byte pattern is driven by format alone, with no type guard.
    assert_schema_default(
        json!({ "format": "byte" }),
        json!({ "$schema": DRAFT4, "format": "byte", "pattern": "^[\\w\\d+\\/=]*$" }),
    );
}

#[test]
fn valid_draft04_formats_pass_through() {
    // These formats are already valid in draft-04. Each is returned with format
    // unchanged and no minimum, maximum, or pattern added.
    for fmt in ["email", "hostname", "ipv4", "ipv6", "uri", "uri-reference"] {
        assert_schema_default(
            json!({ "type": "string", "format": fmt }),
            json!({ "$schema": DRAFT4, "type": "string", "format": fmt }),
        );
    }
}
