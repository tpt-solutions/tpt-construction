# tpt-c-cost

Construction cost model — the pricing vocabulary for a takeoff. Defines resource
rates (labor / material / equipment / subcontractor), cost codes and a
`CostDatabase` that maps codes to rates, priced cost items and assemblies,
estimate line items, and the markup math (overhead, profit, escalation, tax)
that turns a subtotal into a bid total. Currency-aware `Money` arithmetic
rejects mismatched currencies instead of silently combining them.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `Money` — ISO 4217 amounts with `checked_add`/`checked_sub` (currency-mismatch
  errors), `scaled`, `rounded`, and `+`/`-`/`*` operators
- `ResourceRate`, `ResourceKind`, `RateUnit` — unit rates keyed for lookup by cost code
- `CostDatabase` — code-to-rate map with `insert`, `get`, `rates`, `add_rate`
- `CostCode` (directly or `from_classification`), `CostItem`, `CostAssembly` roll-ups
- `Estimate` — ordered `LineItem`s with computed extensions, plus `subtotal`,
  `totals`, and `total`
- `Markup` — waterfall applied in order overhead → profit → escalation → tax,
  producing `MarkupTotals`
- `Budget` — allowance + contingency with `variance()` against an estimate total
- `round_money` cent-rounding helper and a typed `CostError`

## Usage

```toml
[dependencies]
tpt-c-cost = "0.1"
```

```rust
use tpt_c_cost::{CostCode, Estimate, LineItem, Markup, Money};
use tpt_c_model::Quantity;
use tpt_c_units::Volume;

let mut estimate = Estimate::new("Demo", "USD")
    .with_markup(Markup::none().with_overhead(10.0).with_profit(10.0));
estimate.add_line(LineItem::new(
    1,
    CostCode::new("03 30 00"),
    Quantity::Volume(Volume::from_cubic_yards(10.0)),
    Money::new(120.0, "USD"),
));

// Line extension = 10 CY (7.6455 m³) × 120 USD; markup applied in order.
let totals = estimate.totals().unwrap();
assert!(totals.total.amount() > totals.subtotal.amount());
```

## Crate relationships

- **Depends on:** `serde`, `thiserror`, `tpt-c-core`, `tpt-c-classification`,
  `tpt-c-ids`, `tpt-c-model`, `tpt-c-units`
- **Used by:** [`tpt-c-change`](../tpt-c-change), [`tpt-c-contracts`](../tpt-c-contracts),
  [`tpt-c-csv`](../tpt-c-csv), [`tpt-c-estimating`](../tpt-c-estimating),
  [`tpt-c-payapps`](../tpt-c-payapps), [`tpt-c-xlsx`](../tpt-c-xlsx),
  `examples/estimate-export`, `examples/field-execution-e2e`, and `examples/tpt`

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) /
[LICENSE-APACHE](../../LICENSE-APACHE)).
