# tpt-c-earned-value

Earned Value Management (EVM) for construction projects. Earned value
integrates scope, schedule, and cost on a single basis: this crate captures a
periodic status snapshot — Budget at Completion, Planned Value (BCWS/PV),
Earned Value (BCWP/EV), and Actual Cost (ACWP/AC) — and derives the standard
schedule and cost variances, performance indices (SPI/CPI), estimates at
completion (EAC under three forecasting methods), ETC, VAC, and to-complete
performance indices (TCPI). Monetary math uses the crate-local `Amount` type
(magnitude + ISO 4217 currency), so the crate stays free of external monetary
substrates. Reach for it when you need defensible progress-and-cost reporting.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `EarnedValueStatus::new` validates that all four inputs share a single currency (`EarnedValueError::CurrencyMismatch` otherwise)
- Variances: `schedule_variance` (SV = EV − PV) and `cost_variance` (CV = EV − AC)
- Indices: `spi`, `cpi`, and `percent_complete` (EV / BAC); zero denominators yield `0.0` rather than NaN
- Forecasts: `eac` with `EacMethod::Typical` (`BAC/CPI`), `Atypical` (`AC + (BAC − EV)`), and `Combined` (`AC + (BAC − EV)/(CPI·SPI)`), plus `etc` and `vac`
- To-complete performance indices: `tcpib` (finish at BAC) and `tcpie` (finish at a given EAC)
- `Amount` with `checked_add`/`checked_sub` and `Add`/`Sub` operators that reject mixed currencies
- Pure computation crate: no IO, fully serializable — WASM-friendly by construction

## Usage

```toml
[dependencies]
tpt-c-earned-value = "0.1"
```

```rust
use tpt_c_core::ProjectId;
use tpt_c_earned_value::{Amount, EacMethod, EarnedValueStatus};

let ev = EarnedValueStatus::new(
    ProjectId::nil(),
    Amount::new(1000.0, "USD"), // BAC
    Amount::new(600.0, "USD"),  // BCWS / PV
    Amount::new(500.0, "USD"),  // BCWP / EV
    Amount::new(550.0, "USD"),  // ACWP / AC
)
.unwrap();

assert_eq!(ev.cost_variance().value(), -50.0);
assert_eq!(ev.schedule_variance().value(), -100.0);
assert!((ev.spi() - 500.0 / 600.0).abs() < 1e-9);
assert!((ev.cpi() - 500.0 / 550.0).abs() < 1e-9);

let eac = ev.eac(EacMethod::Typical);
let vac = ev.vac(EacMethod::Atypical);
let etc = ev.etc(EacMethod::Atypical);
println!("TCPI(BAC) {:.2}", ev.tcpib());
```

## Crate relationships

- **Depends on:** `serde`, `thiserror`, `tpt-c-core`
- **Used by:** the `cpm-schedule` example (reporting an earned-value status snapshot alongside CPM and risk results)

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) / [LICENSE-APACHE](../../LICENSE-APACHE)).
