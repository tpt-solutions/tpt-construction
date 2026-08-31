// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! REST/GraphQL API models, auth patterns, authorization, project access control,
//! webhooks, and audit logging.
//!
//! Provides the transport-agnostic types used by API layers: request/response
//! envelopes, auth tokens, project permissions, webhook payloads, and audit
//! log entries.

use serde::{Deserialize, Serialize};
use tpt_c_core::ProjectId;

/// A bearer token used for API authentication.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuthToken {
    /// Token string.
    pub token: String,
    /// Subject / user id.
    pub subject: String,
    /// Issued at (ISO 8601).
    pub issued_at: String,
    /// Expires at (ISO 8601), if applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    /// Scopes granted to this token.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scopes: Vec<String>,
}

impl AuthToken {
    /// Build an auth token.
    pub fn new(
        token: impl Into<String>,
        subject: impl Into<String>,
        issued_at: impl Into<String>,
    ) -> Self {
        Self {
            token: token.into(),
            subject: subject.into(),
            issued_at: issued_at.into(),
            expires_at: None,
            scopes: Vec::new(),
        }
    }

    /// Set expiry.
    pub fn with_expiry(mut self, expires_at: impl Into<String>) -> Self {
        self.expires_at = Some(expires_at.into());
        self
    }

    /// Grant scopes.
    pub fn with_scopes(mut self, scopes: Vec<String>) -> Self {
        self.scopes = scopes;
        self
    }
}

/// A permission scope.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    /// Read project data.
    Read,
    /// Write project data.
    Write,
    /// Administer project settings and members.
    Admin,
    /// Manage billing and payment applications.
    Billing,
    /// View cost and financial data.
    Finance,
}

/// Project-level access control entry.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectPermission {
    /// Who the permission applies to (user id or service name).
    pub subject: String,
    /// The project.
    pub project_id: ProjectId,
    /// Granted permissions.
    pub permissions: Vec<Permission>,
}

impl ProjectPermission {
    /// Build a project permission entry.
    pub fn new(
        subject: impl Into<String>,
        project_id: ProjectId,
        permissions: Vec<Permission>,
    ) -> Self {
        Self {
            subject: subject.into(),
            project_id,
            permissions,
        }
    }
}

/// A webhook subscription.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Webhook {
    /// Webhook id.
    pub id: String,
    /// Target URL.
    pub url: String,
    /// Events to deliver.
    pub events: Vec<String>,
    /// Secret used for HMAC signing, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    /// Whether the webhook is active.
    #[serde(default)]
    pub active: bool,
}

impl Webhook {
    /// Build a webhook subscription.
    pub fn new(id: impl Into<String>, url: impl Into<String>, events: Vec<String>) -> Self {
        Self {
            id: id.into(),
            url: url.into(),
            events,
            secret: None,
            active: true,
        }
    }

    /// Set the signing secret.
    pub fn with_secret(mut self, secret: impl Into<String>) -> Self {
        self.secret = Some(secret.into());
        self
    }
}

/// An audit log entry.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// ISO 8601 timestamp.
    pub timestamp: String,
    /// Who performed the action.
    pub actor: String,
    /// Action name (e.g. `estimate.approved`).
    pub action: String,
    /// Project id, if applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<ProjectId>,
    /// Target entity type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_type: Option<String>,
    /// Target entity id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_id: Option<String>,
    /// Optional JSON payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

impl AuditLogEntry {
    /// Build an audit log entry.
    pub fn new(
        timestamp: impl Into<String>,
        actor: impl Into<String>,
        action: impl Into<String>,
    ) -> Self {
        Self {
            timestamp: timestamp.into(),
            actor: actor.into(),
            action: action.into(),
            project_id: None,
            target_type: None,
            target_id: None,
            payload: None,
        }
    }

    /// Attach a project context.
    pub fn with_project(mut self, project_id: ProjectId) -> Self {
        self.project_id = Some(project_id);
        self
    }

    /// Attach a target entity.
    pub fn with_target(
        mut self,
        target_type: impl Into<String>,
        target_id: impl Into<String>,
    ) -> Self {
        self.target_type = Some(target_type.into());
        self.target_id = Some(target_id.into());
        self
    }

    /// Attach a JSON payload.
    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = Some(payload);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_c_ids::IdFactory;

    #[test]
    fn auth_token_builder() {
        let token = AuthToken::new("abc", "alice", "2026-01-01T00:00:00Z")
            .with_expiry("2026-12-31T23:59:59Z")
            .with_scopes(vec!["read".into(), "write".into()]);
        assert_eq!(token.subject, "alice");
        assert_eq!(token.scopes, vec!["read", "write"]);
    }

    #[test]
    fn webhook_builder() {
        let wh = Webhook::new(
            "wh-1",
            "https://example.com/hook",
            vec!["estimate.approved".into()],
        )
        .with_secret("shhh");
        assert!(wh.active);
        assert_eq!(wh.secret, Some("shhh".to_string()));
    }

    #[test]
    fn audit_log_entry() {
        let project_id = ProjectId::from_uuid(IdFactory::deterministic("p1"));
        let entry = AuditLogEntry::new("2026-01-01T00:00:00Z", "alice", "estimate.approved")
            .with_project(project_id)
            .with_target("Estimate", "est-1");
        assert_eq!(entry.action, "estimate.approved");
        assert!(entry.project_id.is_some());
    }
}
