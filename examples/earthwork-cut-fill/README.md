# earthwork-cut-fill

Phase 5 integration example: builds a corridor alignment (tangent — circular
curve — tangent plan plus a grade/crest-curve profile), samples it every 25 m
against a synthetic rolling existing-ground line, accumulates the cut and fill
volumes into a mass haul diagram, and reports the earthwork balance including
swell/shrink conversions and unit conversions.

Part of the [tpt-construction](../../README.md) workspace.

## Run it

```bash
cargo run -p earthwork-cut-fill
```

The example takes no arguments; it prints a report shaped like (numbers come
from the alignment geometry):

```text
Corridor plan length: 914.2 m
  horizontal element 0: 300.0 m
  horizontal element 1: 314.2 m
  horizontal element 2: 300.0 m
Station <station>: proposed elevation <e> m (existing <e> m)
Superelevation at 60 m into runoff: <s>%

Mass haul (37 stations):
  total cut:   <c> bank m3
  total fill:  <f> bank m3
  net (cut-fill):  <n> bank m3
  cut as loose volume: <l> m3
  cut in cubic yards:  <cy> CY

Balance (haul factor 1.15):
  excess cut: <x> bank m3
  => surplus cut must be hauled to spoil or stockpiled
```

The final `=>` line is chosen from the computed balance: surplus cut to haul,
a fill deficit needing borrow, or a balanced site.

## What it exercises

- `tpt_c_alignment` — `HorizontalElement` (tangents and a circular curve),
  `VerticalElement` (grades and a vertical curve), `Station` formatting, and
  `Superelevation` runoff rates.
- `tpt_c_earthwork` — `MassHaulDiagram` with `SwellShrink` factors (25% swell,
  10% shrink), total cut/fill and net cut, loose-volume conversion, and
  `CutFillBalance::compute` with a 1.15 haul factor.
- `tpt_c_units::Volume` — cubic-metre to cubic-yard conversion.

## License

Dual-licensed `MIT OR Apache-2.0`.
