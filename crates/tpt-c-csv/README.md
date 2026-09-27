# tpt-c-csv

Tabular CSV import/export for construction data. Provides round-trippable record
types for cost databases so an estimator can edit rates in a spreadsheet and
re-import them: `read_cost_database` and `write_cost_database` convert between
any `std::io::Read`/`Write` sink and a [`tpt-c-cost`](../tpt-c-cost)
`CostDatabase`, with quoting, header validation, and per-line error reporting
handled for you.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `read_cost_database(R: Read) -> CostDatabase` — parse rates from CSV into the cost crate
- `write_cost_database(W: Write, &CostDatabase)` — export rates back to CSV
- `CostRateRow` schema: `code,title,kind,unit,rate,currency` (with `CostRateRow::header()`)
- Kind and unit strings mapped to `ResourceKind` / `RateUnit`, with errors for unknown values
- RFC-4180-style quoting both ways: embedded commas, double quotes, and newlines
- Strict header check and per-row validation; `CsvError::Malformed` carries the 1-based line number
- Blank lines are skipped; tolerant of CRLF and LF line endings

## Usage

```toml
[dependencies]
tpt-c-csv = "0.1"
tpt-c-cost = "0.1"
```

```rust
use tpt_c_cost::{CostDatabase, Money, RateUnit, ResourceKind};
use tpt_c_csv::{read_cost_database, write_cost_database};

let mut db = CostDatabase::new();
db.add_rate(
    "03 30 00",
    "03 30 00",
    ResourceKind::Material,
    RateUnit::Volume,
    Money::new(120.0, "USD"),
);

// Export to CSV (fields containing commas/quotes are quoted and escaped).
let mut buf: Vec<u8> = Vec::new();
write_cost_database(&mut buf, &db).unwrap();

// Re-import; round-trips to an equal database.
let back = read_cost_database(buf.as_slice()).unwrap();
assert_eq!(back.get("03 30 00").unwrap().rate.amount(), 120.0);
```

## Crate relationships

- **Depends on:** `serde`, `thiserror`, `tpt-c-core`, `tpt-c-units`,
  `tpt-c-model`, `tpt-c-cost`
- **Used by:** [`tpt-c-estimating`](../tpt-c-estimating) (integration tests),
  `examples/estimate-export`, and the `examples/tpt` CLI/web app
  (`--cost-db` rate loading)

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) /
[LICENSE-APACHE](../../LICENSE-APACHE)).
