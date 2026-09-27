# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `Incident` with ordered `Severity` (`Low`, `Medium`, `High`, `Critical`),
  `IncidentStatus` (`Open`, `Investigating`, `Closed`), injury flag, optional
  outcome note, and `begin_investigation` / `close` transitions.
- `NearMiss` and `SafetyObservation` with `SafetyCategory` (`Housekeeping`,
  `Height`, `Electrical`, `Plant`, `Environmental`, `Other`), `review` and
  `action` tracking respectively.
- `ToolboxTalk` with `with_id`, `add_attendee`, and `attendee_count`.
- `Inspection` with `InspectionResult` (`Pass`, `PassWithNotes`, `Fail`),
  `with_note`, and `passed`.
- `ComplianceChecklist` / `ChecklistItem` with `ChecklistStatus` (`Pending`,
  `Done`, `NotApplicable`), `add_item`, `open_items` (N/A excluded), and
  `is_complete`.
- `SafetyError` (`NotFound`).
