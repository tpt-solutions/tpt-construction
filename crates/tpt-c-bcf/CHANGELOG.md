# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `BcfTopic`, `BcfComment`, `BcfViewpoint`, and `BcfMarkup` domain types, all
  Serde-serializable
- `BcfMarkup::from_xml` — parse BCF 2.x-style markup XML using a built-in,
  dependency-free element-tree parser (case-insensitive element/attribute names)
- `BcfMarkup::to_xml` — serialize markup back to XML with text/attribute escaping
- `TopicStatus` and `TopicPriority` enums with passthrough `Other(String)` variants
- `BcfError` with `Xml` (byte offset + detail) and `Missing` (element/attribute) cases
- Round-trip guarantee: `from_xml(to_xml(m)) == m`
