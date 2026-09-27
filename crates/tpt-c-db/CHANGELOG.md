# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `Repository<K, V>` trait for key-value entity storage.
- `InMemoryRepository`, a hash-map-backed `Repository` implementation.
- `JsonFileRepository`, a `Repository` that persists its map to a JSON file on
  every mutation and reloads existing data on `open`.
- `MigrationTarget` trait and the `RecordingTarget` statement-capturing test
  backend.
- `Migration`, `MigrationFn` and `MigrationRegistry` for declaring ordered,
  duplicate-free migrations.
- `Migrator` with `run_pending`, `applied_versions`, `save_state`/`load_state`,
  backed by the serializable `AppliedVersions` log.
- `DbError` covering I/O, JSON, duplicate-version and migration-failure cases.
