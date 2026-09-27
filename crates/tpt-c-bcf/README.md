# tpt-c-bcf

BIM Collaboration Format (BCF) domain model and XML (de)serialization. BCF encodes
coordination issues as small XML documents (`markup.bcf`, `viewpoint.bcf`, ...). This
crate models the parts needed for issue tracking — topics (issues), comments,
viewpoints, lifecycle statuses, priorities, and assignments — and reads/writes
BCF 2.x-style markup XML with a small, dependency-free parser, so it works offline
and without external XML crates.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `BcfMarkup`: a complete markup document — one `BcfTopic` plus its `BcfComment`s and `BcfViewpoint`s
- `BcfMarkup::from_xml` / `BcfMarkup::to_xml`: round-trippable BCF-style XML with entity escaping
- `TopicStatus` lifecycle (`Open`, `InProgress`, `Resolved`, `Closed`) plus a passthrough `Other(String)`
- `TopicPriority` (`Low`, `Medium`, `High`, or any custom string)
- Topic metadata: type, creator, creation date, assignee, due date, free-text description
- Comment edit tracking via `modified_date` / `modified_author`
- Serde `Serialize`/`Deserialize` on every model type
- `BcfError` reporting malformed XML with byte offsets, or the missing element/attribute by name

## Usage

```toml
[dependencies]
tpt-c-bcf = "0.1"
```

```rust
use tpt_c_bcf::{BcfMarkup, TopicPriority, TopicStatus};

const MARKUP: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Markup>
  <Topic Guid="topic-1" TopicType="Clash" TopicStatus="in_progress">
    <Title>Wall clashes with beam</Title>
    <Priority>high</Priority>
    <AssignedTo>bob</AssignedTo>
    <Description>Two elements overlap.</Description>
  </Topic>
  <Comment Guid="c1" Author="bob" Date="2026-01-02T00:00:00Z">
    <Comment>Confirmed clash.</Comment>
  </Comment>
  <Viewpoints>
    <Viewpoint Guid="vp1"><Description>Clash zoomed view</Description></Viewpoint>
  </Viewpoints>
</Markup>"#;

// Parse a markup document (element and attribute names are case-insensitive).
let markup = BcfMarkup::from_xml(MARKUP).unwrap();
assert_eq!(markup.topic.title, "Wall clashes with beam");
assert_eq!(markup.topic.status, TopicStatus::InProgress);
assert_eq!(markup.topic.priority, Some(TopicPriority::High));
assert_eq!(markup.comments[0].text, "Confirmed clash.");
assert_eq!(markup.viewpoints[0].guid, "vp1");

// Serialize back to XML; the result reparses to an equal value.
let xml = markup.to_xml();
assert_eq!(BcfMarkup::from_xml(&xml).unwrap(), markup);
```

## Crate relationships

- **Depends on:** `serde`, `thiserror`
- **Used by:** Not yet consumed by other workspace crates; see the examples/
  directory for integration usage.

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) /
[LICENSE-APACHE](../../LICENSE-APACHE)).
