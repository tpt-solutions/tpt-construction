# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `ScheduleNetwork` with `new`, `add_activity`, `add_dependency`, `add_constraint`, `activities`, and `schedule` running the CPM forward/backward passes.
- `CpmResult` (per-activity schedules, `project_duration`, `critical_path`) and `ActivitySchedule` (early/late dates, `total_float`, `free_float`, `critical`).
- `Activity` / `ActivityStatus`, `Dependency` (`RelationshipType` FS/SS/FF/SF with lag), and `ActivityConstraint` (`ConstraintType` SNET/SLNT/FNET/FLNT/MSO/MFO).
- `Schedule` aggregate with `new`, `with_id`, `add_activity`, `add_dependency`, `add_constraint`, `compute`, `activities`, `activity_dates`, and `project_finish_date`.
- `Calendar` with `standard_40h`, `is_working`, `add_working_hours`, and `working_hours_between`; crate-local `NaiveDate` and `Weekday`.
- `Baseline` snapshot capture and `Actuals` with `finish_variance_hours`.
- `ScheduleError` (`DuplicateActivity`, `UnknownActivity`, `DanglingDependency`, `CycleDetected`, `InvalidDate`, `EmptySchedule`) and the crate `Result` alias.
