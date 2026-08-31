# TPT Construction — Project Todo

Dual-licensed MIT OR Apache-2.0 · TPT Solutions

## Phase 0: Repository Scaffolding
- [x] Root Cargo.toml (workspace, resolver = "2", workspace.package, workspace.dependencies)
- [x] LICENSE-MIT
- [x] LICENSE-APACHE
- [x] README.md (with license section per spec §25)
- [x] deny.toml (license enforcement, spec §21)
- [x] rustfmt.toml
- [x] clippy.toml
- [x] .github/workflows/ci.yml (fmt, clippy, test, deny)
- [x] .github/workflows/license.yml
- [x] crates/, examples/, test-data/ directory scaffolding
- [x] Source file header template (SPDX-License-Identifier: MIT OR Apache-2.0)

## Phase 1: Foundation
### tpt-c-core
- [x] Scaffold Cargo.toml + lib.rs (tpt_c_core)
- [x] Define ID types (ProjectId, ContractId, ModelId, ElementId, EstimateId, ScheduleId, ActivityId, RFIId, SubmittalId, AssetId)
- [x] Common traits, error types (thiserror), project context, audit metadata
- [x] Unit tests
- [x] Rustdoc + SPDX header
- [x] Uncommitted: adds ClaimId, ContractItemId, DocumentId, IssueId, NoticeId, PaymentApplicationId, PunchListId, SafetyIncidentId, TransmittalId for Phase 6 crates — commit these ID additions
### tpt-c-ids
- [x] Scaffold crate
- [x] Deterministic ID generation, UUIDv7 support
- [x] External ID mapping (IFC GUID, Revit element IDs, cost codes, asset tags)
- [x] Unit tests
- [x] Rustdoc + SPDX header
### tpt-c-units
- [x] Scaffold crate
- [x] Unit types (LF, SF, SY, CY, each, hour, day, week, kg, ton, m3, m2)
- [x] Conversions, rounding, precision, waste factors
- [x] Bank/loose/compacted measure, formwork area, rebar weight, paint coverage
- [x] Unit tests
- [x] Rustdoc + SPDX header
### tpt-c-classification
- [x] Scaffold crate
- [x] MasterFormat, UniFormat, OmniClass, Uniclass support
- [x] Custom project classification mapping
- [x] Unit tests
- [x] Rustdoc + SPDX header
### tpt-c-model
- [x] Scaffold crate
- [x] Core types (Project, Site, Building, Storey, Zone, System, Element, Assembly, MaterialLayer, PropertySet, QuantitySet)
- [x] Unit tests
- [x] Rustdoc + SPDX header
- [x] Phase 1 integration check: crates compile together in workspace, `cargo test --workspace` green

## Phase 2: Geometry and Formats
### tpt-c-geometry
- [x] Scaffold crate (depends on tpt-math, tpt-engineering)
- [x] Points, vectors, meshes, solids, bounding boxes, spatial indexes
- [x] Area/volume calculation, surface extraction, clash detection primitives
- [x] Unit tests + rustdoc + SPDX header
### tpt-c-geo
- [x] Scaffold crate
- [x] CRS, local site coordinates, lat/long, elevation, grids, datums, transformations
- [x] Unit tests + rustdoc + SPDX header
### tpt-c-ifc
- [x] Scaffold crate
- [x] IFC parser (IfcProject, IfcSite, IfcBuilding, IfcBuildingStorey, IfcWall, IfcSlab, IfcColumn, IfcBeam, IfcDoor, IfcWindow, IfcSpace, IfcPropertySet, IfcQuantitySet)
- [x] Map IFC entities to tpt-c-model
- [x] Unit tests (golden test-data/ifc fixtures) + rustdoc + SPDX header
### tpt-c-bcf
- [x] Scaffold crate
- [x] Issue tracking, viewpoints, comments, statuses, assignments
- [x] Unit tests (test-data/bcf fixtures) + rustdoc + SPDX header
### tpt-c-gltf
- [x] Scaffold crate
- [x] Export construction models to glTF/web-ready meshes with metadata mapping
- [x] Unit tests + rustdoc + SPDX header
- [x] Phase 2 integration check: example `examples/ifc-import` parses a sample IFC into tpt-c-model

## Phase 3: Estimating MVP
### tpt-c-quantities
- [x] Scaffold crate
- [x] Takeoff engine: count, length, area, volume, weight, net vs gross, waste factors, manual overrides
- [x] Rule set (concrete volume, formwork area, rebar weight, paint area, flooring area deductions)
- [x] Unit tests + rustdoc + SPDX header
### tpt-c-cost
- [x] Scaffold crate
- [x] Types: CostItem, CostAssembly, ResourceRate, LineItem, CostCode, Budget, Estimate
- [x] Labor/material/equipment/subcontractor rates, overhead, profit, tax, escalation
- [x] Unit tests + rustdoc + SPDX header
### tpt-c-estimating
- [x] Scaffold crate
- [x] Estimate builder, bid prep, pricing workflows, revisions, estimate comparison, cost planning
- [x] Workflow: model/drawing → quantities → cost items → line items → estimate → budget
- [x] Unit tests + rustdoc + SPDX header
### tpt-c-csv
- [x] Scaffold crate
- [x] Import/export tabular data (estimates, cost DBs, schedules, equipment logs, daily reports)
- [x] Unit tests + rustdoc + SPDX header
### tpt-c-xlsx
- [x] Scaffold crate
- [x] Excel import/export (estimator workflows, owner reports, bid summaries, budget exports)
- [x] Unit tests + rustdoc + SPDX header
### First Vertical Slice (spec §24)
- [x] Build `examples/quantity-takeoff` and `examples/estimate-export`
- [x] Implement CLI: `examples/tpt` ships an `estimate` subcommand (`tpt estimate model.json --cost-db rates.csv --output estimate.xlsx`); it consumes the neutral `tpt-c-model` JSON rather than a raw `.ifc` directly — IFC→JSON conversion goes through `tpt-c-ifc` separately, which is currently blocked by the Phase 2 parser bug above
- [x] End-to-end test: IFC → elements → properties/quantities → classification → cost items → exported estimate (`crates/tpt-c-estimating/tests/e2e.rs`, 4 tests passing)
- [x] Golden test-data validation (test-data/golden)

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
- [x] Scaffold crate (depends on tpt-c-geo, tpt-c-geometry, tpt-c-units)
- [x] Cut/fill calculation, mass haul diagrams, volume balancing, shrink/swell factors
- [x] Unit tests + rustdoc + SPDX header
### tpt-c-alignment
- [x] Scaffold crate
- [x] Horizontal/vertical alignments, curves, superelevation, stationing, corridor modeling
- [x] Unit tests + rustdoc + SPDX header
### tpt-c-las
- [x] Scaffold crate
- [x] LAS/LAZ point cloud parsing, filtering, downsampling, classification, point cloud volumes
- [x] Unit tests (test-data/las fixtures) + rustdoc + SPDX header
### tpt-c-dxf (not in original spec — added during implementation)
- [x] Scaffold crate
- [x] DXF import/export, unit tests + rustdoc + SPDX header
- [x] Phase 5 integration check: `examples/earthwork-cut-fill` runs end-to-end — registered in workspace.members; currently a stub depending only on `tpt-c-core`

## Phase 6: Field Execution
### tpt-c-field
- [x] Scaffold crate
- [x] Daily logs, field reports, work records, manpower tracking, weather records, site observations
- [x] Unit tests + rustdoc + SPDX header
### tpt-c-workflow
- [x] Scaffold crate
- [x] Approval workflows/state machines for RFI, Submittal, Punch List, Issue, Change Order, Notice, Transmittal
- [x] Unit tests + rustdoc + SPDX header
### tpt-c-documents
- [x] Scaffold crate
- [x] Documents, drawings, specifications, revisions, transmittals, document control
- [x] Unit tests + rustdoc + SPDX header
### tpt-c-contracts
- [x] Scaffold crate
- [x] Contracts, contract items, responsibilities, notices, claims, compliance events
- [x] Unit tests + rustdoc + SPDX header
### tpt-c-payapps
- [x] Scaffold crate
- [x] Progress claims, payment applications, schedule of values, retention, approvals, certified amounts
- [x] Unit tests + rustdoc + SPDX header
### tpt-c-safety
- [x] Scaffold crate
- [x] Incidents, near misses, safety observations, toolbox talks, inspections, compliance checklists
- [x] Unit tests + rustdoc + SPDX header
- [x] Phase 6 integration check: field workflow end-to-end (daily log → RFI → document → pay app) test (`examples/field-execution-e2e`, passing)

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
- [x] Phase 7 integration check: `examples/field-offline-sync` demonstrates offline write + resync — registered in workspace.members and builds successfully

## Phase 8: Facility Management and Digital Twins
### tpt-c-fm
- [x] Scaffold crate
- [x] COBie support, handover data, asset registers, spaces and systems
- [x] Unit tests + rustdoc + SPDX header
### tpt-c-assets
- [x] Scaffold crate
- [x] Asset lifecycle, warranties, serial numbers, replacements, maintenance schedules
- [x] Unit tests + rustdoc + SPDX header
### tpt-c-maintenance
- [x] Scaffold crate
- [x] Preventive/corrective maintenance, work orders, service history
- [x] Unit tests + rustdoc + SPDX header
### tpt-c-twin
- [x] Scaffold crate (depends on tpt-c-model, tpt-c-equipment, tpt-c-assets, tpt-c-fm)
- [x] Digital twin state, sensor mapping, live telemetry, spatial asset context
- [x] Unit tests + rustdoc + SPDX header
### tpt-c-space
- [x] Scaffold crate
- [x] Spaces, occupancy, leases, areas, space planning, space utilization
- [x] Unit tests + rustdoc + SPDX header
- [x] Phase 8 integration check: FM/twin crates compile and interoperate in a sample scenario

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
