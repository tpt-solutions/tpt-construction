# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `Document` aggregate with `DocumentType`, `DocumentStatus` (`Draft`, `Current`,
  `Issued`, `Superseded`), sequential `Revision` records (`add_revision`,
  `current_revision_no`, `supersede`), and optional `Revision::file_ref`.
- `DocumentTransmittal` binding an external transmittal number to transmitted
  `DocumentId`s and a `TransmittalWorkflow`; `document_count()` helper.
- `DocumentRegister` in-memory controller: `add_document`, `get`, `add_revision`,
  `issue_transmittal`, `current_count`, and `transmittal` lookup by external id.
- `DocumentError` (`NotFound`, `DuplicateRevision`, core error passthrough) and
  re-export of `tpt_c_ids::TransmittalId`.
