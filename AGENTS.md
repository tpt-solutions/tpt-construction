# AGENTS.md — tpt-construction

## Toolchain
- Stable Rust, edition 2021, MSRV 1.82 (`clippy.toml`).
- CI sets `RUSTFLAGS="-D warnings"`; use the same locally.

## Commands (exact)
```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo deny check                                 # or: cargo deny check --all-features
cargo doc --workspace --no-deps --open
cargo run -p tpt -- estimate test-data/golden/sample-model.json --cost-db test-data/golden/rates.csv --output estimate.xlsx
cargo build -p tpt-c-wasm --target wasm32-unknown-unknown              # with js glue
cargo build -p tpt-c-wasm --target wasm32-unknown-unknown --no-default-features  # pure Rust
```

Order when fixing CI: `fmt -> clippy -> test -> deny`.

## Workspace boundaries
- Root is a virtual manifest (`resolver = "2"`).
- `crates/` holds `tpt-c-*` libraries (40 crates, all with README/CHANGELOG/categories/keywords); `examples/` holds runnable binaries and the `tpt` CLI; `cookbook/` holds markdown recipes (not workspace members).
- **Every** `crates/*` directory is listed in `Cargo.toml` `members`; `scripts/check-workspace-members.sh` (run in CI) fails if the lists drift.
- `tpt-c-change` exists on disk (change orders, revision diffs, registers).

## Quality gates (all must pass)
- **Rustfmt**: `cargo fmt --all --check`
- **Clippy**: `cargo clippy --all-targets --all-features -- -D warnings`
- **Tests**: `cargo test --workspace --all-features`
- **Cargo deny**: license/bans/source/advisory checks (`deny.toml`). Only MIT/Apache-2.0/BSD/ISC/Zlib/Unicode/CC0 allowed. Copyleft is banned.
- **Rustdoc**: `cargo doc --workspace --no-deps` with `RUSTDOCFLAGS="-D warnings"` (CI job).
- **Workspace membership**: `bash scripts/check-workspace-members.sh`.
- **SPDX header**: every `.rs` file under `crates/` and `examples/` must carry `SPDX-License-Identifier: MIT OR Apache-2.0` on line 1 or 2.

## Windows / newline gotcha
There is no `.gitattributes`. `rustfmt.toml` sets `newline_style = "Unix"`.
If `core.autocrlf=true`, `cargo fmt --all --check` will fail on every file.
Fix: re-checkout with `git config core.autocrlf input`, or run `cargo fmt --all` to normalize first.

## Architecture conventions
- **Neutral model first**: file formats (IFC, BCF, etc.) parse into `tpt-c-model`; domain engines consume `tpt-c-model` and never depend directly on file formats.
- **Pure core crates**: deterministic, serializable, platform-independent; IO stays at edges.
- **Event-first auditability**: important state changes are representable as domain events.
- **WASM as first-class target**: engines compile to `wasm32-unknown-unknown`.

## Crate naming
- Package names: `tpt-c-<name>` (kebab-case).
- Rust import names: `tpt_c_<name>` (snake_case).

## Testing patterns
- Unit tests live in `#[cfg(test)]` modules inside `src/*.rs`.
- Integration tests live in `<crate>/tests/*.rs` (e.g., `crates/tpt-c-estimating/tests/e2e.rs`).
- Golden fixtures live in `test-data/golden/` (model JSON, CSV rates, XLSX output).
- Run a single crate's tests: `cargo test -p tpt-c-estimating`.
- Run a single test by name: `cargo test -p tpt-c-estimating <filter>`.

## License
Contributions are dual-licensed MIT OR Apache-2.0 unless explicitly stated otherwise.
