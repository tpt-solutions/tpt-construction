# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `RiskModel` with `new`, `add_activity`, `activities_mut`, `add_dependency`, `add_constraint`, and `simulate(iterations, seed)`.
- `RiskActivity` builder: `new`, `with_duration_dist`, `with_cost_dist`, `with_weather`.
- `DurationDistribution` and `CostDistribution` enums (`Uniform`, `Triangular`, `Normal`, `Pert`) with `sample`.
- `WeatherExposure` (expected lost days and hours per lost day).
- `SimulationResult` with raw samples, `schedule`/`cost` `Statistics` (mean, std dev, P50/P80/P90), `probability_schedule_met`, `probability_cost_met`, and `cost_contingency`.
- Vendored `Rng` (PCG64-style) with `uniform`, `standard_normal`, `normal`, `triangular`, `pert`, and `poisson` samplers.
- `RiskError` (`EmptyModel`, `NoIterations`, `Schedule`) for simulation guards.
