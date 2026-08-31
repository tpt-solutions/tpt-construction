// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Deterministic and standard identifier generation plus external-ID mapping.
//!
//! Internal domain entities are addressed by the UUID-backed identifiers from
//! [`tpt_c_core`]. Interoperating with authoring tools requires mapping those
//! internal ids to external identifiers such as IFC GUIDs, Revit element ids,
//! cost codes, and asset tags. This crate provides that bridge.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Core domain identifiers re-exported for convenience.
pub use tpt_c_core::{
    AssetId, ChangeOrderId, ClaimId, ContractId, ContractItemId, DocumentId, ElementId, EstimateId,
    IssueId, NoticeId, PaymentApplicationId, ProjectId, PunchListId, RFIId, SafetyIncidentId,
    SubmittalId, TransmittalId,
};

/// Fixed namespace used for deterministic (UUIDv5) identifier derivation.
///
/// Distinct from the DNS/URL/OID/IETF namespaces defined by RFC 4122 so that
/// TPT-derived identifiers never collide with vendor-generated ones.
pub const TPT_NAMESPACE: Uuid = Uuid::from_u128(0x9f1b_2c3d_4e5f_6071_8293_a4b5_c6d7_e8f9);

/// Generates identifiers. Wraps UUIDv7 (time-ordered, random) and a stable
/// UUIDv5 derivation for reproducible builds and tests.
#[derive(Clone, Copy, Debug, Default)]
pub struct IdFactory;

impl IdFactory {
    /// Generate a fresh time-ordered UUIDv7.
    pub fn uuid7() -> Uuid {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        Uuid::new_v7(uuid::Timestamp::from_unix(
            uuid::NoContext,
            now.as_secs(),
            now.subsec_nanos(),
        ))
    }

    /// Deterministically derive a UUIDv5 from `name` within the TPT namespace.
    ///
    /// Two calls with the same `name` always yield the same UUID, which makes
    /// import pipelines reproducible and lets fixtures be authored by hand.
    pub fn deterministic(name: &str) -> Uuid {
        Uuid::new_v5(&TPT_NAMESPACE, name.as_bytes())
    }

    /// Generate a fresh [`ElementId`] backed by UUIDv7.
    pub fn element() -> ElementId {
        ElementId::from_uuid(Self::uuid7())
    }

    /// Deterministically derive an [`ElementId`] from `name`.
    pub fn deterministic_element(name: &str) -> ElementId {
        ElementId::from_uuid(Self::deterministic(name))
    }

    /// Generate a fresh [`AssetId`] backed by UUIDv7.
    pub fn asset() -> AssetId {
        AssetId::from_uuid(Self::uuid7())
    }

    /// Generate a fresh [`EstimateId`] backed by UUIDv7.
    pub fn estimate() -> EstimateId {
        EstimateId::from_uuid(Self::uuid7())
    }
}

/// An external identifier assigned to an entity by a source system.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ExternalId {
    /// Industry Foundation Classes globally unique identifier (22-char base64).
    IfcGuid(String),
    /// Revit element / type id (numeric or `Guid`-style).
    RevitId(String),
    /// Cost code (e.g. MasterFormat `03 30 00`).
    CostCode(String),
    /// Asset tag printed on equipment / FM assets.
    AssetTag(String),
    /// Any other source-specific identifier.
    Other(String),
}

impl ExternalId {
    /// Return the underlying string value.
    pub fn value(&self) -> &str {
        match self {
            ExternalId::IfcGuid(v)
            | ExternalId::RevitId(v)
            | ExternalId::CostCode(v)
            | ExternalId::AssetTag(v)
            | ExternalId::Other(v) => v,
        }
    }
}

/// Bi-directional mapping between an internal entity id and external ids.
///
/// A single internal id may carry several external aliases (IFC GUID *and*
/// Revit id, for example); each external id maps back to exactly one internal
/// id.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound(deserialize = "I: Deserialize<'de> + Eq + std::hash::Hash + Clone"))]
pub struct ExternalIdMap<I> {
    internal_to_external: HashMap<I, Vec<ExternalId>>,
    external_to_internal: HashMap<ExternalId, I>,
}

impl<I: Clone + std::hash::Hash + Eq> Default for ExternalIdMap<I> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I: Clone + std::hash::Hash + Eq> ExternalIdMap<I> {
    /// Create an empty mapping.
    pub fn new() -> Self {
        Self {
            internal_to_external: HashMap::new(),
            external_to_internal: HashMap::new(),
        }
    }

    /// Associate `external` with `internal`, returning the previous external id
    /// if the same external id was already mapped to a different internal id.
    pub fn insert(
        &mut self,
        internal: I,
        external: ExternalId,
    ) -> Result<(), ExternalIdConflict<I>> {
        if let Some(prev) = self.external_to_internal.get(&external) {
            if prev != &internal {
                return Err(ExternalIdConflict {
                    external,
                    existing: prev.clone(),
                });
            }
        }
        self.external_to_internal
            .insert(external.clone(), internal.clone());
        self.internal_to_external
            .entry(internal)
            .or_default()
            .push(external);
        Ok(())
    }

    /// Look up all external ids for an internal id.
    pub fn externals_for(&self, internal: &I) -> &[ExternalId] {
        self.internal_to_external
            .get(internal)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Resolve an external id to its internal id.
    pub fn resolve(&self, external: &ExternalId) -> Option<&I> {
        self.external_to_internal.get(external)
    }
}

/// Error returned when an external id collides with a different internal id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternalIdConflict<I> {
    /// The conflicting external id.
    pub external: ExternalId,
    /// The internal id that already owns it.
    pub existing: I,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid7_is_unique_and_time_ordered() {
        let a = IdFactory::uuid7();
        let b = IdFactory::uuid7();
        assert_ne!(a, b);
    }

    #[test]
    fn deterministic_is_reproducible() {
        assert_eq!(
            IdFactory::deterministic("wall-1"),
            IdFactory::deterministic("wall-1")
        );
        assert_ne!(
            IdFactory::deterministic("wall-1"),
            IdFactory::deterministic("wall-2")
        );
    }

    #[test]
    fn external_id_map_bidirectional() {
        let mut map: ExternalIdMap<ElementId> = ExternalIdMap::new();
        let eid = IdFactory::deterministic_element("w1");
        map.insert(eid, ExternalId::IfcGuid("3O0MLfx3BDgvhRaydMkJh6".into()))
            .unwrap();
        map.insert(eid, ExternalId::RevitId("123456".into()))
            .unwrap();

        assert_eq!(map.externals_for(&eid).len(), 2);
        let guid = ExternalId::IfcGuid("3O0MLfx3BDgvhRaydMkJh6".into());
        assert_eq!(map.resolve(&guid), Some(&eid));
    }

    #[test]
    fn external_id_conflict_detected() {
        let mut map: ExternalIdMap<ElementId> = ExternalIdMap::new();
        let a = IdFactory::deterministic_element("a");
        let b = IdFactory::deterministic_element("b");
        let guid = ExternalId::IfcGuid("SAME".into());
        map.insert(a, guid.clone()).unwrap();
        assert!(map.insert(b, guid).is_err());
    }
}
