# TPT Construction — Project Todo

Dual-licensed MIT OR Apache-2.0 · TPT Solutions

## Phase 0: Repository Scaffolding
- [ ] Root Cargo.toml (workspace, resolver = "2", workspace.package, workspace.dependencies)
- [ ] LICENSE-MIT
- [ ] LICENSE-APACHE
- [ ] README.md (with license section per spec §25)
- [ ] deny.toml (license enforcement, spec §21)
- [ ] rustfmt.toml
- [ ] clippy.toml
- [ ] .github/workflows/ci.yml (fmt, clippy, test, deny)
- [ ] .github/workflows/license.yml
- [ ] crates/, examples/, test-data/ directory scaffolding
- [ ] Source file header template (SPDX-License-Identifier: MIT OR Apache-2.0)

## Phase 1: Foundation
### tpt-c-core
- [ ] Scaffold Cargo.toml + lib.rs (tpt_c_core)
- [ ] Define ID types (ProjectId, ContractId, ModelId, ElementId, EstimateId, ScheduleId, ActivityId, RFIId, SubmittalId, AssetId)
- [ ] Common traits, error types (thiserror), project context, audit metadata
- [ ] Unit tests
- [ ] Rustdoc + SPDX header
### tpt-c-ids
- [ ] Scaffold crate
- [ ] Deterministic ID generation, UUIDv7 support
- [ ] External ID mapping (IFC GUID, Revit element IDs, cost codes, asset tags)
- [ ] Unit tests
- [ ] Rustdoc + SPDX header
### tpt-c-units
- [ ] Scaffold crate
- [ ] Unit types (LF, SF, SY, CY, each, hour, day, week, kg, ton, m3, m2)
- [ ] Conversions, rounding, precision, waste factors
- [ ] Bank/loose/compacted measure, formwork area, rebar weight, paint coverage
- [ ] Unit tests
- [ ] Rustdoc + SPDX header
### tpt-c-classification
- [ ] Scaffold crate
- [ ] MasterFormat, UniFormat, OmniClass, Uniclass support
- [ ] Custom project classification mapping
- [ ] Unit tests
- [ ] Rustdoc + SPDX header
### tpt-c-model
- [ ] Scaffold crate
- [ ] Core types (Project, Site, Building, Storey, Zone, System, Element, Assembly, MaterialLayer, PropertySet, QuantitySet)
- [ ] Unit tests
- [ ] Rustdoc + SPDX header
- [ ] Phase 1 integration check: crates compile together in workspace, `cargo test --workspace` green

## Phase 2: Geometry and Formats
### tpt-c-geometry
- [ ] Scaffold crate (depends on tpt-math, tpt-engineering)
- [ ] Points, vectors, meshes, solids, bounding boxes, spatial indexes
- [ ] Area/volume calculation, surface extraction, clash detection primitives
- [ ] Unit tests + rustdoc + SPDX header
### tpt-c-geo
- [ ] Scaffold crate
- [ ] CRS, local site coordinates, lat/long, elevation, grids, datums, transformations
- [ ] Unit tests + rustdoc + SPDX header
### tpt-c-ifc
- [ ] Scaffold crate
- [ ] IFC parser (IfcProject, IfcSite, IfcBuilding, IfcBuildingStorey, IfcWall, IfcSlab, IfcColumn, IfcBeam, IfcDoor, IfcWindow, IfcSpace, IfcPropertySet, IfcQuantitySet)
- [ ] Map IFC entities to tpt-c-model
- [ ] Unit tests (golden test-data/ifc fixtures) + rustdoc + SPDX header
### tpt-c-bcf
- [ ] Scaffold crate
- [ ] Issue tracking, viewpoints, comments, statuses, assignments
- [ ] Unit tests (test-data/bcf fixtures) + rustdoc + SPDX header
### tpt-c-gltf
- [ ] Scaffold crate
- [ ] Export construction models to glTF/web-ready meshes with metadata mapping
- [ ] Unit tests + rustdoc + SPDX header
- [ ] Phase 2 integration check: example `examples/ifc-import` parses a sample IFC into tpt-c-model

## Phase 3: Estimating MVP
### tpt-c-quantities
- [ ] Scaffold crate
- [ ] Takeoff engine: count, length, area, volume, weight, net vs gross, waste factors, manual overrides
- [ ] Rule set (concrete volume, formwork area, rebar weight, paint area, flooring area deductions)
- [ ] Unit tests + rustdoc + SPDX header
### tpt-c-cost
- [ ] Scaffold crate
- [ ] Types: CostItem, CostAssembly, ResourceRate, LineItem, CostCode, Budget, Estimate
- [ ] Labor/material/equipment/subcontractor rates, overhead, profit, tax, escalation
- [ ] Unit tests + rustdoc + SPDX header
### tpt-c-estimating
- [ ] Scaffold crate
- [ ] Estimate builder, bid prep, pricing workflows, revisions, estimate comparison, cost planning
- [ ] Workflow: model/drawing → quantities → cost items → line items → estimate → budget
- [ ] Unit tests + rustdoc + SPDX header
### tpt-c-csv
- [ ] Scaffold crate
- [ ] Import/export tabular data (estimates, cost DBs, schedules, equipment logs, daily reports)
- [ ] Unit tests + rustdoc + SPDX header
### tpt-c-xlsx
- [ ] Scaffold crate
- [ ] Excel import/export (estimator workflows, owner reports, bid summaries, budget exports)
- [ ] Unit tests + rustdoc + SPDX header
### First Vertical Slice (spec §24)
- [ ] Build `examples/quantity-takeoff` and `examples/estimate-export`
- [ ] Implement CLI: `tpt-c estimate model.ifc --cost-db rates.csv --output estimate.xlsx`
- [ ] End-to-end test: IFC → elements → properties/quantities → classification → cost items → exported estimate
- [ ] Golden test-data validation (test-data/golden)

## Phase 4: Scheduling
### tpt-c-schedule
- [x] Scaffold crate
- [x] Activities, relationships (FS/SS/FF/SF), constraints, calendars, durations, lag/lead
- [x] CPM engine, float, baselines, actuals
- [x] Unit tests + rustdoc + SPDX header
### tpt-c-risk
- [x] Scaffold crate (self-contained prob module stands in for tpt-math-prob-dist; does not depend on the absent tpt-c-estimating)
- [x] Schedule/cost risk, Monte Carlo simulation, weather risk, productivity uncertainty
- [x] Unit tests + rustdoc + SPDX header
### tpt-c-earned-value
- [x] Scaffold crate
- [x] EVM: SPI, CPI, TCPI, BCWS, BCWP, ACWP
- [x] Unit tests + rustdoc + SPDX header
- [x] Phase 4 integration check: `examples/cpm-schedule` runs a sample CPM schedule end-to-end

## Phase 5: Civil and Earthwork
### tpt-c-earthwork
- [ ] Scaffold crate (depends on tpt-c-geo, tpt-c-geometry, tpt-math-optimize-general, tpt-engineering)
- [ ] Cut/fill calculation, mass haul diagrams, volume balancing, haul route optimization, shrink/swell factors
- [ ] Unit tests + rustdoc + SPDX header
### tpt-c-alignment
- [ ] Scaffold crate
- [ ] Horizontal/vertical alignments, curves, superelevation, stationing, corridor modeling
- [ ] Unit tests + rustdoc + SPDX header
### tpt-c-las
- [ ] Scaffold crate
- [ ] LAS/LAZ point cloud parsing, filtering, downsampling, classification, point cloud volumes
- [ ] Unit tests (test-data/las fixtures) + rustdoc + SPDX header
- [ ] Phase 5 integration check: `examples/earthwork-cut-fill` runs end-to-end

## Phase 6: Field Execution
### tpt-c-field
- [ ] Scaffold crate
- [ ] Daily logs, field reports, work records, manpower tracking, weather records, site observations
- [ ] Unit tests + rustdoc + SPDX header
### tpt-c-workflow
- [ ] Scaffold crate
- [ ] Approval workflows/state machines for RFI, Submittal, Punch List, Issue, Change Order, Notice, Transmittal
- [ ] Unit tests + rustdoc + SPDX header
### tpt-c-documents
- [ ] Scaffold crate
- [ ] Documents, drawings, specifications, revisions, transmittals, document control
- [ ] Unit tests + rustdoc + SPDX header
### tpt-c-contracts
- [ ] Scaffold crate
- [ ] Contracts, contract items, responsibilities, notices, claims, compliance events
- [ ] Unit tests + rustdoc + SPDX header
### tpt-c-payapps
- [ ] Scaffold crate
- [ ] Progress claims, payment applications, schedule of values, retention, approvals, certified amounts
- [ ] Unit tests + rustdoc + SPDX header
### tpt-c-safety
- [ ] Scaffold crate
- [ ] Incidents, near misses, safety observations, toolbox talks, inspections, compliance checklists
- [ ] Unit tests + rustdoc + SPDX header
- [ ] Phase 6 integration check: field workflow end-to-end (daily log → RFI → document → pay app) test

## Phase 7: Equipment and Sync
### tpt-c-equipment
- [x] Scaffold crate
- [x] Equipment registry, utilization tracking, fuel tracking, idle time, maintenance triggers, telematics events (J1939 CAN bus, OEM telematics, GPS, fuel/maintenance logs)
- [x] Unit tests + rustdoc + SPDX header
### tpt-c-sync
- [x] Scaffold crate
- [x] Offline-first sync, local storage, conflict resolution, CRDTs, connectivity resilience
- [x] Unit tests + rustdoc + SPDX header
### tpt-c-events
- [x] Scaffold crate
- [x] Domain events, audit trail, event sourcing, projections, replay (ModelImported, QuantityAdjusted, CostItemApplied, EstimateApproved, ScheduleUpdated, RFICreated, RFIAnswered, SubmittalApproved, PaymentApplicationSubmitted)
- [x] Unit tests + rustdoc + SPDX header
### tpt-c-db
- [x] Scaffold crate
- [x] Persistence patterns: PostgreSQL, SQLite, migrations, repositories
- [x] Unit tests + rustdoc + SPDX header
- [x] Phase 7 integration check: `examples/field-offline-sync` demonstrates offline write + resync

## Phase 8: Facility Management and Digital Twins
### tpt-c-fm
- [ ] Scaffold crate
- [ ] COBie support, handover data, asset registers, spaces and systems
- [ ] Unit tests + rustdoc + SPDX header
### tpt-c-assets
- [ ] Scaffold crate
- [ ] Asset lifecycle, warranties, serial numbers, replacements, maintenance schedules
- [ ] Unit tests + rustdoc + SPDX header
### tpt-c-maintenance
- [ ] Scaffold crate
- [ ] Preventive/corrective maintenance, work orders, service history
- [ ] Unit tests + rustdoc + SPDX header
### tpt-c-twin
- [ ] Scaffold crate (depends on tpt-c-model, tpt-c-equipment, tpt-c-assets, tpt-c-fm)
- [ ] Digital twin state, sensor mapping, live telemetry, spatial asset context
- [ ] Unit tests + rustdoc + SPDX header
### tpt-c-space
- [ ] Scaffold crate
- [ ] Spaces, occupancy, leases, areas, space planning, space utilization
- [ ] Unit tests + rustdoc + SPDX header
- [ ] Phase 8 integration check: FM/twin crates compile and interoperate in a sample scenario

## Phase 9: Web Platform Enablement
### tpt-c-wasm
- [ ] Scaffold crate
- [ ] WASM bindings for IFC parsing, quantity takeoff, CPM scheduling, clash detection prep, model simplification, glTF export
- [ ] Browser build target verified (wasm-pack or similar)
- [ ] Unit tests + rustdoc + SPDX header
### tpt-c-api
- [ ] Scaffold crate
- [ ] REST/GraphQL API models, auth patterns, authorization, project access control, webhooks, audit logging
- [ ] Unit tests + rustdoc + SPDX header
- [ ] Phase 9 integration check: browser demo consuming tpt-c-wasm

## Ongoing / Cross-Cutting
- [ ] Maintain cargo-deny license checks passing on every phase (spec §21)
- [ ] Keep CI green (fmt, clippy, test, deny) after each crate lands
- [ ] Update Industry Coverage Map (spec §19) as crates land
- [ ] Isolate any unavoidable Apache-only dependency behind an optional feature, documented per spec §4
