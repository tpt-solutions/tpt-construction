// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Core domain primitives for TPT Construction.
//!
//! This crate provides the shared vocabulary used across every other `tpt-c-*`
//! crate: strongly-typed identifiers, error types, audit metadata, and the
//! common traits that domain aggregates implement.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Generate a strongly-typed UUID-backed identifier newtype.
///
/// Each identifier de/serializes to its inner UUID (transparently), orders
/// deterministically, and renders as the canonical UUID string.
macro_rules! define_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            /// Create an identifier from an explicit UUID (e.g. imported).
            pub fn from_uuid(id: Uuid) -> Self {
                Self(id)
            }

            /// The nil (all-zeros) identifier, useful as a sentinel.
            pub fn nil() -> Self {
                Self(Uuid::nil())
            }

            /// Return the inner UUID.
            pub fn as_uuid(&self) -> &Uuid {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl FromStr for $name {
            type Err = $crate::CoreError;
            fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
                Uuid::parse_str(s)
                    .map(Self)
                    .map_err(|e| $crate::CoreError::InvalidId {
                        ty: stringify!($name),
                        detail: e.to_string(),
                    })
            }
        }
    };
}

define_id!(
    /// Identifies a [`Project`](crate::ProjectContext) aggregate root.
    ProjectId
);
define_id!(
    /// Identifies a contract aggregate.
    ContractId
);
define_id!(
    /// Identifies a model (a versioned BIM/coordination model).
    ModelId
);
define_id!(
    /// Identifies an element within a model.
    ElementId
);
define_id!(
    /// Identifies a cost estimate aggregate.
    EstimateId
);
define_id!(
    /// Identifies a schedule aggregate.
    ScheduleId
);
define_id!(
    /// Identifies an activity within a schedule.
    ActivityId
);
define_id!(
    /// Identifies a Request For Information.
    RFIId
);
define_id!(
    /// Identifies a submittal.
    SubmittalId
);
define_id!(
    /// Identifies a change order.
    ChangeOrderId
);
define_id!(
    /// Identifies a physical asset (FM / equipment).
    AssetId
);

/// Errors shared across the construction domain crates.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CoreError {
    /// A string could not be parsed into the expected identifier type.
    #[error("invalid {ty} identifier: {detail}")]
    InvalidId {
        /// The identifier type name.
        ty: &'static str,
        /// The underlying parse failure message.
        detail: String,
    },

    /// A required field or relationship was missing.
    #[error("missing required {0}")]
    MissingRequired(&'static str),

    /// A domain invariant was violated.
    #[error("invariant violated: {0}")]
    InvariantViolated(&'static str),

    /// A referenced entity does not exist in the aggregate.
    #[error("not found: {0}")]
    NotFound(&'static str),

    /// A numeric value was outside the supported range.
    #[error("value out of range: {0}")]
    OutOfRange(&'static str),

    /// An operation is not permitted in the current state.
    #[error("illegal state: {0}")]
    IllegalState(&'static str),
}

/// Convenience `Result` alias for core operations.
pub type Result<T> = std::result::Result<T, CoreError>;

/// Who made a change and when, captured for every audited mutation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditMeta {
    /// Actor responsible for the change (user id, service name, ...).
    pub actor: String,
    /// RFC 3339 timestamp of the change.
    pub at: String,
    /// Optional human-readable note.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl AuditMeta {
    /// Build an audit record from an actor and an RFC 3339 timestamp.
    pub fn new(actor: impl Into<String>, at: impl Into<String>) -> Self {
        Self {
            actor: actor.into(),
            at: at.into(),
            note: None,
        }
    }

    /// Attach a human-readable note.
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }
}

/// A lightweight reference to the owning project, carried by most entities.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectRef(pub ProjectId);

/// Common capability for aggregate roots that belong to a project.
pub trait ProjectScoped {
    /// Return the project this entity belongs to.
    fn project_id(&self) -> ProjectId;
}

/// Common capability for entities that expose a stable identifier.
pub trait Identified {
    /// The concrete identifier type for this entity.
    type Id;
    /// Return this entity's identifier.
    fn id(&self) -> Self::Id;
}

/// Context describing the project under which domain aggregates operate.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectContext {
    /// The project identifier.
    pub id: ProjectId,
    /// Human-readable project name.
    pub name: String,
    /// Optional owning organization.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,
    /// ISO 4217 currency code used for monetary values in this project.
    #[serde(default = "default_currency")]
    pub currency: String,
}

fn default_currency() -> String {
    "USD".to_string()
}

impl ProjectContext {
    /// Create a new project context.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: ProjectId::nil(),
            name: name.into(),
            organization: None,
            currency: default_currency(),
        }
    }

    /// Assign a generated project id, returning the updated context.
    pub fn with_id(mut self, id: ProjectId) -> Self {
        self.id = id;
        self
    }

    /// Assign an owning organization.
    pub fn with_organization(mut self, org: impl Into<String>) -> Self {
        self.organization = Some(org.into());
        self
    }

    /// Set the project currency.
    pub fn with_currency(mut self, currency: impl Into<String>) -> Self {
        self.currency = currency.into();
        self
    }
}

impl ProjectScoped for ProjectContext {
    fn project_id(&self) -> ProjectId {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_roundtrip_string() {
        let id = ProjectId::nil();
        assert_eq!(id.to_string(), "00000000-0000-0000-0000-000000000000");
        let parsed: ProjectId = "00000000-0000-0000-0000-000000000000".parse().unwrap();
        assert_eq!(parsed, id);
    }

    #[test]
    fn id_rejects_garbage() {
        assert!("not-a-uuid".parse::<ElementId>().is_err());
    }

    #[test]
    fn audit_meta_builder() {
        let a = AuditMeta::new("alice", "2026-01-01T00:00:00Z").with_note("created");
        assert_eq!(a.actor, "alice");
        assert_eq!(a.note.as_deref(), Some("created"));
    }

    #[test]
    fn project_context_defaults() {
        let p = ProjectContext::new("Demo").with_currency("EUR");
        assert_eq!(p.currency, "EUR");
        assert!(p.organization.is_none());
    }
}
