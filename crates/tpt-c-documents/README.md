# tpt-c-documents

Document control for construction projects: controlled documents (drawings,
specifications, contract documents, reports) with sequential revision histories,
plus the transmittal process by which documents are formally issued. A
`DocumentRegister` acts as the in-memory document controller that keeps documents,
revisions, and transmittals consistent; each `DocumentTransmittal` is driven by a
`TransmittalWorkflow` from `tpt-c-workflow`.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `Document` with `DocumentType` (drawing, specification, contract document,
  report, other) and `DocumentStatus` lifecycle (draft, current, issued,
  superseded).
- Sequential `Revision` history via `Document::add_revision` (auto-numbered,
  1-based) and `current_revision_no()`; `supersede()` marks a document outdated.
- `DocumentTransmittal` couples an external transmittal number (e.g. "TX-0142")
  with the transmitted `DocumentId`s and a live `TransmittalWorkflow`
  (prepared -> sent -> received -> acknowledged -> closed).
- `DocumentRegister` mints document ids, tracks documents and transmittals, and
  offers `get`, `add_revision`, `issue_transmittal`, `current_count`, and
  `transmittal` lookup by external id.
- `DocumentError` for not-found and duplicate-revision situations.
- Serializable with serde; a pure core crate with no IO.

## Usage

```toml
[dependencies]
tpt-c-documents = "0.1"
```

```rust
use tpt_c_core::AuditMeta;
use tpt_c_documents::{DocumentRegister, DocumentType};

let mut reg = DocumentRegister::new();

// Add a controlled document and revise it.
let doc = reg.add_document("Spec 1", DocumentType::Specification).expect("doc");
reg.add_revision(doc.id, "initial issue").unwrap();
reg.add_revision(doc.id, "clash resolution").unwrap();
assert_eq!(reg.get(doc.id).unwrap().revisions.len(), 2);

// Issue a transmittal and drive its workflow.
let mut tx = reg.issue_transmittal("TX-1", vec![doc.id]);
tx.workflow
    .send(AuditMeta::new("pm", "2026-01-01T00:00:00Z"))
    .unwrap();
assert_eq!(tx.workflow.state(), tpt_c_workflow::TransmittalState::Sent);
assert!(reg.transmittal("TX-1").is_some());
```

## Crate relationships

- **Depends on:** `serde`, `thiserror`, `uuid`, `tpt-c-core`, `tpt-c-ids`
  (`DocumentId`, `TransmittalId`), and `tpt-c-workflow` (`TransmittalWorkflow`).
- **Used by:** Not yet consumed by other workspace crates;
  [examples/field-execution-e2e](../../examples/field-execution-e2e) exercises the
  register + transmittal flow end to end.

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) / [LICENSE-APACHE](../../LICENSE-APACHE)).
