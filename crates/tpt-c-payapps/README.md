# tpt-c-payapps

Progress claims and payment applications. A `ScheduleOfValues` breaks a contract
into valued line items; a `PaymentApplication` draws down against it — work
completed to date plus stored materials, less retainage and previously certified
amounts, yields the net amount payable this period. Applications move through a
`Draft -> Submitted -> UnderReview -> Approved -> Certified` lifecycle, with
monetary math delegated to `tpt-c-cost`'s `Money`.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `ScheduleOfValues` with `SovItem` line items (cost-code id, description,
  `Money` contract value), `total()`, `len()`, and `is_empty()`.
- `PaymentApplication` modeling a period draw-down: `work_completed`,
  `stored_materials`, `retainage` fraction, and `previously_certified`.
- Derived figures: `gross_val()` (work + stored materials), `retainage_amount()`,
  and `net_claim()` (`gross - retainage - previously_certified`).
- Retainage validated to the `[0, 1]` fraction range via `set_retainage`.
- `PaymentApplicationStatus` lifecycle with `submit`, `begin_review`,
  `approve(by)`, `reject`, and `certify` (certification only from `Approved`).
- `PayAppError` mapping `tpt-c-cost` currency mismatches to
  `PayAppError::CurrencyMismatch`.
- Serializable with serde; a pure core crate with no IO.

## Usage

```toml
[dependencies]
tpt-c-payapps = "0.1"
```

```rust
use tpt_c_cost::Money;
use tpt_c_ids::PaymentApplicationId;
use tpt_c_payapps::{PaymentApplication, PaymentApplicationStatus, ScheduleOfValues};
use uuid::Uuid;

let mut sov = ScheduleOfValues::new("SOV-1");
sov.add_item("01", "Foundations", Money::new(200_000.0, "USD"));
sov.add_item("02", "Superstructure", Money::new(800_000.0, "USD"));
assert_eq!(sov.total().amount(), 1_000_000.0);

let mut app = PaymentApplication::new(
    PaymentApplicationId::from_uuid(Uuid::now_v7()),
    "2026-03",
    sov.total(),
)
.with_sov("SOV-1");
app.set_work_completed(Money::new(150_000.0, "USD"));
app.set_retainage(0.05).unwrap();
// gross 150k, retainage 7.5k, previously certified 0 => net 142.5k
assert_eq!(app.net_claim().amount(), 142_500.0);

app.approve("owner".to_string());
app.certify();
assert_eq!(app.status, PaymentApplicationStatus::Certified);
```

## Crate relationships

- **Depends on:** `serde`, `thiserror`, `uuid`, `tpt-c-core`, `tpt-c-ids`
  (`PaymentApplicationId`), and `tpt-c-cost` (`Money`, `CostError`).
- **Used by:** Not yet consumed by other workspace crates;
  [examples/field-execution-e2e](../../examples/field-execution-e2e) certifies a
  pay application against a schedule of values in its scenario.

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) / [LICENSE-APACHE](../../LICENSE-APACHE)).
