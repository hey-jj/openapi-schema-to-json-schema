# Changelog

## [0.2.0] - 2026-07-07

### Changed
- Schemas with `format: "int64"` now emit exact `i64::MIN` and `i64::MAX` JSON integer bounds. (#18)
- Schemas with `format: "float"` now emit binary32 finite bounds. (#19)
- Schemas with `type: "null"` and `nullable: true` now return one `null` type entry. (#20)

## [0.2.0] - 2026-07-07

### Changed
- Schemas with `format: "int64"` now emit exact `i64::MIN` and `i64::MAX` JSON integer bounds. (#18)
- Schemas with `format: "float"` now emit binary32 finite bounds. (#19)
- Schemas with `type: "null"` and `nullable: true` now return one `null` type entry. (#20)
