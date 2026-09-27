# quantity-takeoff

Reads a neutral `tpt-c-model` project from JSON, runs the `tpt-c-quantities`
takeoff engine over every element, and prints a per-element quantity summary
(net, gross, and waste percentages, plus manual-override markers) followed by
gross totals for each quantity kind (Volume, Area, Length, Mass, Count).

Part of the [tpt-construction](../../README.md) workspace.

## Run it

```bash
cargo run -p quantity-takeoff -- <model.json>
# for example, against the golden fixture:
cargo run -p quantity-takeoff -- test-data/golden/sample-model.json
```

A model path is required; without one the binary prints
`usage: quantity-takeoff <model.json>` and exits with a failure code.

Output for the golden fixture (numbers depend on the model):

```text
Takeoff for 'Golden Demo': 3 elements

[01a0a99d-...] SLAB-1 (Slab)
  - GrossVolume: net 7.6455  gross ...  waste ...%
  ...

Total Volume: net ...  gross ...
Total Area: net ...  gross ...
Total Length: net ...  gross ...
Total Mass: net ...  gross ...
Total Count: net ...  gross ...
```

## What it exercises

- `tpt_c_model::Project` — deserialized from the neutral model JSON via
  `serde_json`.
- `tpt_c_quantities::TakeoffEngine` — measures each element and produces a
  `TakeoffResult` with per-quantity net values, gross (net + waste), and
  manual-override flags.
- `tpt_c_quantities::QuantityKind` — aggregate totals per kind via
  `total_net` / `total_gross`.
- `tpt_c_core` / `tpt_c_units` — the base value and ratio primitives used to
  format quantities.
- Error handling for missing files, unreadable paths, and invalid model JSON,
  each reported on stderr with a failure exit code.

## License

Dual-licensed `MIT OR Apache-2.0`.
