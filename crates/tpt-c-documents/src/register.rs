// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! In-memory document register / controller.

use serde::{Deserialize, Serialize};
use tpt_c_ids::DocumentId;
use uuid::Uuid;

use crate::document::{Document, DocumentStatus, DocumentType};
use crate::{DocumentError, DocumentTransmittal};

/// A controller holding documents and transmittals for a project.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DocumentRegister {
    /// Controlled documents.
    #[serde(default)]
    pub documents: Vec<Document>,
    /// Transmittals issued.
    #[serde(default)]
    pub transmittals: Vec<DocumentTransmittal>,
}

impl DocumentRegister {
    /// Create an empty register.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a document, generating its identifier.
    pub fn add_document(
        &mut self,
        title: impl Into<String>,
        doc_type: DocumentType,
    ) -> Result<Document, DocumentError> {
        let doc = Document::new(DocumentId::from_uuid(Uuid::now_v7()), title, doc_type);
        self.documents.push(doc.clone());
        Ok(doc)
    }

    /// Fetch a document by id.
    pub fn get(&self, id: DocumentId) -> Result<&Document, DocumentError> {
        self.documents
            .iter()
            .find(|d| d.id == id)
            .ok_or(DocumentError::NotFound("document"))
    }

    /// Add a revision to a document already in the register.
    pub fn add_revision(
        &mut self,
        id: DocumentId,
        note: impl Into<String>,
    ) -> Result<i32, DocumentError> {
        let doc = self
            .documents
            .iter_mut()
            .find(|d| d.id == id)
            .ok_or(DocumentError::NotFound("document"))?;
        Ok(doc.add_revision(note))
    }

    /// Record a transmittal of documents.
    pub fn issue_transmittal(
        &mut self,
        external_id: impl Into<String>,
        document_ids: Vec<DocumentId>,
    ) -> DocumentTransmittal {
        let tx = DocumentTransmittal::new(external_id, document_ids);
        self.transmittals.push(tx.clone());
        tx
    }

    /// Count of documents currently issued for construction.
    pub fn current_count(&self) -> usize {
        self.documents
            .iter()
            .filter(|d| d.status == DocumentStatus::Current)
            .count()
    }

    /// Look up a transmittal by its external id.
    pub fn transmittal(&self, id: &str) -> Option<&DocumentTransmittal> {
        self.transmittals.iter().find(|t| t.id == id)
    }
}
