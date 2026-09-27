# tpt-c-api

Transport-agnostic API-layer types for tpt-construction services: bearer auth
tokens, permission scopes, project-level access control, webhook subscriptions
and audit log entries. Reach for it when building REST/GraphQL surfaces over
the workspace engines and you need serializable request/response domain types
for authentication, authorization, webhooks and auditability — without tying
them to a specific web framework.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `AuthToken`: bearer token with subject, issued/expiry timestamps (ISO 8601)
  and scope strings; `with_expiry` and `with_scopes` builders.
- `Permission` enum: `Read`, `Write`, `Admin`, `Billing`, `Finance`
  (serialized as snake_case).
- `ProjectPermission`: subject-to-project access control entry listing
  granted permissions.
- `Webhook` subscription with target URL, event filter, HMAC signing secret
  (`with_secret`) and an `active` flag (default on).
- `AuditLogEntry`: timestamp, actor, action name plus optional project
  (`with_project`), target entity (`with_target`) and JSON payload
  (`with_payload`).
- Serde `Serialize`/`Deserialize` on all public types; payloads use
  `serde_json::Value`.

## Usage

```toml
[dependencies]
tpt-c-api = "0.1"
```

```rust
use tpt_c_api::{AuditLogEntry, AuthToken, Webhook};

let token = AuthToken::new("abc", "alice", "2026-01-01T00:00:00Z")
    .with_expiry("2026-12-31T23:59:59Z")
    .with_scopes(vec!["read".into(), "write".into()]);
assert_eq!(token.scopes, vec!["read", "write"]);

let webhook = Webhook::new(
    "wh-1",
    "https://example.com/hook",
    vec!["estimate.approved".into()],
)
.with_secret("shhh");
assert!(webhook.active);

let entry = AuditLogEntry::new("2026-01-01T00:00:00Z", "alice", "estimate.approved")
    .with_target("Estimate", "est-1");
assert_eq!(entry.action, "estimate.approved");
```

## Crate relationships

- **Depends on:** `serde`, `serde_json`, `tpt-c-core` (`ProjectId`), `tpt-c-ids` (test ids), `tpt-c-model`.
- **Used by:** `examples/tpt` CLI (optional `serve` feature; `examples/tpt/src/serve.rs` uses `AuditLogEntry`, `AuthToken` and `Permission`).

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) / [LICENSE-APACHE](../../LICENSE-APACHE)).
