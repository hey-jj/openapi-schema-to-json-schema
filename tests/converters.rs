//! Named-export smoke test plus empty-input baseline.

mod common;

use common::{assert_schema_default, DRAFT4};
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
