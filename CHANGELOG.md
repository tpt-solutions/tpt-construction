# Changelog

All notable changes to the tpt-construction workspace are documented in this
file. Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
versioning follows [Semantic Versioning](https://semver.org/). Per-crate
changes are documented in each crate's own `CHANGELOG.md`.

## [Unreleased]

### Added

- `tpt-c-change`: new crate for change control — `ChangeOrder` with a
  guarded review state machine (`Draft → Submitted → UnderReview →
  Approved/Rejected → Implemented/Withdrawn`), `ChangeRegister` with
  approved cost/time roll-ups, and `RevisionDiff` producing quantity deltas
  (`QuantityDelta`, spec §12's 1000 CY → 1050 CY example) and cost deltas
  per revision.
- `tpt serve` subcommand (behind the `serve` feature): thin dependency-free
  HTTP/1.1 server exposing takeoff, estimate, and CPM scheduling over JSON,
  with bearer-token auth/scopes (`tpt-c-api` `AuthToken`/`Permission`) and
  `AuditLogEntry` logging.
- Per-crate `README.md` and `CHANGELOG.md` for every crate, plus
  `readme`/`categories`/`keywords` manifest metadata workspace-wide.
- `cookbook/` — minimal, heavily-commented, copy-paste recipes covering the
  model → takeoff → estimate → export flow, CPM, Monte Carlo risk, change
  orders, sync, twins, and the HTTP server.
- `docs/getting-started.md` — smallest end-to-end flow (CLI and library).
- `CONTRIBUTING.md` — dev commands, crate conventions, header requirement.
- CI: workspace-membership check (`scripts/check-workspace-members.sh`)
  failing when a `crates/*` directory is not in `[workspace] members`, and
  a `cargo doc --workspace --no-deps` rustdoc job (warnings denied).

### Changed

- `tpt-c-las`, `tpt-c-dxf`, `tpt-c-db`, `tpt-c-equipment`, `tpt-c-events`,
  and `tpt-c-sync` are now regular workspace members, so
  `cargo test --workspace` and all CI gates cover them.

### Fixed

- `tpt-c-dxf`: the parser never compiled (type errors in section/entity
  lookahead); parsing is rebuilt around a `(group code, value)` pair list,
  with LWPOLYLINE/TEXT/POINT handling and error-path tests.
- `tpt-c-las`: format-0 point parsing read the classification from the wrong
  byte (scan-direction/edge flags live inside the returns byte), and the
  test fixture mis-declared its header size/point offset so the round-trip
  test never ran green.
- `tpt-c-ifc`: removed a leftover `eprintln!` debug trace from `parse`.
- `tpt-c-schedule` and `tpt-c-xlsx`: removed the last production-code
  `unwrap()`s (CPM topological sort and sheet-row handling are now
  panic-free); library code is unwrap-free apart from tests.

## [0.1.0] - 2026-08-17

### Added

- Initial workspace: foundation (`tpt-c-core`, `tpt-c-ids`, `tpt-c-units`,
  `tpt-c-classification`, `tpt-c-model`), geometry & formats
  (`tpt-c-geometry`, `tpt-c-geo`, `tpt-c-ifc`, `tpt-c-bcf`, `tpt-c-gltf`,
  `tpt-c-las`, `tpt-c-dxf`), estimating (`tpt-c-quantities`, `tpt-c-cost`,
  `tpt-c-estimating`, `tpt-c-csv`, `tpt-c-xlsx`), scheduling & controls
  (`tpt-c-schedule`, `tpt-c-risk`, `tpt-c-earned-value`), civil
  (`tpt-c-earthwork`, `tpt-c-alignment`), field execution (`tpt-c-field`,
  `tpt-c-workflow`, `tpt-c-documents`, `tpt-c-contracts`, `tpt-c-payapps`,
  `tpt-c-safety`), equipment & sync (`tpt-c-equipment`, `tpt-c-events`,
  `tpt-c-sync`, `tpt-c-db`), FM & twins (`tpt-c-fm`, `tpt-c-assets`,
  `tpt-c-maintenance`, `tpt-c-twin`, `tpt-c-space`), and web platform
  (`tpt-c-wasm`, `tpt-c-api`).
- `tpt` CLI with `estimate` subcommand; runnable examples for IFC import,
  quantity takeoff, estimate export, CPM scheduling, earthwork cut/fill,
  field execution, offline sync, and a WASM browser demo.
- CI (fmt, clippy, test, cargo-deny, SPDX headers) and dual MIT OR
  Apache-2.0 licensing with `deny.toml` enforcement.

[Unreleased]: https://github.com/tpt-solutions/tpt-construction/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/tpt-solutions/tpt-construction/releases/tag/v0.1.0
