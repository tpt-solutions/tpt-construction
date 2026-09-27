# tpt-c-xlsx

Excel (`.xlsx`) export for estimator workflows. A small, dependency-light OOXML
writer built directly over `zip` and `flate2` — no large spreadsheet library.
The generic `write_table` emits a header row plus string/number cells, and
`write_estimate` produces a bid-ready estimate workbook with an "Estimate" sheet
(one row per line item) and a "Summary" sheet carrying the markup breakdown
(subtotal, overhead, profit, escalation, tax, total).

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `write_table(path, sheet_name, headers, rows)` — generic single-sheet writer;
  numeric strings are emitted as real number cells so Excel treats them as values
- `write_estimate(path, &Estimate)` — two-sheet bid workbook: line items plus a
  markup summary (overhead, profit, escalation, tax, total)
- Hand-rolled minimal OOXML package (`[Content_Types].xml`, rels, workbook,
  styles, worksheets) with XML escaping
- Deflate-compressed ZIP output via `zip` + `flate2`
- `XlsxError` covering IO, ZIP, and malformed-XML failures
- Round-trip tested: output opens as a valid ZIP archive with well-formed XML parts

## Usage

```toml
[dependencies]
tpt-c-xlsx = "0.1"
```

```rust
use tpt_c_cost::{CostCode, Estimate, LineItem, Markup, Money};
use tpt_c_model::Quantity;
use tpt_c_units::Volume;
use tpt_c_xlsx::{write_estimate, write_table};

// Generic table export.
let path = std::env::temp_dir().join("rates.xlsx");
let headers = vec!["Code".to_string(), "Rate".to_string()];
let rows = vec![vec!["03 30 00".to_string(), "120.5".to_string()]];
write_table(&path, "Rates", &headers, &rows).unwrap();

// Bid-ready estimate workbook ("Estimate" + "Summary" sheets).
let mut estimate = Estimate::new("Demo", "USD").with_markup(Markup::none());
estimate.add_line(LineItem::new(
    1,
    CostCode::new("03 30 00"),
    Quantity::Volume(Volume::from_cubic_yards(10.0)),
    Money::new(120.0, "USD"),
));
write_estimate("estimate.xlsx", &estimate).unwrap();
```

## Crate relationships

- **Depends on:** `zip`, `flate2`, `serde`, `thiserror`, `tpt-c-core`,
  `tpt-c-cost`, `tpt-c-estimating`, `tpt-c-model`, `tpt-c-units`
- **Used by:** [`tpt-c-estimating`](../tpt-c-estimating) (integration tests),
  `examples/estimate-export`, and the `examples/tpt` CLI (`--output estimate.xlsx`)

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) /
[LICENSE-APACHE](../../LICENSE-APACHE)).
