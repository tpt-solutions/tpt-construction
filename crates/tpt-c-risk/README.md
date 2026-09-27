# tpt-c-risk

Construction schedule and cost risk analysis via Monte Carlo simulation.
Activities carry optional duration and cost uncertainty (`DurationDistribution`
/ `CostDistribution`: Uniform, Triangular, Normal, PERT) plus a
`WeatherExposure` for expected lost working days; `RiskModel::simulate`
repeatedly samples those distributions, resolves the network with the
`tpt-c-schedule` CPM engine, and summarises the resulting project-duration and
total-cost distributions. From there it derives the probability of meeting a
schedule or cost target and the contingency required at a given confidence
level. Reach for it when a deterministic CPM plan needs P-values and
contingency, not just a single answer.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `RiskModel::simulate(iterations, seed)`: seeded, reproducible Monte Carlo over the CPM engine
- `RiskActivity`: a base schedule `Activity` with optional duration/cost distributions and weather exposure
- `DurationDistribution` and `CostDistribution`: `Uniform`, `Triangular`, `Normal`, and `Pert` sampling
- `WeatherExposure`: Poisson-style expected lost days converted into lost working hours
- `SimulationResult`: raw duration/cost samples plus `Statistics` (mean, std dev, P50/P80/P90)
- `probability_schedule_met`, `probability_cost_met`, and `cost_contingency(baseline, confidence)`
- Self-contained vendored PCG64-style `Rng` (uniform, normal, triangular, PERT, Poisson) — no external probability substrate

## Usage

```toml
[dependencies]
tpt-c-risk = "0.1"
```

```rust
use tpt_c_core::ActivityId;
use tpt_c_risk::{CostDistribution, DurationDistribution, RiskActivity, RiskModel};
use tpt_c_schedule::Activity;
use tpt_c_units::Duration;

let id = ActivityId::from_uuid(uuid::Uuid::now_v7());
let mut model = RiskModel::new();
model.add_activity(
    RiskActivity::new(Activity::new(id, "Excavation", Duration::from_hours(40.0)))
        .with_duration_dist(DurationDistribution::Triangular {
            min_hours: 32.0,
            mode_hours: 40.0,
            max_hours: 50.0,
        })
        .with_cost_dist(CostDistribution::Triangular {
            min: 38_250.0,
            mode: 45_000.0,
            max: 54_000.0,
        }),
);

let sim = model.simulate(5_000, 42)?;
println!("P80 duration: {} h", sim.schedule.p80);
println!("P(cost <= 50k): {:.1}%", sim.probability_cost_met(50_000.0) * 100.0);
println!("contingency @90%: {}", sim.cost_contingency(sim.cost.mean, 0.9));
```

## Crate relationships

- **Depends on:** `serde`, `thiserror`, `tpt-c-core`, `tpt-c-schedule`, `tpt-c-units` (dev: `tpt-c-ids`, `uuid`)
- **Used by:** the `cpm-schedule` example (schedule + risk + earned-value walkthrough)

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) / [LICENSE-APACHE](../../LICENSE-APACHE)).
