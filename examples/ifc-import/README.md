# ifc-import

Phase 2 integration example: parses an IFC (STEP) file with `tpt-c-ifc` and
converts it into the neutral `tpt-c-model::Project`, then prints a one-line
summary per element. When invoked without arguments it imports the bundled
`test-data/ifc/sample.ifc` fixture; otherwise the path given as the first
argument is read instead.

Part of the [tpt-construction](../../README.md) workspace.

## Run it

```bash
cargo run -p ifc-import
# or with an explicit IFC file
cargo run -p ifc-import -- path/to/model.ifc
```

Output for the bundled sample fixture:

```text
Project: Riverside Office Building
Elements: 4
  - [Wall] East Wall (storey: Level 1)
  - [Slab] Floor Slab L1 (storey: Level 1)
  - [Column] Grid C Column (storey: Level 2)
  - [Beam] Primary Beam (storey: Level 2)
```

## What it exercises

- `tpt_c_ifc::parse` — tokenizes a STEP/IFC payload into entities.
- `tpt_c_ifc::to_model` — maps IFC entities (storeys, walls, slabs, columns,
  beams) into the neutral model, including property sets (`Pset_WallCommon`)
  and element quantity sets (`BaseQuantities`).
- `tpt_c_model::Project` — the neutral, format-agnostic project model that
  downstream engines (takeoff, estimating, scheduling) consume.
- Spatial hierarchy: elements are attached to their building storey via the
  `IFCRELCONTAINEDINSPATIALSTRUCTURE` relationships in the file.
- Fails with a nonzero exit code and a message on `IFC import failed` when the
  source cannot be parsed.

## License

Dual-licensed `MIT OR Apache-2.0`.
