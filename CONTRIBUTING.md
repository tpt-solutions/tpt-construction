# Contributing to tpt-construction

Thanks for helping build the construction layer of the TPT ecosystem. This
document covers the commands, conventions, and checks that keep the
workspace green.

## Development commands

Stable Rust, edition 2021, MSRV **1.82** (see `clippy.toml`). CI sets
`RUSTFLAGS="-D warnings"`; do the same locally:

```bash
cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo deny check            # licenses, bans, sources
cargo doc --workspace --no-deps   # CI fails on rustdoc warnings
bash scripts/check-workspace-members.sh   # every crates/* dir must be a member
```

Order when fixing CI: **fmt → clippy → test → deny**.

### Windows / newline gotcha

There is no `.gitattributes`, and `rustfmt.toml` sets
`newline_style = "Unix"`. If your checkout was created with
`core.autocrlf=true`, `cargo fmt --all --check` fails on every file. Fix with
`git config core.autocrlf input` and re-checkout, or run `cargo fmt --all`
once to normalize the tree.

## Workspace layout

- `crates/` — `tpt-c-*` libraries (package names kebab-case, import names
  snake_case: `tpt-c-cost` → `tpt_c_cost`).
- `examples/` — runnable end-to-end programs and the `tpt` CLI; every
  example is a workspace member with its own README.
- `cookbook/` — minimal, heavily-commented copy-paste recipes (docs, not
  workspace members).
- `test-data/` — fixtures: `ifc/`, `bcf/`, and `golden/` (model JSON, CSV
  rates, XLSX output).

## Architecture conventions

1. **Neutral model first.** File formats parse into `tpt-c-model`; domain
   engines consume `tpt-c-model` and never depend on file-format crates.
2. **Pure core crates.** Crates are deterministic, serializable, and
   platform-independent. IO stays at the edges (CLI, examples). Everything
   must compile for `wasm32-unknown-unknown`.
3. **Event-first auditability.** Important state changes should be
   representable as domain events (`tpt-c-events`).
4. **License hygiene.** Prefer `MIT OR Apache-2.0` dependencies. Apache-only
   dependencies require review and, when unavoidable, isolation behind an
   optional feature. Copyleft is banned outright (`deny.toml` enforces
   this).
5. **No unwrap/panic in library code.** Errors use `thiserror`; fallible
   operations return `Result`. `.unwrap()` is acceptable only inside
   `#[cfg(test)]` modules.

## Source file headers

Every `.rs` file under `crates/` and `examples/` starts with (see
[docs/source-file-header.txt](docs/source-file-header.txt)):

```rust
// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0
```

The license workflow fails if the header is missing from line 1–2.

## Adding a crate

1. `cargo new crates/tpt-c-<name> --lib` and wire
   `version.workspace = true`, `edition.workspace = true`,
   `license.workspace = true`, `repository.workspace = true`,
   `authors.workspace = true`, plus `[lints] workspace = true`.
2. Add the crate to the root `Cargo.toml` `[workspace] members` list **and**
   to `[workspace.dependencies]` — the membership check script and CI fail
   otherwise.
3. Add the SPDX header, rustdoc crate docs (`//!`), and unit tests in
   `#[cfg(test)]` modules; integration tests go in `tests/*.rs`.
4. Every crate carries a `README.md`, `CHANGELOG.md` (Keep a Changelog
   format), and `readme`/`categories`/`keywords` manifest fields — copy the
   shape from an existing crate.

## Testing patterns

- Unit tests live inside `src/*.rs` under `#[cfg(test)] mod tests`.
- Integration tests live in `<crate>/tests/*.rs` (see
  `crates/tpt-c-estimating/tests/e2e.rs`).
- Golden fixtures live in `test-data/golden/`.
- Single crate: `cargo test -p tpt-c-estimating`; single test:
  `cargo test -p tpt-c-estimating <filter>`.

## Documentation expectations

- Public items need rustdoc comments; CI denies rustdoc warnings.
- User-facing behavior changes update the affected crate's `CHANGELOG.md`
  under `[Unreleased]`, and the root `CHANGELOG.md` for workspace-level
  changes.
- New user-facing workflows deserve a cookbook recipe or example.

## License

Contributions are dual-licensed **MIT OR Apache-2.0**, and by submitting a
patch you agree to license your work under those terms (see the contribution
license text in `spec.txt` §27).
