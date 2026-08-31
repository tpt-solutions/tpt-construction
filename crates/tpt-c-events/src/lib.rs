// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Domain events, append-only event sourcing, projections and replay.
//!
//! Every meaningful mutation in the TPT domain is captured as a [`DomainEvent`]
//! wrapped in a [`StoredEvent`] envelope that records a monotonic sequence
//! number, a unique event id, the actor and an RFC 3339 timestamp. An
//! [`EventStore`] is an append-only, content-addressed log that doubles as the
//! system of record and the audit trail. [`Projection`]s derive read models by
//! folding over the log, and can be re-derived at any time via [`replay`].

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tpt_c_core::{ElementId, EstimateId, ModelId, RFIId, ScheduleId, SubmittalId};
use tpt_c_ids::IdFactory;
use uuid::Uuid;

/// A domain event capturing a meaningful state change.
///
/// Variants are intentionally narrow and carry only the identifiers and scalars
/// needed to reconstruct the change, keeping the events cheap to store and
/// replay.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DomainEvent {
    /// A BIM/coordination model was imported.
    ModelImported {
        /// The imported model.
        model_id: ModelId,
        /// Number of elements brought in.
        element_count: u64,
    },
    /// A quantity takeoff value was adjusted for an element.
    QuantityAdjusted {
        /// Affected element.
        element_id: ElementId,
        /// Name of the quantity that changed (e.g. `GrossVolume`).
        quantity_name: String,
        /// Signed delta applied to the quantity.
        delta: f64,
    },
    /// A cost item was applied to an estimate.
    CostItemApplied {
        /// Target estimate.
        estimate_id: EstimateId,
        /// Identifying name of the cost item.
        cost_item: String,
        /// Monetary amount applied.
        amount: f64,
    },
    /// An estimate was approved.
    EstimateApproved {
        /// Approved estimate.
        estimate_id: EstimateId,
        /// Approving actor.
        by: String,
    },
    /// A schedule was updated.
    ScheduleUpdated {
        /// Updated schedule.
        schedule_id: ScheduleId,
    },
    /// A Request For Information was raised.
    RFICreated {
        /// The new RFI.
        rfi_id: RFIId,
        /// Subject line.
        subject: String,
    },
    /// An RFI was answered.
    RFIAnswered {
        /// Answered RFI.
        rfi_id: RFIId,
        /// The answer text.
        answer: String,
    },
    /// A submittal was approved.
    SubmittalApproved {
        /// Approved submittal.
        submittal_id: SubmittalId,
        /// Approving actor.
        by: String,
    },
    /// A payment application was submitted.
    PaymentApplicationSubmitted {
        /// Application identifier (e.g. `PA-2026-014`).
        application_id: String,
        /// Certified amount submitted.
        amount: f64,
    },
}

impl DomainEvent {
    /// A stable, human-readable kind name for the event (used in reporting).
    pub fn kind(&self) -> &'static str {
        match self {
            DomainEvent::ModelImported { .. } => "model_imported",
            DomainEvent::QuantityAdjusted { .. } => "quantity_adjusted",
            DomainEvent::CostItemApplied { .. } => "cost_item_applied",
            DomainEvent::EstimateApproved { .. } => "estimate_approved",
            DomainEvent::ScheduleUpdated { .. } => "schedule_updated",
            DomainEvent::RFICreated { .. } => "rfi_created",
            DomainEvent::RFIAnswered { .. } => "rfi_answered",
            DomainEvent::SubmittalApproved { .. } => "submittal_approved",
            DomainEvent::PaymentApplicationSubmitted { .. } => "payment_application_submitted",
        }
    }
}

/// A domain event wrapped with the metadata required for sourcing and audit.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StoredEvent<E> {
    /// Monotonic sequence number within the store (1-based).
    pub seq: u64,
    /// Globally unique event id.
    pub event_id: Uuid,
    /// RFC 3339 timestamp of occurrence.
    pub occurred_at: String,
    /// Actor responsible for the event.
    pub actor: String,
    /// The domain event payload.
    pub event: E,
}

/// An append-only, content-addressed event log.
///
/// Events are never removed; the log is the system of record. Read models are
/// derived through [`Projection`]s rather than mutated in place.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EventStore<E> {
    events: Vec<StoredEvent<E>>,
}

impl<E> EventStore<E> {
    /// Create an empty store.
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Append an event, returning its assigned sequence number.
    pub fn append(&mut self, event: E, actor: impl Into<String>, at: impl Into<String>) -> u64 {
        let seq = (self.events.len() as u64) + 1;
        self.events.push(StoredEvent {
            seq,
            event_id: IdFactory::uuid7(),
            occurred_at: at.into(),
            actor: actor.into(),
            event,
        });
        seq
    }

    /// Number of events stored.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Whether the store has no events.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Current version (number of events) of the log.
    pub fn version(&self) -> u64 {
        self.events.len() as u64
    }

    /// Fetch a stored event by its sequence number (1-based).
    pub fn get(&self, seq: u64) -> Option<&StoredEvent<E>> {
        self.events.get((seq.saturating_sub(1)) as usize)
    }

    /// All stored events in order.
    pub fn all(&self) -> &[StoredEvent<E>] {
        &self.events
    }

    /// Iterate events starting at `offset` (0-based), for incremental replay.
    pub fn iter_from(&self, offset: usize) -> impl Iterator<Item = &StoredEvent<E>> {
        self.events.iter().skip(offset)
    }

    /// Serialize the entire log to JSON (useful for persistence).
    pub fn to_json(&self) -> Result<String, serde_json::Error>
    where
        E: Serialize,
    {
        serde_json::to_string(self)
    }
}

impl<E: Serialize> EventStore<E> {
    /// Deserialize a log previously produced by [`EventStore::to_json`].
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error>
    where
        E: for<'de> Deserialize<'de>,
    {
        serde_json::from_str(json)
    }
}

/// A read-model derived by folding over an event log.
///
/// Implementors define a [`State`](Projection::State) and a [`fold`] step. The
/// log never mutates; only the derived state does, so projections can be rebuilt
/// from scratch at any time.
///
/// [`fold`]: Projection::fold
pub trait Projection<E> {
    /// The derived read-model type.
    type State;

    /// Fold one stored event into the running state.
    fn fold(&self, state: &mut Self::State, event: &StoredEvent<E>);
}

/// Rebuild a projection's state by replaying the whole log.
pub fn replay<E, P: Projection<E>>(store: &EventStore<E>, proj: &P, initial: P::State) -> P::State {
    let mut state = initial;
    for ev in store.all() {
        proj.fold(&mut state, ev);
    }
    state
}

/// Rebuild a projection's state replaying only events from `offset` (0-based),
/// starting from an existing `state`. Useful for incremental refresh.
pub fn replay_from<E, P: Projection<E>>(
    store: &EventStore<E>,
    proj: &P,
    offset: usize,
    mut state: P::State,
) -> P::State {
    for ev in store.iter_from(offset) {
        proj.fold(&mut state, ev);
    }
    state
}

/// Convenience alias: the canonical TPT domain event log / audit trail.
pub type AuditTrail = EventStore<DomainEvent>;

/// Counts occurrences of each domain event kind.
pub struct EventTypeCounter;

impl Projection<DomainEvent> for EventTypeCounter {
    type State = HashMap<&'static str, u64>;

    fn fold(&self, state: &mut Self::State, event: &StoredEvent<DomainEvent>) {
        *state.entry(event.event.kind()).or_insert(0) += 1;
    }
}

/// A single line in an audit timeline.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuditLine {
    /// Monotonic sequence number.
    pub seq: u64,
    /// RFC 3339 timestamp.
    pub at: String,
    /// Acting principal.
    pub actor: String,
    /// Event kind.
    pub kind: &'static str,
}

/// Builds a chronological audit timeline of every event.
pub struct AuditTimeline;

impl Projection<DomainEvent> for AuditTimeline {
    type State = Vec<AuditLine>;

    fn fold(&self, state: &mut Self::State, event: &StoredEvent<DomainEvent>) {
        state.push(AuditLine {
            seq: event.seq,
            at: event.occurred_at.clone(),
            actor: event.actor.clone(),
            kind: event.event.kind(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_c_ids::IdFactory;

    fn sample_store() -> AuditTrail {
        let mut s: AuditTrail = EventStore::new();
        let model = ModelId::from_uuid(IdFactory::deterministic("m1"));
        let est = IdFactory::estimate();
        let rfi = RFIId::from_uuid(IdFactory::deterministic("rfi1"));
        s.append(
            DomainEvent::ModelImported {
                model_id: model,
                element_count: 42,
            },
            "importer",
            "2026-01-01T00:00:00Z",
        );
        s.append(
            DomainEvent::EstimateApproved {
                estimate_id: est,
                by: "chief-estimator".into(),
            },
            "chief-estimator",
            "2026-01-02T00:00:00Z",
        );
        s.append(
            DomainEvent::RFICreated {
                rfi_id: rfi,
                subject: "Rebar lap splice".into(),
            },
            "field-eng",
            "2026-01-03T00:00:00Z",
        );
        s
    }

    #[test]
    fn append_assigns_monotonic_seq() {
        let s = sample_store();
        assert_eq!(s.len(), 3);
        assert_eq!(s.version(), 3);
        assert_eq!(s.get(1).unwrap().seq, 1);
        assert_eq!(s.get(3).unwrap().seq, 3);
        assert!(s.get(4).is_none());
    }

    #[test]
    fn replay_counts_event_types() {
        let s = sample_store();
        let counts = replay(&s, &EventTypeCounter, HashMap::new());
        assert_eq!(counts.get("model_imported"), Some(&1));
        assert_eq!(counts.get("estimate_approved"), Some(&1));
        assert_eq!(counts.get("rfi_created"), Some(&1));
    }

    #[test]
    fn replay_builds_audit_timeline() {
        let s = sample_store();
        let timeline = replay(&s, &AuditTimeline, Vec::new());
        assert_eq!(timeline.len(), 3);
        assert_eq!(timeline[0].actor, "importer");
        assert_eq!(timeline[2].kind, "rfi_created");
    }

    #[test]
    fn incremental_replay_from_offset() {
        let s = sample_store();
        let full = replay(&s, &EventTypeCounter, HashMap::new());
        // Replay only the last two events on top of the first event's count.
        let mut partial: HashMap<&'static str, u64> = HashMap::new();
        partial.insert("model_imported", 1);
        let partial = replay_from(&s, &EventTypeCounter, 1, partial);
        assert_eq!(partial, full);
    }

    #[test]
    fn json_roundtrip() {
        let s = sample_store();
        let json = s.to_json().unwrap();
        let back = AuditTrail::from_json(&json).unwrap();
        assert_eq!(s, back);
    }
}
