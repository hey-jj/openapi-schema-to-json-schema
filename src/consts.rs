//! Constant tables that drive conversion.

/// The `$schema` value written on the root of every converted schema.
pub(crate) const DRAFT_04: &str = "http://json-schema.org/draft-04/schema#";

/// OpenAPI-only keywords stripped after transform, in strip order.
pub(crate) const NOT_SUPPORTED: &[&str] = &[
    "nullable",
    "discriminator",
    "readOnly",
    "writeOnly",
    "xml",
    "externalDocs",
    "example",
    "deprecated",
];

/// Keywords recursed into as sub-schemas, in recursion order.
pub(crate) const STRUCTS: &[&str] = &[
    "allOf",
    "anyOf",
    "oneOf",
    "not",
    "items",
    "additionalProperties",
];

/// Formats already valid in JSON Schema draft-04. These pass through untouched.
pub(crate) const VALID_OPENAPI_FORMATS: &[&str] = &[
    "date-time",
    "email",
    "hostname",
    "ipv4",
    "ipv6",
    "uri",
    "uri-reference",
];

/// The draft-04 type set accepted in strict mode.
pub(crate) const VALID_TYPES: &[&str] = &[
    "integer", "number", "string", "boolean", "object", "array", "null",
];

/// Base64 pattern written for `format: "byte"`. RFC 4648 standard alphabet.
pub(crate) const BYTE_PATTERN: &str = "^[\\w\\d+\\/=]*$";
