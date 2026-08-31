// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Minimal ASCII DXF (Drawing eXchange Format) reader for 2D/3D entities.
//!
//! Parses the group-code/value structure and extracts common entities
//! (LINE, LWPOLYLINE, CIRCLE, POINT, TEXT). Geometry can be lifted into
//! [`tpt_c_geometry`] primitives for takeoff and clash work.

use thiserror::Error;

/// Errors raised while parsing DXF.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DxfError {
    /// The document had no ENTITIES section.
    #[error("missing ENTITIES section")]
    NoEntities,
    /// A group code line was not a valid integer.
    #[error("invalid group code")]
    BadGroupCode,
    /// Unexpected end of input.
    #[error("unexpected end of input")]
    UnexpectedEof,
}

type Result<T> = std::result::Result<T, DxfError>;

/// A parsed DXF entity.
#[derive(Clone, Debug, PartialEq)]
pub enum DxfEntity {
    /// A 3D line.
    Line {
        /// Start point.
        start: [f64; 3],
        /// End point.
        end: [f64; 3],
    },
    /// A lightweight polyline (planar).
    LwPolyline {
        /// Vertices (x, y).
        points: Vec<[f64; 2]>,
        /// Whether the polyline is closed.
        closed: bool,
    },
    /// A circle in the XY plane.
    Circle {
        /// Center (x, y).
        center: [f64; 2],
        /// Radius.
        radius: f64,
    },
    /// A point.
    Point {
        /// Location.
        p: [f64; 3],
    },
    /// A text entity.
    Text {
        /// Insertion point.
        p: [f64; 3],
        /// Text content.
        text: String,
    },
    /// Any other entity, kept by name with its raw group values.
    Other {
        /// Entity type name.
        name: String,
    },
}

/// A parsed DXF document.
#[derive(Clone, Debug, Default)]
pub struct DxfDocument {
    /// Entities found in the ENTITIES section.
    pub entities: Vec<DxfEntity>,
}

impl DxfDocument {
    /// Parse a DXF document from text.
    pub fn parse(input: &str) -> Result<Self> {
        let mut entities = Vec::new();
        let mut in_entities = false;
        let mut tokens = input.lines().map(str::trim).peekable();

        while let Some(code_line) = tokens.next() {
            let code: i32 = code_line.parse().map_err(|_| DxfError::BadGroupCode)?;
            let value = tokens.next().ok_or(DxfError::UnexpectedEof)?.to_string();
            if code == 0 && value.eq_ignore_ascii_case("SECTION") {
                // Look ahead for the section name (group 2).
                if let (Some(&c2), Some(v2)) = (tokens.peek().copied(), tokens.peek().map(|_| tokens.next().unwrap().to_string())) {
                    if c2 == 2 && v2.eq_ignore_ascii_case("ENTITIES") {
                        let _ = tokens.next(); // consume the group-2 line (already taken)
                        in_entities = true;
                    }
                }
                continue;
            }
            if in_entities && code == 0 {
                if value.eq_ignore_ascii_case("ENDSEC") {
                    in_entities = false;
                    continue;
                }
                // Parse an entity: read its following groups until the next 0 group.
                let mut props: Vec<(i32, String)> = Vec::new();
                while let Some(&c) = tokens.peek() {
                    if c == 0 {
                        break;
                    }
                    let c = tokens.next().unwrap().parse().map_err(|_| DxfError::BadGroupCode)?;
                    let v = tokens.next().ok_or(DxfError::UnexpectedEof)?.to_string();
                    props.push((c, v));
                }
                if let Some(e) = build_entity(&value, &props) {
                    entities.push(e);
                }
            }
        }

        if entities.is_empty() && !in_entities {
            // Allow documents that only list entities without a section marker.
            return Err(DxfError::NoEntities);
        }
        Ok(Self { entities })
    }

    /// Total entity count.
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Whether the document has no entities.
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Collect line-like segments (lines + polyline edges + circle approximations).
    pub fn segments(&self) -> Vec<([f64; 3], [f64; 3])> {
        let mut out = Vec::new();
        for e in &self.entities {
            match e {
                DxfEntity::Line { start, end } => out.push((*start, *end)),
                DxfEntity::LwPolyline { points, closed } => {
                    for w in points.windows(2) {
                        out.push(([w[0][0], w[0][1], 0.0], [w[1][0], w[1][1], 0.0]));
                    }
                    if *closed && points.len() > 1 {
                        let a = points.first().unwrap();
                        let b = points.last().unwrap();
                        out.push(([a[0], a[1], 0.0], [b[0], b[1], 0.0]));
                    }
                }
                DxfEntity::Circle { center, radius } => {
                    let n = 24;
                    let c = *center;
                    let r = *radius;
                    for i in 0..n {
                        let a0 = 2.0 * std::f64::consts::PI * (i as f64) / (n as f64);
                        let a1 = 2.0 * std::f64::consts::PI * ((i + 1) as f64) / (n as f64);
                        out.push((
                            [c[0] + r * a0.cos(), c[1] + r * a0.sin(), 0.0],
                            [c[0] + r * a1.cos(), c[1] + r * a1.sin(), 0.0],
                        ));
                    }
                }
                _ => {}
            }
        }
        out
    }
}

fn get_f(props: &[(i32, String)], code: i32) -> Option<f64> {
    props
        .iter()
        .find(|(c, _)| *c == code)
        .and_then(|(_, v)| v.parse::<f64>().ok())
}

fn build_entity(name: &str, props: &[(i32, String)]) -> Option<DxfEntity> {
    match name.to_ascii_uppercase().as_str() {
        "LINE" => Some(DxfEntity::Line {
            start: [
                get_f(props, 10)?,
                get_f(props, 20)?,
                get_f(props, 30).unwrap_or(0.0),
            ],
            end: [
                get_f(props, 11)?,
                get_f(props, 21)?,
                get_f(props, 31).unwrap_or(0.0),
            ],
        }),
        "LWPOLYLINE" => {
            let count = get_f(props, 90)? as usize;
            let mut points = Vec::new();
            let mut closed = false;
            let mut i = 0usize;
            let mut piter = props.iter().peekable();
            while let Some(&(c, _)) = piter.peek() {
                if *c == 0 {
                    break;
                }
                let (c, v) = piter.next().unwrap();
                if c == 10 {
                    let x = v.parse::<f64>().ok()?;
                    // consume following 20 (y)
                    if let Some(&(20, ref yv)) = piter.peek() {
                        let _ = piter.next();
                        let y = yv.parse::<f64>().ok()?;
                        points.push([x, y]);
                    } else {
                        points.push([x, 0.0]);
                    }
                    i += 1;
                } else if c == 70 {
                    closed = v.parse::<i32>().map(|v| v & 1 == 1).unwrap_or(false);
                } else {
                    let _ = v;
                }
            }
            if points.len() != count {
                // Be lenient: keep what we parsed.
            }
            Some(DxfEntity::LwPolyline { points, closed })
        }
        "CIRCLE" => Some(DxfEntity::Circle {
            center: [get_f(props, 10)?, get_f(props, 20)?],
            radius: get_f(props, 40)?,
        }),
        "POINT" => Some(DxfEntity::Point {
            p: [
                get_f(props, 10)?,
                get_f(props, 20)?,
                get_f(props, 30).unwrap_or(0.0),
            ],
        }),
        "TEXT" => Some(DxfEntity::Text {
            p: [
                get_f(props, 10).unwrap_or(0.0),
                get_f(props, 20).unwrap_or(0.0),
                get_f(props, 30).unwrap_or(0.0),
            ],
            text: props
                .iter()
                .find(|(c, _)| *c == 1)
                .map(|(_, v)| v.clone())
                .unwrap_or_default(),
        }),
        other => Some(DxfEntity::Other {
            name: other.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"0
SECTION
2
ENTITIES
0
LINE
8
0
10
0.0
20
0.0
30
0.0
11
10.0
21
0.0
31
0.0
0
CIRCLE
8
0
10
5.0
20
5.0
40
2.0
0
ENDSEC
0
EOF
"#;

    #[test]
    fn parse_entities() {
        let doc = DxfDocument::parse(SAMPLE).unwrap();
        assert_eq!(doc.len(), 2);
        let segs = doc.segments();
        // 1 line + 24 circle segments.
        assert_eq!(segs.len(), 25);
        assert_eq!(doc.entities[0], DxfEntity::Line { start: [0.0,0.0,0.0], end: [10.0,0.0,0.0] });
    }
}
