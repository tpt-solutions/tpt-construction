# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- Re-exports of the engine stack: `Project`, `Element`, `PropertySet`,
  `PropertyValue`, `Quantity`, `QuantitySet` (model); `parse`, `to_model`,
  `IfcError` (IFC); `TakeoffEngine`, `TakeoffResult` (quantities);
  `Activity`, `Schedule` (scheduling); `BoundingBox`, `Mesh`, `Point3`,
  `Vec3` (geometry); `export` (glTF).
- `WasmResult<T>` serializable result envelope with `ok` and `err`
  constructors.
- `js` feature (default) with the `parse_ifc` `#[wasm_bindgen]` export that
  converts IFC text into the neutral project model as JSON, plus the `web`
  feature alias; both are optional so native consumers can build with
  `--no-default-features`.
