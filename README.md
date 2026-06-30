# openapi-schema-to-json-schema

Convert an OpenAPI 3.0 Schema Object into a JSON Schema draft-04 document.

OpenAPI 3.0 describes schemas with an extended subset of JSON Schema that is
not itself valid draft-04. This crate bridges the gap. It is pure and in
memory. It performs no I/O, no network access, and no `$ref` resolution.

## Install

```toml
[dependencies]
openapi-schema-to-json-schema = "0.1"
serde_json = "1"
```

## Use

```rust
use openapi_schema_to_json_schema::{from_schema, Options};
use serde_json::json;

let input = json!({ "type": "string", "nullable": true });
let output = from_schema(input, &Options::new()).unwrap();

assert_eq!(
    output,
    json!({
        "type": ["string", "null"],
        "$schema": "http://json-schema.org/draft-04/schema#"
    })
);
```

## What it does

- `nullable: true` adds `"null"` to `type` and appends `null` to an `enum`.
- Strips OpenAPI-only keywords: `nullable`, `discriminator`, `readOnly`,
  `writeOnly`, `xml`, `externalDocs`, `example`, `deprecated`.
- Maps numeric formats to bounds: `int32`, `int64`, `float`, `double` become
  `minimum` and `maximum`. `byte` becomes a base64 `pattern`. `date` becomes
  `date-time` when `date_to_date_time` is on.
- Recurses combiners and structs: `allOf`, `anyOf`, `oneOf`, `not`, `items`,
  `additionalProperties`, and `properties`.
- Prunes `required` of names no longer present in `properties`.
- Rejects an invalid `type` in strict mode.

The root of the result carries `"$schema"`. Nested schemas do not.

## Parameters and responses

`from_parameter` converts a Parameter Object or Response Object. With a
`schema` member it returns one JSON Schema. With a `content` member it returns
a map keyed by MIME type, where each value is a JSON Schema with its own
`$schema`.

```rust
use openapi_schema_to_json_schema::{from_parameter, Options};
use serde_json::json;

let param = json!({
    "name": "id",
    "in": "query",
    "schema": { "type": "string", "nullable": true }
});
let out = from_parameter(param, &Options::new()).unwrap();
assert_eq!(out["type"], json!(["string", "null"]));
```

## Options

| Option | Default | Effect |
|---|---|---|
| `date_to_date_time` | false | Rewrite `format: "date"` to `date-time`. |
| `clone_schema` | true | Kept for API parity. The crate owns its input, so conversion never mutates the caller's data. |
| `support_pattern_properties` | false | Move `x-patternProperties` to `patternProperties` and run the handler. |
| `keep_not_supported` | empty | Keep listed keywords that would otherwise be stripped. |
| `strict_mode` | true | Reject `type` values outside the draft-04 set. |
| `remove_read_only` | false | Drop properties marked `readOnly: true`. |
| `remove_write_only` | false | Drop properties marked `writeOnly: true`. |
| `pattern_properties_handler` | dedup handler | Replace the default `patternProperties` handler. |
| `definition_keywords` | empty | Dotted paths whose values hold named sub-schemas to convert. |
| `before_transform` | none | Hook run on each node before keyword processing. |
| `after_transform` | none | Hook run on each node after keyword processing. |

## License

Licensed under the [MIT license](LICENSE).
