# tpt-c-db

Storage-agnostic persistence patterns for tpt-construction: a generic
`Repository` read/write abstraction, two ready-made backends (in-memory and
JSON-file), and a versioned schema/data migration framework built from
`Migration`, `MigrationRegistry` and `Migrator`. Production deployments
typically implement the traits against PostgreSQL or SQLite, while the shipped
backends keep domain crates storage-agnostic and the test suite hermetic.
Reach for it when you need key-value entity storage or ordered, run-once
migrations without tying domain logic to a database vendor.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `Repository<K, V>` trait: `get`, `insert`, `remove`, `contains`, `len`,
  `is_empty`, plus `keys`/`values` for clonable keys and values.
- `InMemoryRepository`: hash-map-backed implementation (`Clone`, `Default`).
- `JsonFileRepository`: loads an existing JSON file on `open` and rewrites it
  after every `insert`/`remove`.
- `MigrationTarget` trait with `RecordingTarget`, a test backend that captures
  every executed statement in order.
- `Migration` and `MigrationFn`: boxed up-migrations carrying a unique
  `u32` version, name and description.
- `MigrationRegistry`: rejects duplicate versions and keeps migrations sorted;
  `Migrator::run_pending` applies each migration once, in ascending version
  order, and aborts without losing state if one fails.
- `AppliedVersions` log that can be saved to and reloaded from JSON via
  `Migrator::save_state`/`load_state`.
- `DbError` covering I/O, JSON, duplicate-version and migration-failure cases.

## Usage

```toml
[dependencies]
tpt-c-db = "0.1"
```

```rust
use tpt_c_db::{InMemoryRepository, Migration, Migrator, RecordingTarget, Repository};

// Key-value entity storage.
let mut repo: InMemoryRepository<String, u32> = InMemoryRepository::new();
repo.insert("a".into(), 1);
assert_eq!(repo.get(&"a".into()), Some(&1));
assert_eq!(repo.remove(&"a".into()), Some(1));

// Ordered, run-once migrations against any MigrationTarget.
let mut migrator = Migrator::new(Box::new(RecordingTarget::default()));
migrator.register(Migration::new(
    1,
    "init",
    "create tables",
    Box::new(|t| t.execute("CREATE TABLE equipment")),
))?;
assert_eq!(migrator.run_pending()?, vec![1]); // a second run returns []
# Ok::<(), tpt_c_db::DbError>(())
```

## Crate relationships

- **Depends on:** `serde`, `serde_json`, `thiserror` — no other workspace crates.
- **Used by:** Not yet consumed by other workspace crates; see the examples/ directory for integration usage.

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) / [LICENSE-APACHE](../../LICENSE-APACHE)).
