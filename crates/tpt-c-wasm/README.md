# tpt-c-wasm

WASM bindings for the tpt-construction engines. The crate re-exports the
pure-Rust engines — the neutral project model, IFC parsing, quantity takeoff,
CPM scheduling, geometry types and glTF export — behind a single façade, and
adds a thin `wasm-bindgen` layer (gated behind the `js` feature) so the same
functionality can be consumed from JavaScript. Reach for it when you want one
dependency that exposes the engine stack for `wasm32-unknown-unknown` builds
or for native Rust callers that want the re-exports without pulling in
`wasm-bindgen` (build with `--no-default-features`).

Part of the [tpt-construction](../../README.md) workspace.

## Features

- Re-exports the neutral model: `Project`, `Element`, `PropertySet`,
  `PropertyValue`, `Quantity`, `QuantitySet`.
- Re-exports IFC parsing: `parse`, `to_model` and `IfcError`.
- Re-exports quantity takeoff: `TakeoffEngine` and `TakeoffResult`.
- Re-exports scheduling (`Activity`, `Schedule`), geometry (`BoundingBox`,
  `Mesh`, `Point3`, `Vec3`) and glTF `export`.
- `WasmResult<T>`: a serializable ok/value/error envelope for crossing the
  WASM boundary with `WasmResult::ok` and `WasmResult::err`.
- `js` feature (enabled by default): `parse_ifc(source) -> Result<JsValue,
  JsValue>` `#[wasm_bindgen]` export that turns IFC text into the neutral
  project model as JSON; the `web` feature is an alias enabling the same
  optional stack.

## Usage

```toml
[dependencies]
tpt-c-wasm = "0.1"
```

```rust
use tpt_c_wasm::{parse, to_model, WasmResult};

// Parse IFC text into the neutral model and wrap it for a WASM boundary
// (same flow as examples/wasm-browser-demo).
fn import_ifc(source: &str) -> WasmResult<tpt_c_wasm::Project> {
    let doc = parse(source).map_err(|e| e.to_string())?;
    let project = to_model(&doc).map_err(|e| e.to_string())?;
    WasmResult::ok(project)
}

let boundary: WasmResult<i32> = WasmResult::err("boom");
assert_eq!(boundary.error.as_deref(), Some("boom"));
```

Build for the browser with the default `js` feature:

```bash
cargo build -p tpt-c-wasm --target wasm32-unknown-unknown
```

## Crate relationships

- **Depends on:** `serde`, `tpt-c-core`, `tpt-c-ifc`, `tpt-c-model`, `tpt-c-quantities`, `tpt-c-schedule`, `tpt-c-geometry`, `tpt-c-gltf`; optional `wasm-bindgen`, `serde-wasm-bindgen`, `web-sys`.
- **Used by:** `examples/wasm-browser-demo` (consumes `tpt_c_wasm::WasmResult` and the re-exported engines).

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) / [LICENSE-APACHE](../../LICENSE-APACHE)).
