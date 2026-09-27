# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `AuthToken` bearer token with `new`, `with_expiry` and `with_scopes`
  builders over subject, timestamps and scopes.
- `Permission` enum (`Read`, `Write`, `Admin`, `Billing`, `Finance`) and
  `ProjectPermission` project access control entry.
- `Webhook` subscription with `new` and `with_secret` builders, event
  filters, HMAC secret and `active` flag.
- `AuditLogEntry` with `new`, `with_project`, `with_target` and
  `with_payload` builders.
- Serde `Serialize`/`Deserialize` support on all public types.
