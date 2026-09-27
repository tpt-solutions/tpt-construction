# wasm-browser-demo

Phase 9 integration example: a native Rust binary that simulates how a browser
front end would consume the WASM-facing API. It imports an IFC file into the
neutral `tpt-c-model` project (the same path `tpt-c-wasm` exposes to
JavaScript), runs the quantity takeoff engine, and formats the measured
quantities for display, exercising `tpt-c-wasm`'s `WasmResult` envelope at the
boundary.

Part of the [tpt-construction](../../README.md) workspace.

## Run it

```bash
cargo run -p wasm-browser-demo
# or with an explicit IFC file
cargo run -p wasm-browser-demo -- path/to/model.ifc
```

Without an argument the bundled `test-data/ifc/sample.ifc` fixture is used.
Output for that fixture:

```text
Project imported: Riverside Office Building
Elements: 4
Takeoff items: 4
  - [<element id>] East Wall (Wall)
      Length: net=20.00 m gross=<gross> m
      GrossFootprintArea: net=60.00 m² gross=<gross> m²
      ...
```

Each takeoff item also prints its `Count`/`Length`/`Area`/`Volume`/`Mass`
quantities formatted as `each`/`m`/`m²`/`m³`/`kg`. The demo finishes by
round-tripping `WasmResult::ok`/`WasmResult::err` values through the same
result envelope the WASM glue serializes for JavaScript callers.

## What it exercises

- `tpt_c_wasm` — the `WasmResult` success/error envelope used at the
  JS-to-Rust boundary.
- `tpt_c_ifc` — `parse` + `to_model`, the same import pipeline the WASM module
  exposes to the browser.
- `tpt_c_model::Project` — the neutral model handed to the takeoff engine.
- `tpt_c_quantities` — `TakeoffEngine` and `MeasuredQuantity` formatting.
- Error handling for unreadable files and failed imports, reported on stderr
  with a nonzero exit code.

## License

Dual-licensed `MIT OR Apache-2.0`.
