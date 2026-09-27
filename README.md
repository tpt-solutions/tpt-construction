# tpt-construction

[![CI](https://github.com/tpt-solutions/tpt-construction/actions/workflows/ci.yml/badge.svg)](https://github.com/tpt-solutions/tpt-construction/actions/workflows/ci.yml)
[![License headers](https://github.com/tpt-solutions/tpt-construction/actions/workflows/license.yml/badge.svg)](https://github.com/tpt-solutions/tpt-construction/actions/workflows/license.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Dual-licensed **MIT OR Apache-2.0** · TPT Solutions

A modular Rust workspace providing reusable libraries for the construction
industry: BIM/IFC, quantity takeoff, estimating, scheduling, project controls,
field management, civil/earthwork, equipment telemetry, safety, documents,
contracts, facility management, digital twins, and WebAssembly tooling.

`tpt-construction` is the construction domain layer of the TPT ecosystem:

```text
tpt-math            numeric computation
        ↑
tpt-engineering     applied engineering primitives
        ↑
tpt-construction    tpt-c-* construction crates
```

The two upstream substrates are not required to build this repository today:
the crates here are self-contained, and the local `[patch.crates-io]` overrides
for `../tpt-math` / `../tpt-engineering` stay commented out in the root manifest
until those checkouts are available locally.

## Status

| Item | State |
|---|---|
| Library crates | 40 `tpt-c-*` crates — **all** workspace members, all with their own `README.md` and `CHANGELOG.md` |
| Examples / binaries | 9 — `ifc-import`, `quantity-takeoff`, `estimate-export`, `cpm-schedule`, `earthwork-cut-fill`, `field-execution-e2e`, `field-offline-sync`, `wasm-browser-demo`, and the `tpt` CLI |
| Version | `0.1.0` (pre-1.0; APIs may change between releases) |
| Toolchain | stable Rust, edition 2021 |
| Tests | `cargo test --workspace --all-features` green (222 tests) |
| License policy | `cargo deny check licenses` green — no Apache-only crates remain |
| WASM target | `tpt-c-wasm` is buildable for `wasm32-unknown-unknown` |

Every crate under `crates/` is listed in the root manifest's `members`, so
`cargo test --workspace` and all CI gates cover the entire set; a CI job
fails if a `crates/*` directory is ever left out again.

## Quick start

Requires a stable Rust toolchain and `cargo` (`rustup` recommended):

```bash
git clone https://github.com/tpt-solutions/tpt-construction.git
cd tpt-construction

cargo build --workspace        # build every workspace member
cargo test --workspace         # run the unit and integration tests
```

### CLI: price a model and export an estimate

The `tpt` binary reads a neutral model (`tpt-c-model` JSON) plus a CSV cost
database and writes a multi-sheet `.xlsx` estimate:

```bash
cargo run -p tpt -- estimate test-data/golden/sample-model.json \
    --cost-db test-data/golden/rates.csv \
    --output estimate.xlsx
```

```text
Estimate written to estimate.xlsx: 5 line items, subtotal 6141.10 USD, total 7295.63 USD
```

Running `tpt` with no arguments — or with `help` / `--help` / `-h` — prints the
usage summary.

### CLI: serve the engines over HTTP

With the `serve` feature the CLI also exposes takeoff, estimate, and CPM
scheduling as JSON endpoints (bearer-token auth optional):

```bash
cargo run -p tpt --features serve -- serve --addr 127.0.0.1:8090
curl -s http://127.0.0.1:8090/health        # {"status":"ok"}
```

See [cookbook/11-serve-over-http.md](cookbook/11-serve-over-http.md) for the
full endpoint list.

### Learning path

- [docs/getting-started.md](docs/getting-started.md) — the smallest
  end-to-end flow (model → takeoff → estimate → export), CLI and library.
- [cookbook/](cookbook/) — minimal, copy-paste recipes for the common
  workflows, with verified outputs.
- [CONTRIBUTING.md](CONTRIBUTING.md) — dev commands and crate conventions;
  [CHANGELOG.md](CHANGELOG.md) — what changed and when.

### Runnable examples

Each example is a workspace member; run it with `cargo run -p <name>`:

```bash
# Parse IFC (STEP) text into the neutral tpt-c-model (defaults to test-data/ifc/sample.ifc)
cargo run -p ifc-import
cargo run -p ifc-import -- path/to/model.ifc

# Quantity takeoff over a neutral model JSON
cargo run -p quantity-takeoff -- test-data/golden/sample-model.json

# Model + cost database -> estimate.xlsx
cargo run -p estimate-export -- test-data/golden/sample-model.json \
    test-data/golden/rates.csv estimate.xlsx

# CPM schedule + Monte Carlo risk + earned-value snapshot
cargo run -p cpm-schedule

# Alignment stake-out + mass haul + cut/fill balance
cargo run -p earthwork-cut-fill

# Offline CRDT sync between two field devices + event-sourced audit replay
cargo run -p field-offline-sync

# Daily log -> RFI -> transmittal -> contract notice/claim -> pay app -> safety incident
cargo run -p field-execution-e2e

# WASM-facing API demo (IFC import, takeoff, glTF export surface)
cargo run -p wasm-browser-demo
```

## Crate naming

All crates in this workspace use the `tpt-c-` prefix. Rust import names use
underscores:

```rust
use tpt_c_core::ProjectId;
use tpt_c_ifc::IfcProject;
use tpt_c_quantities::TakeoffEngine;
use tpt_c_estimating::EstimateBuilder;
```

## Workspace layout

```text
tpt-construction/
├── Cargo.toml            # virtual workspace manifest (members + workspace deps)
├── crates/               # tpt-c-* libraries (each with README.md + CHANGELOG.md)
├── examples/             # runnable end-to-end examples and the `tpt` CLI
├── cookbook/             # minimal, heavily-commented copy-paste recipes
├── docs/                 # source-file-header.txt, getting-started.md
├── scripts/              # CI helper scripts (workspace membership check)
└── test-data/            # fixtures: ifc/, bcf/, golden/
    ├── ifc/sample.ifc
    ├── bcf/markup.bcf
    └── golden/           # sample-model.json, rates.csv, estimate.xlsx
```

## Crates

Every crate has its own README (linked in the reference below), CHANGELOG,
and crates.io metadata (`categories`/`keywords`).

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

### Crate reference

| Crate | Purpose |
|---|---|
| [tpt-c-core](crates/tpt-c-core/README.md) | Core domain primitives: ids, errors, audit metadata, project context |
| [tpt-c-ids](crates/tpt-c-ids/README.md) | Deterministic/UUIDv7 id generation and external-id mapping (IFC GUID, Revit, cost codes, asset tags) |
| [tpt-c-units](crates/tpt-c-units/README.md) | Construction measurement units, conversions, rounding, waste factors, domain measures |
| [tpt-c-classification](crates/tpt-c-classification/README.md) | MasterFormat, UniFormat, OmniClass, Uniclass support and project classification mapping |
| [tpt-c-model](crates/tpt-c-model/README.md) | Neutral construction domain model: project, site, building, storey, zone, element, assembly, property/quantity sets |
| [tpt-c-geometry](crates/tpt-c-geometry/README.md) | Points, vectors, meshes, bounding boxes, spatial indexing, area/volume, clash-detection primitives |
| [tpt-c-geo](crates/tpt-c-geo/README.md) | CRS, lat/long, local site coordinates, elevations, grids, datums, transformations |
| [tpt-c-ifc](crates/tpt-c-ifc/README.md) | ISO-10303-21 (STEP) IFC parser mapping building elements to the neutral `tpt-c-model` |
| [tpt-c-bcf](crates/tpt-c-bcf/README.md) | BIM Collaboration Format topics, viewpoints, comments, statuses, assignments (XML round-trip) |
| [tpt-c-gltf](crates/tpt-c-gltf/README.md) | Export neutral models to glTF 2.0 JSON for web/mesh viewers |
| [tpt-c-las](crates/tpt-c-las/README.md) | LAS/LAZ point-cloud parsing, classification and volume estimation |
| [tpt-c-dxf](crates/tpt-c-dxf/README.md) | ASCII DXF reader for LINE, LWPOLYLINE, CIRCLE, POINT and TEXT entities |
| [tpt-c-csv](crates/tpt-c-csv/README.md) | Tabular CSV import/export for estimates, cost databases, schedules and field logs |
| [tpt-c-xlsx](crates/tpt-c-xlsx/README.md) | Excel (`.xlsx`) export for estimates, owner reports and bid summaries |
| [tpt-c-quantities](crates/tpt-c-quantities/README.md) | Takeoff engine: count/length/area/volume/weight, net vs gross, waste factors, manual overrides, derived rules |
| [tpt-c-cost](crates/tpt-c-cost/README.md) | Resource rates, cost items, assemblies, line items, markups, budgets and estimates |
| [tpt-c-estimating](crates/tpt-c-estimating/README.md) | Estimate builder, bid preparation, pricing workflows, revisions, comparison, cost planning |
| [tpt-c-change](crates/tpt-c-change/README.md) | Change orders, revision diffs, quantity/cost deltas and change registers (spec §12) |
| [tpt-c-schedule](crates/tpt-c-schedule/README.md) | Activities, relationships (FS/SS/FF/SF), calendars, constraints, CPM engine, float, baselines, actuals |
| [tpt-c-risk](crates/tpt-c-risk/README.md) | Schedule and cost risk: Monte Carlo simulation, weather and productivity uncertainty |
| [tpt-c-earned-value](crates/tpt-c-earned-value/README.md) | Earned Value Management: PV/EV/AC, SPI, CPI, TCPI, EAC, ETC, VAC |
| [tpt-c-earthwork](crates/tpt-c-earthwork/README.md) | Earthwork volumes, shrink/swell, mass haul and volume balancing |
| [tpt-c-alignment](crates/tpt-c-alignment/README.md) | Horizontal/vertical road alignments, curves, superelevation, stationing, corridor modelling |
| [tpt-c-field](crates/tpt-c-field/README.md) | Field execution records: daily logs, field reports, work records, manpower, weather, site observations |
| [tpt-c-workflow](crates/tpt-c-workflow/README.md) | Approval workflows and state machines for RFI, submittal, punch list, issue, change order, notice, transmittal |
| [tpt-c-documents](crates/tpt-c-documents/README.md) | Document control: documents, drawings, specifications, revisions, transmittals |
| [tpt-c-contracts](crates/tpt-c-contracts/README.md) | Contracts, contract items, responsibilities, notices, claims, compliance events |
| [tpt-c-payapps](crates/tpt-c-payapps/README.md) | Progress claims and payment applications: schedule of values, retention, approvals, certified amounts |
| [tpt-c-safety](crates/tpt-c-safety/README.md) | Incidents, near misses, safety observations, toolbox talks, inspections, compliance checklists |
| [tpt-c-equipment](crates/tpt-c-equipment/README.md) | Equipment registry, utilization, fuel/maintenance tracking, telematics events |
| [tpt-c-events](crates/tpt-c-events/README.md) | Domain events, audit trail, event sourcing, projections, replay |
| [tpt-c-sync](crates/tpt-c-sync/README.md) | Offline-first synchronization with CRDTs and deterministic conflict resolution |
| [tpt-c-db](crates/tpt-c-db/README.md) | Persistence patterns: repositories, storage backends and a migration framework |
| [tpt-c-fm](crates/tpt-c-fm/README.md) | Facility management: COBie handover data, asset registers, spaces and systems |
| [tpt-c-assets](crates/tpt-c-assets/README.md) | Asset lifecycle, warranties, serial numbers, replacements, maintenance schedules |
| [tpt-c-maintenance](crates/tpt-c-maintenance/README.md) | Preventive/corrective maintenance, work orders, service history |
| [tpt-c-twin](crates/tpt-c-twin/README.md) | Digital twin state, sensor mapping, live telemetry, spatial asset context |
| [tpt-c-space](crates/tpt-c-space/README.md) | Spaces, occupancy, leases, areas, space planning, utilization |
| [tpt-c-wasm](crates/tpt-c-wasm/README.md) | WASM bindings for IFC parsing and glTF export (pure engines re-exported for wasm32 builds) |
| [tpt-c-api](crates/tpt-c-api/README.md) | REST/GraphQL API models, auth patterns, authorization, project access control, webhooks, audit logging |

## Industry coverage

The crate set targets at least 80% of common construction software needs
(spec §19).

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
| Web tooling | `tpt-c-wasm`, `tpt-c-gltf`, `tpt-c-api` |
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

Toolchain: stable Rust; the minimum supported Rust version is **1.82**, declared
as `msrv` in [clippy.toml](clippy.toml).

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo deny check                                     # licenses, bans, sources
```

> **Windows checkouts:** there is no `.gitattributes`, and
> [rustfmt.toml](rustfmt.toml) sets `newline_style = "Unix"`, so a working tree
> created with `core.autocrlf=true` makes `cargo fmt --all --check` report
> `Incorrect newline style` for every file. Either re-checkout with
> `git config core.autocrlf input`, or run `cargo fmt --all` to normalize the
> tree first. CI runs on Linux, where the files are LF.

### Continuous integration

| Job | Command |
|---|---|
| Workspace membership | `bash scripts/check-workspace-members.sh` — fails if a `crates/*` directory is missing from `[workspace] members` (or a listed member is missing on disk) |
| Rustfmt | `cargo fmt --all --check` |
| Clippy | `cargo clippy --all-targets --all-features -- -D warnings` |
| Test | `cargo test --workspace --all-features` |
| Rustdoc | `cargo doc --workspace --no-deps` (warnings denied) |
| Cargo Deny | `cargo deny check --all-features` (licenses, bans, sources) |
| SPDX header check | every `.rs` file under `crates/` and `examples/` must carry the SPDX header on line 1 or 2 |

See [.github/workflows/ci.yml](.github/workflows/ci.yml) and
[.github/workflows/license.yml](.github/workflows/license.yml).

### Source file headers

Every `.rs` file starts with (see [docs/source-file-header.txt](docs/source-file-header.txt)):

```rust
// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0
```

### Dependency policy

Prefer an MIT-selectable dependency chain (spec §4). [deny.toml](deny.toml)
allows `MIT`, `Apache-2.0`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Zlib`,
`Unicode-DFS-2016`, `Unicode-3.0`, and `CC0-1.0`; copyleft licenses are banned,
unknown registries and unknown git sources are denied, and wildcard version
requirements are denied.

Current direct dependencies (all `MIT OR Apache-2.0`):

| Dependency | Used by |
|---|---|
| `serde`, `serde_json` | model and format (de)serialization across the workspace |
| `thiserror` | library error types |
| `uuid` (v5/v7) | id generation in `tpt-c-core`, `tpt-c-ids` |
| `zip` + `flate2` (`rust_backend`, no `zopfli`) | `.xlsx` writing in `tpt-c-xlsx` |
| `quick-xml` | dev-dependency of `tpt-c-xlsx` (asserts the generated OOXML) |
| `wasm-bindgen`, `serde-wasm-bindgen`, `web-sys` | optional, behind `tpt-c-wasm`'s `js` / `web` features |

`cargo deny check licenses` passes with no Apache-only crates left: the earlier
transitive Apache-only crates (`zopfli` via `rust_xlsxwriter`, `ryu` via `csv`)
were removed by replacing those dependencies with lightweight in-house
implementations in `tpt-c-csv` and `tpt-c-xlsx`.

### WebAssembly builds

`tpt-c-wasm` re-exports the pure engines and gates the `wasm-bindgen` glue
behind the `js` / `web` features (`js` is enabled by default):

```bash
cargo build -p tpt-c-wasm --target wasm32-unknown-unknown                        # with js glue
cargo build -p tpt-c-wasm --target wasm32-unknown-unknown --no-default-features  # pure Rust only
```

### API documentation

```bash
cargo doc --workspace --no-deps --open
```

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