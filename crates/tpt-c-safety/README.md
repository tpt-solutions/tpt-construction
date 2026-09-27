# tpt-c-safety

Safety management for construction sites. Captures the full safety picture:
reportable `Incident`s with a severity scale and investigation lifecycle,
`NearMiss`es, `SafetyObservation`s with corrective-action tracking, `ToolboxTalk`
briefings with attendee lists, `Inspection`s of systems and areas, and the
`ComplianceChecklist`s used to verify controls are in place. Severity and status
enums give each record a clear lifecycle.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `Incident` with ordered `Severity` (`Low` < `Medium` < `High` < `Critical`),
  injury flag, and an `IncidentStatus` lifecycle (`begin_investigation`,
  `close`).
- `NearMiss` with `SafetyCategory` (housekeeping, height, electrical, plant,
  environmental, other), reporter, and a `review()` flag.
- `SafetyObservation` with category and corrective-action `action()` tracking.
- `ToolboxTalk` with topic, date, explicit `with_id` reference, and
  `add_attendee` / `attendee_count`.
- `Inspection` with `InspectionResult` (`Pass`, `PassWithNotes`, `Fail`), optional
  note, and a `passed()` helper that treats pass-with-notes as passing.
- `ComplianceChecklist` of `ChecklistItem`s (`Pending`, `Done`, `NotApplicable`)
  with `open_items()` (N/A excluded) and `is_complete()`.
- Serializable with serde; a pure core crate with no IO.

## Usage

```toml
[dependencies]
tpt-c-safety = "0.1"
```

```rust
use tpt_c_ids::SafetyIncidentId;
use tpt_c_safety::{ComplianceChecklist, Incident, IncidentStatus, Severity, ToolboxTalk};
use uuid::Uuid;

// Record and investigate an incident.
let mut inc = Incident::new(
    SafetyIncidentId::from_uuid(Uuid::now_v7()),
    "Fall from height",
    Severity::High,
);
assert_eq!(inc.status, IncidentStatus::Open);
inc.begin_investigation();
inc.close();

// Verify controls with a checklist.
let mut c = ComplianceChecklist::new("Pre-task");
c.add_item("Barricades in place");
c.add_item("Spotter assigned");
assert_eq!(c.open_items(), 2);
c.items[0].mark_done();
c.items[1].mark_done();
assert!(c.is_complete());

// Log a toolbox talk with attendees.
let mut t = ToolboxTalk::new("Excavation safety", "2026-03-01");
t.add_attendee("alice");
t.add_attendee("bob");
assert_eq!(t.attendee_count(), 2);
```

## Crate relationships

- **Depends on:** `serde`, `thiserror`, `uuid`, `tpt-c-core`, and `tpt-c-ids`
  (`SafetyIncidentId`).
- **Used by:** Not yet consumed by other workspace crates;
  [examples/field-execution-e2e](../../examples/field-execution-e2e) records and
  investigates an incident in its scenario.

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) / [LICENSE-APACHE](../../LICENSE-APACHE)).
