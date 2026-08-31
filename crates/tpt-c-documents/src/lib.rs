// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Document control for construction projects.
//!
//! Covers the documents themselves ([`Document`], with a [`Revision`] history),
//! the kinds of documents teams exchange (drawings, specifications, ...), and
//! the transmittal process by which they are formally issued — a
//! [`TransmittalWorkflow`](tpt_c_workflow::TransmittalWorkflow) drives each
//! [`DocumentTransmittal`]. A [`DocumentRegister`] is the in-memory document
//! controller that keeps revisions and transmittals consistent.

use thiserror::Error;
use tpt_c_core::CoreError;

mod document;
mod register;
mod transmittal;

pub use document::{Document, DocumentStatus, DocumentType, Revision};
pub use register::DocumentRegister;
pub use transmittal::DocumentTransmittal;

/// Errors raised by document control operations.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DocumentError {
    /// The referenced document or revision was not found.
    #[error("not found: {0}")]
    NotFound(&'static str),
    /// A revision number was reused.
    #[error("duplicate revision: {0}")]
    DuplicateRevision(i32),
    /// A core domain error occurred.
    #[error(transparent)]
    Core(#[from] CoreError),
}

/// Identifier for a transmittal (re-exported convenience).
pub use tpt_c_ids::TransmittalId;

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_c_core::AuditMeta;
    use tpt_c_ids::DocumentId;
    use uuid::Uuid;

    #[test]
    fn document_revision_history() {
        let mut doc = Document::new(
            DocumentId::from_uuid(Uuid::now_v7()),
            "Plan A",
            DocumentType::Drawing,
        );
        assert_eq!(doc.status, DocumentStatus::Draft);
        doc.add_revision("initial issue".to_string());
        doc.add_revision("clash resolution".to_string());
        assert_eq!(doc.revisions.len(), 2);
        assert_eq!(doc.current_revision_no(), 2);
        doc.supersede();
        assert_eq!(doc.status, DocumentStatus::Superseded);
    }

    #[test]
    fn transmittal_drives_state() {
        let mut reg = DocumentRegister::new();
        let doc = reg
            .add_document("Spec 1", DocumentType::Specification)
            .expect("doc");
        let mut tx = DocumentTransmittal::new("TX-1".to_string(), vec![doc.id]);
        tx.workflow
            .send(AuditMeta::new("pm", "2026-01-01T00:00:00Z"))
            .unwrap();
        assert_eq!(tx.workflow.state(), tpt_c_workflow::TransmittalState::Sent);
    }
}
