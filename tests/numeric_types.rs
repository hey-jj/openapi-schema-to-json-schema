//! Numeric format range injection.
//!
//! Bounds are f64, matching the source `2 ** n` arithmetic. The expected
//! goldens build float `Value`s the same way so the Number representation
//! agrees. For `double + minimum`, the source golden is `Number.MAX_VALUE - 1`,
//! which equals `f64::MAX` because 1 is below the ULP at that magnitude.

mod common;

use common::{assert_schema_default, DRAFT4};
use serde_json::{json, Value};

fn min_i31() -> f64 {
    -(2f64.powi(31))
}
fn max_i31() -> f64 {
    2f64.powi(31) - 1.0
}
fn min_i63() -> f64 {
    -(2f64.powi(63))
}
fn max_i63() -> f64 {
    2f64.powi(63) - 1.0
}
fn min_f128() -> f64 {
    -(2f64.powi(128))
}
fn max_f128() -> f64 {
    2f64.powi(128) - 1.0
}

fn num(v: f64) -> Value {
    json!(v)
}

// ---- int32 ----------------------------------------------------------------

#[test]
fn handles_int32_format() {
    assert_schema_default(
        json!({ "type": "integer", "format": "int32" }),
        json!({
            "$schema": DRAFT4, "type": "integer", "format": "int32",
            "minimum": num(min_i31()), "maximum": num(max_i31())
        }),
    );
}

#[test]
fn int32_with_specified_minimum() {
    assert_schema_default(
        json!({ "type": "integer", "format": "int32", "minimum": 500 }),
        json!({
            "$schema": DRAFT4, "type": "integer", "format": "int32",
            "minimum": 500, "maximum": num(max_i31())
        }),
    );
}

#[test]
fn int32_with_minimum_too_small() {
    assert_schema_default(
        json!({ "type": "integer", "format": "int32", "minimum": num(-(2f64.powi(32))) }),
        json!({
            "$schema": DRAFT4, "type": "integer", "format": "int32",
            "minimum": num(min_i31()), "maximum": num(max_i31())
        }),
    );
}

#[test]
fn int32_with_specified_maximum() {
    assert_schema_default(
        json!({ "type": "integer", "format": "int32", "maximum": 500 }),
        json!({
            "$schema": DRAFT4, "type": "integer", "format": "int32",
            "minimum": num(min_i31()), "maximum": 500
        }),
    );
}

#[test]
fn int32_with_maximum_too_big() {
    assert_schema_default(
        json!({ "type": "integer", "format": "int32", "maximum": num(2f64.powi(32)) }),
        json!({
            "$schema": DRAFT4, "type": "integer", "format": "int32",
            "minimum": num(min_i31()), "maximum": num(max_i31())
        }),
    );
}

// ---- int64 ----------------------------------------------------------------

#[test]
fn handles_int64_format() {
    assert_schema_default(
        json!({ "type": "integer", "format": "int64" }),
        json!({
            "$schema": DRAFT4, "type": "integer", "format": "int64",
            "minimum": num(min_i63()), "maximum": num(max_i63())
        }),
    );
}

#[test]
fn int64_with_specified_minimum() {
    assert_schema_default(
        json!({ "type": "integer", "format": "int64", "minimum": 500 }),
        json!({
            "$schema": DRAFT4, "type": "integer", "format": "int64",
            "minimum": 500, "maximum": num(max_i63())
        }),
    );
}

#[test]
fn int64_with_minimum_too_small() {
    assert_schema_default(
        json!({ "type": "integer", "format": "int64", "minimum": num(-(2f64.powi(64))) }),
        json!({
            "$schema": DRAFT4, "type": "integer", "format": "int64",
            "minimum": num(min_i63()), "maximum": num(max_i63())
        }),
    );
}

#[test]
fn int64_with_specified_maximum() {
    assert_schema_default(
        json!({ "type": "integer", "format": "int64", "maximum": 500 }),
        json!({
            "$schema": DRAFT4, "type": "integer", "format": "int64",
            "minimum": num(min_i63()), "maximum": 500
        }),
    );
}

#[test]
fn int64_with_maximum_too_big() {
    assert_schema_default(
        json!({ "type": "integer", "format": "int64", "maximum": num(2f64.powi(64)) }),
        json!({
            "$schema": DRAFT4, "type": "integer", "format": "int64",
            "minimum": num(min_i63()), "maximum": num(max_i63())
        }),
    );
}

// ---- float ----------------------------------------------------------------

#[test]
fn handles_float_format() {
    assert_schema_default(
        json!({ "type": "number", "format": "float" }),
        json!({
            "$schema": DRAFT4, "type": "number", "format": "float",
            "minimum": num(min_f128()), "maximum": num(max_f128())
        }),
    );
}

#[test]
fn float_with_specified_minimum() {
    assert_schema_default(
        json!({ "type": "number", "format": "float", "minimum": 500 }),
        json!({
            "$schema": DRAFT4, "type": "number", "format": "float",
            "minimum": 500, "maximum": num(max_f128())
        }),
    );
}

#[test]
fn float_with_minimum_too_small() {
    assert_schema_default(
        json!({ "type": "number", "format": "float", "minimum": num(-(2f64.powi(129))) }),
        json!({
            "$schema": DRAFT4, "type": "number", "format": "float",
            "minimum": num(min_f128()), "maximum": num(max_f128())
        }),
    );
}

#[test]
fn float_with_specified_maximum() {
    assert_schema_default(
        json!({ "type": "number", "format": "float", "maximum": 500 }),
        json!({
            "$schema": DRAFT4, "type": "number", "format": "float",
            "minimum": num(min_f128()), "maximum": 500
        }),
    );
}

#[test]
fn float_with_maximum_too_big() {
    assert_schema_default(
        json!({ "type": "number", "format": "float", "maximum": num(2f64.powi(129)) }),
        json!({
            "$schema": DRAFT4, "type": "number", "format": "float",
            "minimum": num(min_f128()), "maximum": num(max_f128())
        }),
    );
}

// ---- double ---------------------------------------------------------------

#[test]
fn handles_double_format() {
    assert_schema_default(
        json!({ "type": "number", "format": "double" }),
        json!({
            "$schema": DRAFT4, "type": "number", "format": "double",
            "minimum": num(-f64::MAX), "maximum": num(f64::MAX)
        }),
    );
}

#[test]
fn double_with_specified_minimum() {
    // The source golden maximum is Number.MAX_VALUE - 1, which equals f64::MAX.
    assert_schema_default(
        json!({ "type": "number", "format": "double", "minimum": 50.5 }),
        json!({
            "$schema": DRAFT4, "type": "number", "format": "double",
            "minimum": 50.5, "maximum": num(f64::MAX - 1.0)
        }),
    );
}

#[test]
fn double_with_specified_maximum() {
    assert_schema_default(
        json!({ "type": "number", "format": "double", "maximum": 50.5 }),
        json!({
            "$schema": DRAFT4, "type": "number", "format": "double",
            "minimum": num(-f64::MAX), "maximum": 50.5
        }),
    );
}

// ---- boundary preservation ------------------------------------------------

#[test]
fn minimum_zero_preserved() {
    // 0 counts as present, so it is kept (not clamped to the lower bound).
    assert_schema_default(
        json!({ "type": "integer", "format": "int32", "minimum": 0 }),
        json!({
            "$schema": DRAFT4, "type": "integer", "format": "int32",
            "minimum": 0, "maximum": num(max_i31())
        }),
    );
}

#[test]
fn maximum_zero_preserved() {
    assert_schema_default(
        json!({ "type": "integer", "format": "int32", "maximum": 0 }),
        json!({
            "$schema": DRAFT4, "type": "integer", "format": "int32",
            "minimum": num(min_i31()), "maximum": 0
        }),
    );
}

#[test]
fn int64_bound_is_float_value() {
    let got = openapi_to_json_schema::from_schema(
        json!({ "type": "integer", "format": "int64" }),
        &openapi_to_json_schema::Options::new(),
    )
    .unwrap();
    assert_eq!(got["maximum"], json!(2f64.powi(63) - 1.0));
}
