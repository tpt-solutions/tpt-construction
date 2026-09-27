# cpm-schedule

Phase 4 integration example: builds a small seven-activity building schedule,
solves it with the Critical Path Method, projects the dates onto a real 40-hour
working calendar, runs a 5000-trial Monte Carlo risk analysis (triangular
duration and cost distributions plus weather exposure on excavation), and
prints an earned-value status snapshot (SPI/CPI, EAC, VAC, TCPI).

Part of the [tpt-construction](../../README.md) workspace.

## Run it

```bash
cargo run -p cpm-schedule
```

The example takes no arguments; it prints sections like (numbers come from the
solver and simulation):

```text
=== CPM Schedule: Riverside Outbuilding ===
Project duration: <hours> working hours (~<days> working days)
Forecast finish date: <date>

  Mobilization        <start> -> <finish>   float <f>h
  ...
Critical path: Mobilization -> Excavation -> ... -> Closeout

=== Monte Carlo Risk (5000 trials) ===
Schedule (working days): mean <m> | P50 <p> | P80 <p> | P90 <p>
Probability of meeting the <d>-day plan: <p>%
Total cost (USD): mean <m> | P80 <c> | P90 <c>
Cost contingency @90%: <c> USD

=== Earned Value Status (mid-project) ===
SPI <x> | CPI <y>
EAC (typical) <e> | VAC <v> | TCPI(bac) <t>
```

## What it exercises

- `tpt_c_schedule` — `Schedule`, `Activity`, `Dependency` (finish-to-start),
  `Calendar::standard_40h`, CPM forward/backward pass, total float, and the
  critical path.
- `tpt_c_risk` — `RiskModel` with triangular duration/cost distributions and
  `WeatherExposure`, plus a seeded Monte Carlo simulation with schedule and
  cost percentiles and contingency at the 90% confidence level.
- `tpt_c_earned_value` — `EarnedValueStatus` computing SPI, CPI, EAC (typical
  method), VAC, and TCPI from BAC/BCWS/BCWP/ACWP amounts.
- `tpt_c_core` / `tpt_c_ids` — deterministic activity IDs via `IdFactory` so
  the scenario is reproducible; `tpt_c_units::Duration` for working hours.

## License

Dual-licensed `MIT OR Apache-2.0`.
