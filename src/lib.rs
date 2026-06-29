//! Convert OpenAPI 3.0 schema objects to JSON Schema draft-04.
//!
//! OpenAPI 3.0 describes schemas with an extended subset of JSON Schema that is
//! not itself valid JSON Schema draft-04. This crate bridges the gap. It takes
//! an OpenAPI 3.0 Schema Object as a [`serde_json::Value`] and returns a
//! draft-04 document.
//!
//! The conversion is pure and in memory. It performs no I/O, no network access,
//! and no `$ref` resolution. References pass through untouched, so resolve them
//! before calling if you need them inlined.
//!
//! # What it does
//!
//! - `nullable: true` becomes `"null"` added to `type`, and `null` appended to
//!   an `enum` when present.
//! - OpenAPI-only keywords are stripped: `nullable`, `discriminator`,
//!   `readOnly`, `writeOnly`, `xml`, `externalDocs`, `example`, `deprecated`.
//! - Numeric formats (`int32`, `int64`, `float`, `double`) become
//!   `minimum`/`maximum` bounds. `byte` becomes a base64 `pattern`. `date`
//!   optionally becomes `date-time`.
//! - Combiner and struct keywords recurse: `allOf`, `anyOf`, `oneOf`, `not`,
//!   `items`, `additionalProperties`, and `properties`.
//! - `required` is pruned of names no longer present in `properties`.
//! - In strict mode, an invalid `type` raises an error.
//!
//! The root of the result carries `"$schema": "http://json-schema.org/draft-04/schema#"`.
//! Nested schemas do not.
//!
//! # Example
//!
//! ```
//! use openapi_to_json_schema::{from_schema, Options};
//! use serde_json::json;
//!
//! let input = json!({ "type": "string", "nullable": true });
//! let output = from_schema(input, &Options::new()).unwrap();
//! assert_eq!(
//!     output,
//!     json!({
//!         "type": ["string", "null"],
//!         "$schema": "http://json-schema.org/draft-04/schema#"
//!     })
//! );
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod consts;
mod error;
mod options;
mod parameter;
mod pattern;
mod schema;
mod value;

pub use error::Error;
pub use options::{
    AfterTransform, BeforeTransform, Options, PatternPropertiesHandler, ResolvedOptions,
};

use serde_json::Value;

/// Convert an OpenAPI 3.0 Schema Object to a JSON Schema draft-04 document.
///
/// The root of the returned value carries `$schema`. The input is not mutated.
///
/// # Errors
///
/// Returns [`Error::InvalidType`] when strict mode is on and any node carries a
/// `type` value outside the draft-04 type set.
///
/// # Example
///
/// ```
/// use openapi_to_json_schema::{from_schema, Options};
/// use serde_json::json;
///
/// let out = from_schema(json!({ "type": "integer", "format": "int32" }), &Options::new()).unwrap();
/// assert_eq!(out["minimum"], json!(-2147483648_i64));
/// assert_eq!(out["maximum"], json!(2147483647_i64));
/// ```
pub fn from_schema(schema: Value, options: &Options) -> Result<Value, Error> {
    let resolved = options::resolve_options(options);
    schema::convert_from_schema(schema, &resolved)
}

/// Convert an OpenAPI 3.0 Parameter Object or Response Object.
///
/// With a `schema` member the result is a single JSON Schema. With a `content`
/// member the result is a map keyed by MIME type, where each value is a JSON
/// Schema with its own `$schema`. The outer map has no `$schema`.
///
/// A truthy `description` on the parameter is copied onto each result.
///
/// # Errors
///
/// Returns [`Error::InvalidInput`] when strict mode is on and the parameter has
/// neither a `schema` nor a `content` member. Returns [`Error::InvalidType`]
/// under the same conditions as [`from_schema`].
///
/// # Example
///
/// ```
/// use openapi_to_json_schema::{from_parameter, Options};
/// use serde_json::json;
///
/// let param = json!({
///     "name": "id",
///     "in": "query",
///     "schema": { "type": "string", "nullable": true }
/// });
/// let out = from_parameter(param, &Options::new()).unwrap();
/// assert_eq!(out["type"], json!(["string", "null"]));
/// ```
pub fn from_parameter(parameter: Value, options: &Options) -> Result<Value, Error> {
    let resolved = options::resolve_options(options);
    parameter::convert_from_parameter(parameter, &resolved)
}
