# tpt-construction Cookbook

Minimal, heavily-commented, copy-paste recipes for common construction
workflows. Each recipe is a complete, self-contained `main()` you can drop
into a fresh `cargo new` project — unlike the [examples/](../examples/)
directory, which contains larger integration-style programs with their own
workspace members.

All recipes use only crates from this workspace plus their own transitive
dependencies. They are verified against the current 0.1 APIs; when a recipe
and the docs disagree, the crate's rustdoc (`cargo doc --workspace --open`)
wins.

| # | Recipe | Crates |
|---|--------|--------|
| 01 | [Build a neutral model](01-build-a-neutral-model.md) | `tpt-c-model`, `tpt-c-ids`, `tpt-c-units` |
| 02 | [Quantity takeoff](02-quantity-takeoff.md) | `tpt-c-quantities` |
| 03 | [Price an estimate from a CSV cost database](03-price-an-estimate-from-a-csv-cost-db.md) | `tpt-c-estimating`, `tpt-c-csv` |
| 04 | [Export an estimate to Excel](04-export-estimate-xlsx.md) | `tpt-c-xlsx` |
| 05 | [Solve a CPM schedule](05-cpm-schedule.md) | `tpt-c-schedule` |
| 06 | [Schedule risk with Monte Carlo](06-schedule-risk-monte-carlo.md) | `tpt-c-risk` |
| 07 | [RFI approval workflow](07-rfi-approval-workflow.md) | `tpt-c-workflow` |
| 08 | [Change orders and revision diffs](08-change-orders-and-revision-diffs.md) | `tpt-c-change` |
| 09 | [Offline sync between devices](09-offline-sync-between-devices.md) | `tpt-c-sync` |
| 10 | [Digital twin telemetry](10-digital-twin-telemetry.md) | `tpt-c-twin`, `tpt-c-equipment` |
| 11 | [Serve the engines over HTTP](11-serve-over-http.md) | `tpt` CLI (`serve` feature) |

New to the workspace? Start with [docs/getting-started.md](../docs/getting-started.md).
