# tpt-construction

Dual-licensed **MIT OR Apache-2.0** · TPT Solutions

A modular Rust workspace providing reusable libraries for the construction
industry: BIM/IFC, quantity takeoff, estimating, scheduling, project controls,
field management, civil/earthwork, equipment telemetry, safety, documents,
contracts, facility management, digital twins, and WebAssembly tooling.

`tpt-construction` is the construction domain layer of the TPT ecosystem,
built on top of `tpt-math` and `tpt-engineering`.

## Crate naming

All published crates use the `tpt-c-` prefix. Rust import names use underscores:

```rust
use tpt_c_core::ProjectId;
use tpt_c_ifc::IfcProject;
use tpt_c_quantities::TakeoffEngine;
use tpt_c_estimating::EstimateBuilder;
```

## Workspace layout

```text
tpt-construction/
├── Cargo.toml            # virtual workspace manifest
├── crates/               # tpt-c-* libraries
├── examples/             # runnable end-to-end examples
└── test-data/            # fixtures: ifc/, bcf/, las/, csv/, golden/
```

## Crates

| Layer | Crates |
|---|---|
| Foundation | `tpt-c-core`, `tpt-c-ids`, `tpt-c-units`, `tpt-c-classification`, `tpt-c-model` |
| Geometry & formats | `tpt-c-geometry`, `tpt-c-geo`, `tpt-c-ifc`, `tpt-c-bcf`, `tpt-c-gltf`, `tpt-c-las`, `tpt-c-dxf` |
| Estimating | `tpt-c-quantities`, `tpt-c-cost`, `tpt-c-estimating`, `tpt-c-change`, `tpt-c-csv`, `tpt-c-xlsx` |
| Scheduling & controls | `tpt-c-schedule`, `tpt-c-risk`, `tpt-c-earned-value` |
| Civil | `tpt-c-earthwork`, `tpt-c-alignment` |
| Field execution | `tpt-c-field`, `tpt-c-workflow`, `tpt-c-documents`, `tpt-c-contracts`, `tpt-c-payapps`, `tpt-c-safety` |
| Equipment & sync | `tpt-c-equipment`, `tpt-c-sync`, `tpt-c-events`, `tpt-c-db` |
| FM & twins | `tpt-c-fm`, `tpt-c-assets`, `tpt-c-maintenance`, `tpt-c-twin`, `tpt-c-space` |
| Web platform | `tpt-c-wasm`, `tpt-c-api` |

## Industry coverage

| Industry Domain | Covered By |
|---|---|
| BIM / model coordination | `tpt-c-ifc`, `tpt-c-bcf`, `tpt-c-gltf`, `tpt-c-model` |
| Quantity takeoff | `tpt-c-quantities`, `tpt-c-geometry`, `tpt-c-model` |
| Estimating | `tpt-c-cost`, `tpt-c-estimating`, `tpt-c-classification` |
| Cost control | `tpt-c-cost`, `tpt-c-earned-value`, `tpt-c-change` |
| Scheduling | `tpt-c-schedule`, `tpt-c-risk`, `tpt-c-earned-value` |
| Field management | `tpt-c-field`, `tpt-c-workflow`, `tpt-c-documents` |
| RFIs / submittals | `tpt-c-workflow`, `tpt-c-documents` |
| Contracts | `tpt-c-contracts`, `tpt-c-payapps` |
| Payment applications | `tpt-c-payapps` |
| Safety | `tpt-c-safety`, `tpt-c-field` |
| Civil / earthwork | `tpt-c-geo`, `tpt-c-earthwork`, `tpt-c-alignment` |
| Survey / point clouds | `tpt-c-las`, `tpt-c-geo` |
| Equipment / IoT | `tpt-c-equipment`, `tpt-c-twin` |
| Facility management | `tpt-c-fm`, `tpt-c-assets`, `tpt-c-maintenance` |
| Digital twins | `tpt-c-twin`, `tpt-c-equipment`, `tpt-c-assets` |
| Offline field apps | `tpt-c-sync`, `tpt-c-db` |
| Web tooling | `tpt-c-wasm`, `tpt-c-gltf` |
| Reporting / exports | `tpt-c-csv`, `tpt-c-xlsx` |

## Design principles

- **Neutral model first** — file formats parse into `tpt-c-model`; domain
  engines consume `tpt-c-model` and never depend directly on file formats.
- **Pure core crates** — deterministic, testable, serializable,
  platform-independent; IO stays at the edges.
- **Event-first auditability** — important changes are representable as
  domain events (`QuantityAdjusted`, `RFIAnswered`, ...).
- **Offline-first field support** — assume poor connectivity; local storage,
  CRDT merge, retry queues.
- **WASM as a first-class target** — engines compile to `wasm32-unknown-unknown`.

## Development

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
cargo deny check licenses
cargo deny check bans
cargo deny check sources
```

### Source file headers

Every `.rs` file starts with (see [docs/source-file-header.txt](docs/source-file-header.txt)):

```rust
// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0
```

### Dependency policy

Prefer an MIT-selectable dependency chain (spec §4): MIT, `MIT OR Apache-2.0`,
`Unicode-DFS-2016`, and `Unicode-3.0` dependencies are allowed by default;
Apache-only/BSD/ISC/Zlib require review and must be isolated behind optional
Cargo features when unavoidable. Copyleft licenses are banned. Any unavoidable
Apache-only dependency will be replaced with an alternative implemented from
scratch where practical.

## License

Licensed under either of

- MIT license
- Apache License, Version 2.0

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this repository shall be dual licensed as MIT OR Apache-2.0,
without any additional terms or conditions.

## Contribution License Agreement

```text
By submitting a contribution to this repository, you agree that your
contribution is licensed under either the MIT license or the Apache
License, Version 2.0, at the option of the project maintainers and
downstream users.
```