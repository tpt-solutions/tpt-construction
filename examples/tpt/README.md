# tpt

The TPT Construction command-line interface. The `estimate` subcommand reads a
neutral `tpt-c-model` JSON file and a CSV cost database, prices the model with
the estimating engine, and writes the result to an `.xlsx` workbook. Built
with the `serve` feature it also exposes the takeoff, estimate, and CPM
scheduling engines over a thin, dependency-free HTTP interface.

Part of the [tpt-construction](../../README.md) workspace.

## Run it

```bash
cargo run -p tpt -- estimate <model.json> --cost-db <rates.csv> --output <estimate.xlsx>
# for example, against the golden fixtures:
cargo run -p tpt -- estimate test-data/golden/sample-model.json \
    --cost-db test-data/golden/rates.csv --output estimate.xlsx

# HTTP server (requires the feature flag)
cargo run -p tpt --features serve -- serve [--addr <addr>] [--token <secret>]
```

Short flags `-c`, `-o` are accepted for `--cost-db` and `--output`; running
without arguments, or with an unknown subcommand, prints usage.

Output for `estimate` (numbers depend on the model and rates):

```text
Estimate written to estimate.xlsx: <N> line items, subtotal <X.XX> USD, total <Y.YY> USD
```

`serve` binds to `127.0.0.1:8090` by default; with `--token`, requests must
present `Authorization: Bearer <token>`.

## What it exercises

- `tpt_c_model::Project` — the neutral model JSON consumed by every workflow.
- `tpt_c_csv::read_cost_database` / `tpt_c_cost::CostDatabase` — CSV rate
  ingestion and cost lookup.
- `tpt_c_quantities` / `tpt_c_estimating::EstimateBuilder` — measurement and
  pricing into line items with subtotal/total.
- `tpt_c_xlsx::write_estimate` — Excel export.
- With `--features serve`: `tpt_c_schedule` (CPM), `tpt_c_api`
  (request routing), `tpt_c_units`, and `uuid` behind a hand-rolled
  `std::net::TcpListener` HTTP server (`src/serve.rs`) — JSON only, no
  external web dependencies.

## License

Dual-licensed `MIT OR Apache-2.0`.
