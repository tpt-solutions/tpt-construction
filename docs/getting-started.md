# Getting started with tpt-construction

This guide walks the smallest end-to-end flow — **build a model → quantity
takeoff → price an estimate → export Excel** — first with the CLI (five
minutes, no code), then from Rust. It ends with pointers for scheduling,
field, and web use cases.

## 1. Prerequisites

- A stable Rust toolchain (1.82 or newer): [rustup](https://rustup.rs)
- That's it — no system libraries; every dependency is pure Rust.

Check out and build:

```bash
git clone https://github.com/tpt-solutions/tpt-construction.git
cd tpt-construction
cargo build --workspace
cargo test --workspace          # optional: verify everything is green
```

## 2. The fastest end-to-end flow: the `tpt` CLI

The repository ships a sample neutral model and a sample rate database:

```bash
cargo run -p tpt -- estimate test-data/golden/sample-model.json \
    --cost-db test-data/golden/rates.csv \
    --output estimate.xlsx
```

```text
Estimate written to estimate.xlsx: 5 line items, subtotal 6141.10 USD, total 7295.63 USD
```

What just happened:

1. `sample-model.json` is a **neutral model** (`tpt-c-model`): two building
   elements with quantity sets and MasterFormat classifications. Neutral
   models are what the format parsers (`tpt-c-ifc`, `tpt-c-bcf`, …)
   produce, so everything downstream works the same regardless of where the
   data came from.
2. `rates.csv` is a **cost database**: one row per cost code with kind,
   unit, rate, and currency (`code,title,kind,unit,rate,currency`).
3. The CLI ran quantity takeoff (`tpt-c-quantities`), priced every quantity
   by its code (`tpt-c-estimating`), applied the default markup schedule,
   and wrote a two-sheet `.xlsx` (`tpt-c-xlsx`).

Open `estimate.xlsx`: sheet **Estimate** lists line items, sheet **Summary**
has the markup breakdown and total.

## 3. The same flow from Rust

The library path is four steps and mirrors the CLI exactly — see the
cookbook for complete, runnable programs:

| Step | Cookbook recipe | Crate |
|---|---|---|
| Build/load a model | [1. Build a neutral model](../cookbook/01-build-a-neutral-model.md) | `tpt-c-model` |
| Takeoff | [2. Quantity takeoff](../cookbook/02-quantity-takeoff.md) | `tpt-c-quantities` |
| Price it | [3. Price an estimate from a CSV cost database](../cookbook/03-price-an-estimate-from-a-csv-cost-db.md) | `tpt-c-estimating` |
| Export | [4. Export an estimate to Excel](../cookbook/04-export-estimate-xlsx.md) | `tpt-c-xlsx` |

The essential shape, in code:

```rust,ignore
let project: tpt_c_model::Project = /* file, IFC parse, or builder */;
let takeoff = tpt_c_quantities::TakeoffEngine::new().run(&project);
let estimate = tpt_c_estimating::EstimateBuilder::new("Estimate", "USD")
    .with_database(cost_db)
    .from_takeoff(&takeoff)?;
tpt_c_xlsx::write_estimate("estimate.xlsx", &estimate)?;
```

## 4. Where to next

- **Scheduling:** solve a CPM network ([recipe 5](../cookbook/05-cpm-schedule.md)),
  run Monte Carlo risk ([recipe 6](../cookbook/06-schedule-risk-monte-carlo.md)),
  or run the whole deterministic + risk + earned-value demo:
  `cargo run -p cpm-schedule`.
- **Civil/earthwork:** cut/fill, mass haul and alignments —
  `cargo run -p earthwork-cut-fill`.
- **Field workflows:** daily logs → RFI → pay app end-to-end —
  `cargo run -p field-execution-e2e`; offline-first sync —
  `cargo run -p field-offline-sync`.
- **Change control:** change orders, revision diffs, registers —
  [recipe 8](../cookbook/08-change-orders-and-revision-diffs.md).
- **HTTP integration:** expose takeoff/estimate/schedule to scripts and
  browsers — [recipe 11](../cookbook/11-serve-over-http.md).
- **Formats:** parse IFC into the neutral model with `tpt-c-ifc`
  (`cargo run -p ifc-import`), or browse every crate in the
  [crate reference](../README.md#crate-reference).

Each crate directory also carries its own `README.md` with a focused
example, and `cargo doc --workspace --no-deps --open` renders the full API
documentation.
