// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Persistence patterns: repositories, pluggable storage backends, and a
//! versioned migration framework.
//!
//! Production deployments back these abstractions with PostgreSQL or SQLite, but
//! the same `Repository` and `MigrationTarget` traits are implemented by the
//! offline/test backends shipped here (in-memory and JSON-file). This keeps the
//! domain crates storage-agnostic and the test suite hermetic.
//!
//! A [`Repository`] is the unit of read/write for an entity, while a
//! [`Migrator`] applies ordered schema/data migrations against a
//! [`MigrationTarget`], tracking which versions have already been applied.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::hash::Hash;
use std::path::Path;
use thiserror::Error;

/// Errors produced by the persistence layer.
#[derive(Debug, Error)]
pub enum DbError {
    /// Underlying I/O failure (file backends).
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// JSON (de)serialization failure.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    /// A migration with a duplicate version was registered.
    #[error("duplicate migration version: {0}")]
    DuplicateMigration(u32),
    /// A migration reported failure.
    #[error("migration '{name}' failed: {detail}")]
    MigrationFailed {
        /// Migration name.
        name: String,
        /// Underlying message.
        detail: String,
    },
}

/// A read/write store for entities keyed by `K`.
pub trait Repository<K, V> {
    /// Fetch an entity by key.
    fn get(&self, key: &K) -> Option<&V>;
    /// Insert an entity, returning the previous value if any.
    fn insert(&mut self, key: K, value: V) -> Option<V>;
    /// Remove an entity, returning it if present.
    fn remove(&mut self, key: &K) -> Option<V>;
    /// Whether a key is present.
    fn contains(&self, key: &K) -> bool;
    /// Number of stored entities.
    fn len(&self) -> usize;
    /// Whether the store is empty.
    fn is_empty(&self) -> bool;
    /// All keys (requires `K: Clone`).
    fn keys(&self) -> Vec<K>
    where
        K: Clone;
    /// All values (requires `V: Clone`).
    fn values(&self) -> Vec<V>
    where
        V: Clone;
}

/// An in-memory [`Repository`] backed by a hash map.
#[derive(Clone, Debug)]
pub struct InMemoryRepository<K, V> {
    map: HashMap<K, V>,
}

impl<K, V> Default for InMemoryRepository<K, V> {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}

impl<K, V> InMemoryRepository<K, V> {
    /// Create an empty repository.
    pub fn new() -> Self {
        Self::default()
    }
}

impl<K: Eq + Hash, V> Repository<K, V> for InMemoryRepository<K, V> {
    fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.map.insert(key, value)
    }
    fn remove(&mut self, key: &K) -> Option<V> {
        self.map.remove(key)
    }
    fn contains(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }
    fn len(&self) -> usize {
        self.map.len()
    }
    fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
    fn keys(&self) -> Vec<K>
    where
        K: Clone,
    {
        self.map.keys().cloned().collect()
    }
    fn values(&self) -> Vec<V>
    where
        V: Clone,
    {
        self.map.values().cloned().collect()
    }
}

/// A [`Repository`] that persists its map to a JSON file on every mutation.
#[derive(Clone, Debug)]
pub struct JsonFileRepository<K, V> {
    path: std::path::PathBuf,
    map: HashMap<K, V>,
}

impl<K, V> JsonFileRepository<K, V>
where
    K: Eq + Hash + Serialize + for<'de> Deserialize<'de>,
    V: Serialize + for<'de> Deserialize<'de>,
{
    /// Open a JSON-backed repository, loading existing data if the file exists.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, DbError> {
        let path = path.as_ref().to_path_buf();
        let map = if path.exists() {
            let raw = fs::read_to_string(&path)?;
            serde_json::from_str(&raw)?
        } else {
            HashMap::new()
        };
        Ok(Self { path, map })
    }

    fn save(&self) -> Result<(), DbError> {
        let raw = serde_json::to_string_pretty(&self.map)?;
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(&self.path, raw)?;
        Ok(())
    }
}

impl<K: Eq + Hash, V> Repository<K, V> for JsonFileRepository<K, V>
where
    K: Serialize + for<'de> Deserialize<'de>,
    V: Serialize + for<'de> Deserialize<'de>,
{
    fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        let prev = self.map.insert(key, value);
        let _ = self.save();
        prev
    }
    fn remove(&mut self, key: &K) -> Option<V> {
        let prev = self.map.remove(key);
        let _ = self.save();
        prev
    }
    fn contains(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }
    fn len(&self) -> usize {
        self.map.len()
    }
    fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
    fn keys(&self) -> Vec<K>
    where
        K: Clone,
    {
        self.map.keys().cloned().collect()
    }
    fn values(&self) -> Vec<V>
    where
        V: Clone,
    {
        self.map.values().cloned().collect()
    }
}

/// A target that migrations execute statements against.
///
/// Concrete backends (SQLite, PostgreSQL, ...) implement this with real DDL/DML.
/// The test backend ([`RecordingTarget`]) captures statements for assertions.
pub trait MigrationTarget {
    /// Execute a single migration statement.
    fn execute(&mut self, statement: &str) -> Result<(), DbError>;
}

/// A [`MigrationTarget`] that records every executed statement.
#[derive(Clone, Debug, Default)]
pub struct RecordingTarget {
    statements: Vec<String>,
}

impl RecordingTarget {
    /// Statements executed so far, in order.
    pub fn statements(&self) -> &[String] {
        &self.statements
    }
}

impl MigrationTarget for RecordingTarget {
    fn execute(&mut self, statement: &str) -> Result<(), DbError> {
        self.statements.push(statement.to_string());
        Ok(())
    }
}

/// A boxed migration function executed against a [`MigrationTarget`].
pub type MigrationFn = Box<dyn Fn(&mut dyn MigrationTarget) -> Result<(), DbError>>;

/// A single ordered schema/data migration.
pub struct Migration {
    /// Monotonic version number (must be unique within a registry).
    pub version: u32,
    /// Short name.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// The up-migration, executed against a [`MigrationTarget`].
    pub up: MigrationFn,
}

impl Migration {
    /// Construct a migration.
    pub fn new(
        version: u32,
        name: impl Into<String>,
        description: impl Into<String>,
        up: MigrationFn,
    ) -> Self {
        Self {
            version,
            name: name.into(),
            description: description.into(),
            up,
        }
    }
}

/// Tracks which migration versions have already been applied.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AppliedVersions {
    /// Applied migration versions, ascending.
    pub versions: Vec<u32>,
}

impl AppliedVersions {
    /// Whether `version` has been applied.
    pub fn contains(&self, version: u32) -> bool {
        self.versions.contains(&version)
    }
    /// Record that `version` was applied (idempotent for ordering checks).
    pub fn mark(&mut self, version: u32) {
        if !self.contains(version) {
            self.versions.push(version);
            self.versions.sort_unstable();
        }
    }
    /// Persist the applied-version log to `path` as JSON.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), DbError> {
        let raw = serde_json::to_string_pretty(self)?;
        if let Some(parent) = path.as_ref().parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(path, raw)?;
        Ok(())
    }
    /// Load an applied-version log from `path` (empty if absent).
    pub fn load(path: impl AsRef<Path>) -> Result<Self, DbError> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&raw)?)
    }
}

/// Orders and validates registered migrations, computing pending work.
#[derive(Default)]
pub struct MigrationRegistry {
    migrations: Vec<Migration>,
}

impl MigrationRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            migrations: Vec::new(),
        }
    }

    /// Register a migration, rejecting duplicate versions.
    pub fn register(&mut self, migration: Migration) -> Result<(), DbError> {
        if self
            .migrations
            .iter()
            .any(|m| m.version == migration.version)
        {
            return Err(DbError::DuplicateMigration(migration.version));
        }
        self.migrations.push(migration);
        self.migrations.sort_by_key(|m| m.version);
        Ok(())
    }

    /// Migrations not yet applied, in ascending version order.
    pub fn pending(&self, applied: &AppliedVersions) -> Vec<&Migration> {
        self.migrations
            .iter()
            .filter(|m| !applied.contains(m.version))
            .collect()
    }

    /// The highest registered version, if any.
    pub fn latest_version(&self) -> Option<u32> {
        self.migrations.iter().map(|m| m.version).max()
    }
}

/// Applies pending migrations against a [`MigrationTarget`], tracking state.
pub struct Migrator {
    registry: MigrationRegistry,
    applied: AppliedVersions,
    target: Box<dyn MigrationTarget>,
}

impl Migrator {
    /// Create a migrator targeting `target`, starting from no applied versions.
    pub fn new(target: Box<dyn MigrationTarget>) -> Self {
        Self {
            registry: MigrationRegistry::new(),
            applied: AppliedVersions::default(),
            target,
        }
    }

    /// Register a migration.
    pub fn register(&mut self, migration: Migration) -> Result<(), DbError> {
        self.registry.register(migration)
    }

    /// Currently applied versions.
    pub fn applied_versions(&self) -> &[u32] {
        &self.applied.versions
    }

    /// Apply all not-yet-applied migrations in version order.
    ///
    /// Each successful migration is recorded; a failing migration aborts the
    /// run and leaves already-applied versions intact.
    pub fn run_pending(&mut self) -> Result<Vec<u32>, DbError> {
        let pending = self.registry.pending(&self.applied);
        let mut applied_now = Vec::new();
        for m in pending {
            (m.up)(self.target.as_mut()).map_err(|e| DbError::MigrationFailed {
                name: m.name.clone(),
                detail: e.to_string(),
            })?;
            self.applied.mark(m.version);
            applied_now.push(m.version);
        }
        Ok(applied_now)
    }

    /// Load previously applied versions from `path`.
    pub fn load_state(&mut self, path: impl AsRef<Path>) -> Result<(), DbError> {
        self.applied = AppliedVersions::load(path)?;
        Ok(())
    }

    /// Persist applied versions to `path`.
    pub fn save_state(&self, path: impl AsRef<Path>) -> Result<(), DbError> {
        self.applied.save(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_memory_repository_crud() {
        let mut repo: InMemoryRepository<String, u32> = InMemoryRepository::new();
        assert!(repo.is_empty());
        assert!(repo.insert("a".into(), 1).is_none());
        assert_eq!(repo.get(&"a".into()), Some(&1));
        assert_eq!(repo.insert("a".into(), 2), Some(1));
        assert_eq!(repo.len(), 1);
        assert_eq!(repo.remove(&"a".into()), Some(2));
        assert!(repo.is_empty());
    }

    #[test]
    fn json_file_repository_persists() {
        let dir = std::env::temp_dir().join("tpt_db_test");
        let path = dir.join("repo.json");
        let _ = fs::remove_file(&path);
        {
            let mut repo: JsonFileRepository<String, u32> =
                JsonFileRepository::open(&path).unwrap();
            repo.insert("k".into(), 99);
        }
        // Reopen and confirm durability.
        let repo: JsonFileRepository<String, u32> = JsonFileRepository::open(&path).unwrap();
        assert_eq!(repo.get(&"k".into()), Some(&99));
        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn migrations_run_once_in_order() {
        use std::cell::RefCell;
        use std::rc::Rc;

        #[derive(Clone)]
        struct SharedTarget {
            log: Rc<RefCell<Vec<String>>>,
        }
        impl MigrationTarget for SharedTarget {
            fn execute(&mut self, s: &str) -> Result<(), DbError> {
                self.log.borrow_mut().push(s.to_string());
                Ok(())
            }
        }

        let log = Rc::new(RefCell::new(Vec::new()));
        let target = SharedTarget { log: log.clone() };
        let mut migrator = Migrator::new(Box::new(target));
        migrator
            .register(Migration::new(
                1,
                "init",
                "create tables",
                Box::new(|t| t.execute("CREATE TABLE equipment")),
            ))
            .unwrap();
        migrator
            .register(Migration::new(
                2,
                "seed",
                "seed rows",
                Box::new(|t| t.execute("INSERT INTO equipment DEFAULT")),
            ))
            .unwrap();

        let applied = migrator.run_pending().unwrap();
        assert_eq!(applied, vec![1, 2]);

        // Second run applies nothing new.
        let again = migrator.run_pending().unwrap();
        assert!(again.is_empty());

        assert_eq!(
            *log.borrow(),
            vec![
                "CREATE TABLE equipment".to_string(),
                "INSERT INTO equipment DEFAULT".to_string()
            ]
        );
        assert_eq!(migrator.applied_versions(), &[1, 2]);
    }

    #[test]
    fn duplicate_migration_rejected() {
        let mut reg = MigrationRegistry::new();
        reg.register(Migration::new(1, "a", "", Box::new(|_| Ok(()))))
            .unwrap();
        assert!(reg
            .register(Migration::new(1, "b", "", Box::new(|_| Ok(()))))
            .is_err());
    }
}
