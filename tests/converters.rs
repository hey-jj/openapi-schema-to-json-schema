//! Named-export smoke test plus empty-input baseline.

mod common;

use common::{assert_schema_default, DRAFT4};
use openapi_to_json_schema::{from_schema, Error, Options};
use serde_json::json;

#[test]
fn using_from_schema() {
    assert_schema_default(
        json!({ "type": "string", "nullable": true }),
        json!({ "$schema": DRAFT4, "type": ["string", "null"] }),
    );
}

#[test]
fn empty_input() {
    assert_schema_default(json!({}), json!({ "$schema": DRAFT4 }));
}

#[test]
fn scalar_root_is_rejected() {
    for root in [json!("hello"), json!(42), json!(true), json!(null)] {
        let err = from_schema(root.clone(), &Options::new()).expect_err("scalar root must fail");
        assert!(
            matches!(err, Error::InvalidInput(_)),
            "root {root}: {err:?}"
        );
        assert!(err.to_string().contains("must be an object"));
    }
}

#[test]
fn array_root_passes_through() {
    // An array carries no schema keywords, so it is returned as given. No
    // $schema is added, since only an object root takes the member.
    let root = json!([{ "type": "string" }]);
    let out = from_schema(root.clone(), &Options::new()).unwrap();
    assert_eq!(out, root);
}
