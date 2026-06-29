//! from_parameter branch logic.

mod common;

use common::DRAFT4;
use openapi_to_json_schema::{from_parameter, Error, Options};
use serde_json::json;

#[test]
fn minimal_parameter() {
    let param = json!({
        "name": "parameter name",
        "in": "cookie",
        "schema": { "type": "string", "nullable": true }
    });
    let result = from_parameter(param, &Options::new()).unwrap();
    assert_eq!(
        result,
        json!({ "$schema": DRAFT4, "type": ["string", "null"] })
    );
}

#[test]
fn extensive_parameter() {
    let param = json!({
        "name": "parameter name",
        "in": "cookie",
        "schema": { "type": "string", "nullable": true },
        "required": true,
        "allowEmptyValue": true,
        "deprecated": true,
        "allowReserved": true,
        "style": "matrix",
        "explode": true,
        "example": "parameter example"
    });
    let result = from_parameter(param, &Options::new()).unwrap();
    assert_eq!(
        result,
        json!({ "$schema": DRAFT4, "type": ["string", "null"] })
    );
}

#[test]
fn parameter_with_mime_schemas() {
    let param = json!({
        "name": "parameter name",
        "in": "cookie",
        "content": {
            "application/javascript": { "schema": { "type": "string", "nullable": true } },
            "text/css": { "schema": { "type": "string", "nullable": true } }
        }
    });
    let result = from_parameter(param, &Options::new()).unwrap();
    assert_eq!(
        result,
        json!({
            "application/javascript": { "$schema": DRAFT4, "type": ["string", "null"] },
            "text/css": { "$schema": DRAFT4, "type": ["string", "null"] }
        })
    );
}

#[test]
fn parameter_with_mimes_without_a_schema() {
    let param = json!({
        "name": "parameter name",
        "in": "cookie",
        "content": {
            "application/javascript": { "schema": { "type": "string", "nullable": true } },
            "text/css": {}
        }
    });
    let result = from_parameter(param, &Options::new()).unwrap();
    assert_eq!(
        result,
        json!({
            "application/javascript": { "$schema": DRAFT4, "type": ["string", "null"] },
            "text/css": { "$schema": DRAFT4 }
        })
    );
}

#[test]
fn using_parameter_description() {
    let param = json!({
        "name": "parameter name",
        "in": "cookie",
        "description": "parameter description",
        "schema": { "description": "schema description" }
    });
    let result = from_parameter(param, &Options::new()).unwrap();
    assert_eq!(
        result,
        json!({ "$schema": DRAFT4, "description": "parameter description" })
    );
}

#[test]
fn throwing_on_parameters_without_schemas() {
    let param = json!({ "name": "parameter name", "in": "cookie" });
    let err = from_parameter(param, &Options::new()).unwrap_err();
    assert!(err.to_string().contains("parameter must have either a"));
    assert!(matches!(err, Error::InvalidInput(_)));
}

#[test]
fn lenient_parameter_without_schema() {
    let param = json!({ "name": "parameter name", "in": "cookie" });
    let options = Options {
        strict_mode: Some(false),
        ..Options::new()
    };
    let result = from_parameter(param, &options).unwrap();
    assert_eq!(result, json!({ "$schema": DRAFT4 }));
}
