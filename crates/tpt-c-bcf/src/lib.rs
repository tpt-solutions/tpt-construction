// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! BIM Collaboration Format (BCF) domain model and XML (de)serialization.
//!
//! BCF encodes coordination issues as small XML documents (`markup.bcf`,
//! `viewpoint.bcf`, ...). This crate models the parts needed for issue
//! tracking — topics (issues), comments, viewpoints, statuses, priorities, and
//! assignments — and reads/writes BCF 2.x-style markup XML with a small,
//! dependency-free parser so it works offline and without external XML crates.

use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

/// Errors raised while (de)serializing BCF markup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BcfError {
    /// The XML was malformed.
    Xml(usize, String),
    /// A required element or attribute was missing.
    Missing(&'static str),
}

impl fmt::Display for BcfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BcfError::Xml(o, s) => write!(f, "malformed BCF XML at offset {o}: {s}"),
            BcfError::Missing(s) => write!(f, "missing {s} in BCF markup"),
        }
    }
}

impl std::error::Error for BcfError {}

/// Lifecycle status of a BCF topic (issue).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TopicStatus {
    /// Newly created, unaddressed.
    Open,
    /// Being worked on.
    InProgress,
    /// Resolution proposed.
    Resolved,
    /// Closed.
    Closed,
    /// Any other status string.
    Other(String),
}

impl TopicStatus {
    fn as_str(&self) -> &str {
        match self {
            TopicStatus::Open => "open",
            TopicStatus::InProgress => "in_progress",
            TopicStatus::Resolved => "resolved",
            TopicStatus::Closed => "closed",
            TopicStatus::Other(s) => s.as_str(),
        }
    }
    fn from_str(s: &str) -> Self {
        match s {
            "open" => TopicStatus::Open,
            "in_progress" => TopicStatus::InProgress,
            "resolved" => TopicStatus::Resolved,
            "closed" => TopicStatus::Closed,
            other => TopicStatus::Other(other.to_string()),
        }
    }
}

/// Priority of a BCF topic.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TopicPriority {
    /// Low priority.
    Low,
    /// Medium priority.
    Medium,
    /// High priority.
    High,
    /// Any other priority string.
    Other(String),
}

impl TopicPriority {
    fn as_str(&self) -> &str {
        match self {
            TopicPriority::Low => "low",
            TopicPriority::Medium => "medium",
            TopicPriority::High => "high",
            TopicPriority::Other(s) => s.as_str(),
        }
    }
    fn from_str(s: &str) -> Self {
        match s {
            "low" => TopicPriority::Low,
            "medium" => TopicPriority::Medium,
            "high" => TopicPriority::High,
            other => TopicPriority::Other(other.to_string()),
        }
    }
}

/// A BCF topic — the central coordination issue.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BcfTopic {
    /// Globally unique identifier of the topic.
    pub guid: String,
    /// Short title.
    pub title: String,
    /// Optional topic classification (e.g. `Clash`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic_type: Option<String>,
    /// Lifecycle status.
    pub status: TopicStatus,
    /// Optional priority.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<TopicPriority>,
    /// Author who created the topic.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    /// RFC 3339 creation timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creation_date: Option<String>,
    /// Person/role the topic is assigned to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assigned_to: Option<String>,
    /// Optional due date.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    /// Free-text description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// A comment attached to a topic.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BcfComment {
    /// Comment identifier.
    pub guid: String,
    /// Author of the comment.
    pub author: String,
    /// RFC 3339 timestamp.
    pub date: String,
    /// Comment body.
    pub text: String,
    /// Last modification timestamp, if edited.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modified_date: Option<String>,
    /// Last modifier, if edited.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modified_author: Option<String>,
}

/// A viewpoint referenced by a topic (camera, snapshot, selection).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BcfViewpoint {
    /// Viewpoint identifier.
    pub guid: String,
    /// Optional descriptive note.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional base64 snapshot image reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<String>,
}

/// A complete BCF markup document: one topic plus its comments and viewpoints.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BcfMarkup {
    /// The topic (issue).
    pub topic: BcfTopic,
    /// Comments, in order.
    #[serde(default)]
    pub comments: Vec<BcfComment>,
    /// Viewpoints referenced by the topic.
    #[serde(default)]
    pub viewpoints: Vec<BcfViewpoint>,
}

impl BcfMarkup {
    /// Parse a BCF markup document from XML.
    pub fn from_xml(input: &str) -> Result<Self, BcfError> {
        let root = XmlParser::parse(input)?;
        if root.name.to_ascii_uppercase() != "MARKUP" {
            return Err(BcfError::Missing("Markup root"));
        }
        let topic_el = root
            .child("Topic")
            .ok_or(BcfError::Missing("Topic"))?;
        let guid = topic_el
            .attr("Guid")
            .ok_or(BcfError::Missing("Topic Guid"))?
            .to_string();
        let title = topic_el
            .child_text("Title")
            .ok_or(BcfError::Missing("Topic Title"))?
            .to_string();
        let status = TopicStatus::from_str(
            topic_el
                .attr("TopicStatus")
                .unwrap_or("open"),
        );

        let topic = BcfTopic {
            guid,
            title,
            topic_type: topic_el.attr("TopicType").map(str::to_string),
            status,
            priority: topic_el
                .attr("TopicPriority")
                .or_else(|| topic_el.child_text("Priority"))
                .map(|s| TopicPriority::from_str(s)),
            created_by: topic_el.child_text("CreatedBy").map(str::to_string),
            creation_date: topic_el.child_text("CreationDate").map(str::to_string),
            assigned_to: topic_el.child_text("AssignedTo").map(str::to_string),
            due_date: topic_el.child_text("DueDate").map(str::to_string),
            description: topic_el.child_text("Description").map(str::to_string),
        };

        let mut comments = Vec::new();
        for c in root.children("Comment") {
            let guid = c.attr("Guid").ok_or(BcfError::Missing("Comment Guid"))?.to_string();
            let author = c
                .attr("Author")
                .or_else(|| c.child_text("Author"))
                .ok_or(BcfError::Missing("Comment Author"))?
                .to_string();
            let date = c
                .attr("Date")
                .or_else(|| c.child_text("Date"))
                .ok_or(BcfError::Missing("Comment Date"))?
                .to_string();
            let text = c
                .child_text("Comment")
                .or_else(|| c.child_text("Text"))
                .ok_or(BcfError::Missing("Comment Text"))?
                .to_string();
            comments.push(BcfComment {
                guid,
                author,
                date,
                text,
                modified_date: c.attr("ModifiedDate").map(str::to_string),
                modified_author: c.attr("ModifiedAuthor").map(str::to_string),
            });
        }

        let mut viewpoints = Vec::new();
        if let Some(vps) = root.child("Viewpoints") {
            for vp in vps.children("Viewpoint") {
                let guid = vp.attr("Guid").ok_or(BcfError::Missing("Viewpoint Guid"))?.to_string();
                viewpoints.push(BcfViewpoint {
                    guid,
                    description: vp.child_text("Description").map(str::to_string),
                    snapshot: vp.child_text("Snapshot").map(str::to_string),
                });
            }
        }

        Ok(BcfMarkup {
            topic,
            comments,
            viewpoints,
        })
    }

    /// Serialize the markup to BCF-style XML.
    pub fn to_xml(&self) -> String {
        let mut out = String::new();
        out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        out.push_str("<Markup>\n");
        out.push_str("  <Topic");
        out.push_str(&attr("Guid", &self.topic.guid));
        if let Some(t) = &self.topic.topic_type {
            out.push_str(&attr("TopicType", t));
        }
        out.push_str(&attr("TopicStatus", self.topic.status.as_str()));
        out.push_str(">\n");
        out.push_str(&elem("Title", &self.topic.title));
        if let Some(p) = &self.topic.priority {
            out.push_str(&elem("Priority", p.as_str()));
        }
        if let Some(c) = &self.topic.created_by {
            out.push_str(&elem("CreatedBy", c));
        }
        if let Some(c) = &self.topic.creation_date {
            out.push_str(&elem("CreationDate", c));
        }
        if let Some(a) = &self.topic.assigned_to {
            out.push_str(&elem("AssignedTo", a));
        }
        if let Some(d) = &self.topic.due_date {
            out.push_str(&elem("DueDate", d));
        }
        if let Some(d) = &self.topic.description {
            out.push_str(&elem("Description", d));
        }
        out.push_str("  </Topic>\n");
        for c in &self.comments {
            out.push_str("  <Comment");
            out.push_str(&attr("Guid", &c.guid));
            out.push_str(&attr("Author", &c.author));
            out.push_str(&attr("Date", &c.date));
            if let Some(m) = &c.modified_date {
                out.push_str(&attr("ModifiedDate", m));
            }
            if let Some(m) = &c.modified_author {
                out.push_str(&attr("ModifiedAuthor", m));
            }
            out.push_str(">\n");
            out.push_str(&elem("Comment", &c.text));
            out.push_str("  </Comment>\n");
        }
        if !self.viewpoints.is_empty() {
            out.push_str("  <Viewpoints>\n");
            for v in &self.viewpoints {
                out.push_str("    <Viewpoint");
                out.push_str(&attr("Guid", &v.guid));
                out.push_str(">\n");
                if let Some(d) = &v.description {
                    out.push_str(&elem("Description", d));
                }
                if let Some(s) = &v.snapshot {
                    out.push_str(&elem("Snapshot", s));
                }
                out.push_str("    </Viewpoint>\n");
            }
            out.push_str("  </Viewpoints>\n");
        }
        out.push_str("</Markup>\n");
        out
    }
}

fn attr(name: &str, value: &str) -> String {
    format!(" {}=\"{}\"", name, escape_attr(value))
}

fn elem(name: &str, value: &str) -> String {
    format!("    <{}>{}</{}>\n", name, escape_text(value), name)
}

fn escape_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_attr(s: &str) -> String {
    escape_text(s).replace('"', "&quot;")
}

// ---------------------------------------------------------------------------
// Minimal XML parser (element tree).
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct XmlEl {
    name: String,
    attrs: HashMap<String, String>,
    children: Vec<XmlEl>,
    text: String,
}

impl XmlEl {
    fn attr(&self, name: &str) -> Option<&str> {
        self.attrs.get(name).map(String::as_str)
    }
    fn child(&self, name: &str) -> Option<&XmlEl> {
        self.children.iter().find(|c| c.name.eq_ignore_ascii_case(name))
    }
    fn children(&self, name: &str) -> Vec<&XmlEl> {
        self.children
            .iter()
            .filter(|c| c.name.eq_ignore_ascii_case(name))
            .collect()
    }
    fn child_text(&self, name: &str) -> Option<&str> {
        self.child(name).map(|c| c.text.as_str())
    }
}

struct XmlParser<'a> {
    s: &'a [char],
    pos: usize,
}

impl<'a> XmlParser<'a> {
    fn parse(input: &'a str) -> Result<XmlEl, BcfError> {
        let chars: Vec<char> = input.chars().collect();
        let mut p = XmlParser { s: &chars, pos: 0 };
        p.skip_prolog();
        p.parse_element()
    }

    fn skip_prolog(&mut self) {
        // Skip XML declaration and comments before the root element.
        loop {
            self.skip_ws();
            if self.peek(0) == Some('<') && self.peek(1) == Some('?') {
                self.skip_until("?>");
            } else if self.peek(0) == Some('<') && self.peek(1) == Some('!') {
                self.skip_until("-->");
            } else {
                break;
            }
        }
    }

    fn skip_ws(&mut self) {
        while self.pos < self.s.len() && self.s[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn peek(&self, ahead: usize) -> Option<char> {
        self.s.get(self.pos + ahead).copied()
    }

    fn skip_until(&mut self, needle: &str) {
        let n: Vec<char> = needle.chars().collect();
        while self.pos + n.len() <= self.s.len() {
            if self.s[self.pos..self.pos + n.len()].iter().eq(n.iter()) {
                self.pos += n.len();
                return;
            }
            self.pos += 1;
        }
        self.pos = self.s.len();
    }

    fn parse_element(&mut self) -> Result<XmlEl, BcfError> {
        self.skip_ws();
        if self.peek(0) != Some('<') {
            return Err(BcfError::Xml(self.pos, "expected '<'".into()));
        }
        self.pos += 1; // consume '<'
        let name = self.parse_name()?;
        let attrs = self.parse_attributes()?;
        self.skip_ws();
        // Self-closing?
        if self.peek(0) == Some('/') {
            self.pos += 1;
            if self.peek(0) != Some('>') {
                return Err(BcfError::Xml(self.pos, "expected '>'".into()));
            }
            self.pos += 1;
            return Ok(XmlEl {
                name,
                attrs,
                children: Vec::new(),
                text: String::new(),
            });
        }
        if self.peek(0) != Some('>') {
            return Err(BcfError::Xml(self.pos, "expected '>'".into()));
        }
        self.pos += 1; // consume '>'

        let mut children = Vec::new();
        let mut text = String::new();
        loop {
            self.skip_ws();
            if self.peek(0) == Some('<') && self.peek(1) == Some('/') {
                // closing tag
                self.pos += 2;
                let close = self.parse_name()?;
                if close != name {
                    return Err(BcfError::Xml(self.pos, format!("mismatched close tag {close}")));
                }
                self.skip_ws();
                if self.peek(0) != Some('>') {
                    return Err(BcfError::Xml(self.pos, "expected '>'".into()));
                }
                self.pos += 1;
                return Ok(XmlEl {
                    name,
                    attrs,
                    children,
                    text: text.trim().to_string(),
                });
            } else if self.peek(0) == Some('<') && self.peek(1) == Some('!') {
                self.skip_until("-->");
            } else if self.peek(0) == Some('<') {
                children.push(self.parse_element()?);
            } else {
                text.push_str(&self.parse_text()?);
            }
        }
    }

    fn parse_name(&mut self) -> Result<String, BcfError> {
        self.skip_ws();
        let start = self.pos;
        while self.pos < self.s.len() {
            let c = self.s[self.pos];
            if c.is_alphanumeric() || c == '_' || c == '-' || c == ':' {
                self.pos += 1;
            } else {
                break;
            }
        }
        if self.pos == start {
            return Err(BcfError::Xml(self.pos, "expected tag name".into()));
        }
        Ok(self.s[start..self.pos].iter().collect())
    }

    fn parse_attributes(&mut self) -> Result<HashMap<String, String>, BcfError> {
        let mut attrs = HashMap::new();
        loop {
            self.skip_ws();
            match self.peek(0) {
                Some('>') | Some('/') => break,
                _ => {}
            }
            let key = self.parse_name()?;
            self.skip_ws();
            if self.peek(0) != Some('=') {
                return Err(BcfError::Xml(self.pos, "expected '=' in attribute".into()));
            }
            self.pos += 1;
            self.skip_ws();
            let quote = match self.peek(0) {
                Some('"') | Some('\'') => self.peek(0).unwrap(),
                _ => return Err(BcfError::Xml(self.pos, "expected attribute quote".into())),
            };
            self.pos += 1;
            let start = self.pos;
            while self.pos < self.s.len() && self.peek(0) != Some(quote) {
                self.pos += 1;
            }
            let value: String = self.s[start..self.pos].iter().collect();
            self.pos += 1; // consume quote
            attrs.insert(key, unescape(&value));
        }
        Ok(attrs)
    }

    fn parse_text(&mut self) -> Result<String, BcfError> {
        let start = self.pos;
        while self.pos < self.s.len() && self.peek(0) != Some('<') {
            self.pos += 1;
        }
        Ok(unescape(&self.s[start..self.pos].iter().collect::<String>()))
    }
}

fn unescape(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    const MARKUP: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Markup>
  <Topic Guid="topic-1" TopicType="Clash" TopicStatus="in_progress">
    <Title>Wall clashes with beam</Title>
    <Priority>high</Priority>
    <CreatedBy>alice</CreatedBy>
    <CreationDate>2026-01-01T00:00:00Z</CreationDate>
    <AssignedTo>bob</AssignedTo>
    <Description>Two elements overlap.</Description>
  </Topic>
  <Comment Guid="c1" Author="bob" Date="2026-01-02T00:00:00Z">
    <Comment>Confirmed clash.</Comment>
  </Comment>
  <Viewpoints>
    <Viewpoint Guid="vp1">
      <Description>Clash zoomed view</Description>
    </Viewpoint>
  </Viewpoints>
</Markup>"#;

    #[test]
    fn parse_markup() {
        let m = BcfMarkup::from_xml(MARKUP).expect("parse");
        assert_eq!(m.topic.guid, "topic-1");
        assert_eq!(m.topic.title, "Wall clashes with beam");
        assert_eq!(m.topic.status, TopicStatus::InProgress);
        assert_eq!(m.topic.priority, Some(TopicPriority::High));
        assert_eq!(m.topic.assigned_to.as_deref(), Some("bob"));
        assert_eq!(m.comments.len(), 1);
        assert_eq!(m.comments[0].author, "bob");
        assert_eq!(m.comments[0].text, "Confirmed clash.");
        assert_eq!(m.viewpoints.len(), 1);
        assert_eq!(m.viewpoints[0].guid, "vp1");
    }

    #[test]
    fn round_trip() {
        let m = BcfMarkup::from_xml(MARKUP).unwrap();
        let xml = m.to_xml();
        let m2 = BcfMarkup::from_xml(&xml).unwrap();
        assert_eq!(m, m2);
    }

    #[test]
    fn escaping() {
        let input = "<Markup><Topic Guid=\"t\" TopicStatus=\"open\"><Title>A &amp; B &lt; C</Title></Topic></Markup>";
        let m = BcfMarkup::from_xml(input).unwrap();
        assert_eq!(m.topic.title, "A & B < C");
    }

    #[test]
    fn fixture_file() {
        let m = BcfMarkup::from_xml(include_str!("../../../test-data/bcf/markup.bcf")).expect("fixture");
        assert!(!m.topic.guid.is_empty());
    }
}
