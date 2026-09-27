// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Minimal ASCII DXF (Drawing eXchange Format) reader for 2D/3D entities.
//!
//! Parses the group-code/value structure and extracts common entities
//! (LINE, LWPOLYLINE, CIRCLE, POINT, TEXT). Extracted geometry is plain
//! coordinate arrays, ready to be lifted into mesh primitives for takeoff
//! and clash work.

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
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DxfDocument {
    /// Entities found in the ENTITIES section.
    pub entities: Vec<DxfEntity>,
}

impl DxfDocument {
    /// Parse a DXF document from text.
    pub fn parse(input: &str) -> Result<Self> {
        let pairs = tokenize(input)?;
        let mut entities = Vec::new();
        let mut in_entities = false;
        let mut seen_entities_section = false;
        let mut i = 0;
        while i < pairs.len() {
            let (code, value) = &pairs[i];
            if *code == 0 && value.eq_ignore_ascii_case("SECTION") {
                // The section name follows as a group-2 pair.
                if let Some((2, name)) = pairs.get(i + 1) {
                    if name.eq_ignore_ascii_case("ENTITIES") {
                        in_entities = true;
                        seen_entities_section = true;
                        i += 2;
                        continue;
                    }
                }
                i += 1;
                continue;
            }
            if in_entities && *code == 0 {
                if value.eq_ignore_ascii_case("ENDSEC") {
                    in_entities = false;
                    i += 1;
                    continue;
                }
                // Gather the entity's groups up to the next group-0 pair.
                let mut j = i + 1;
                while j < pairs.len() && pairs[j].0 != 0 {
                    j += 1;
                }
                if let Some(e) = build_entity(value, &pairs[i + 1..j]) {
                    entities.push(e);
                }
                i = j;
                continue;
            }
            i += 1;
        }

        if !seen_entities_section {
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

/// Split DXF text into `(group code, value)` pairs, one per code/value line.
fn tokenize(input: &str) -> Result<Vec<(i32, String)>> {
    let mut lines = input.lines().map(str::trim).filter(|l| !l.is_empty());
    let mut pairs = Vec::new();
    while let Some(code_line) = lines.next() {
        let code: i32 = code_line.parse().map_err(|_| DxfError::BadGroupCode)?;
        let value = lines.next().ok_or(DxfError::UnexpectedEof)?.to_string();
        pairs.push((code, value));
    }
    Ok(pairs)
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
            let mut points = Vec::new();
            let mut closed = false;
            let mut idx = 0;
            while idx < props.len() {
                let (c, v) = &props[idx];
                match *c {
                    10 => {
                        let x = v.parse::<f64>().unwrap_or(0.0);
                        // The matching y follows as a group-20 pair.
                        let y = match props.get(idx + 1) {
                            Some((20, yv)) => {
                                idx += 1;
                                yv.parse::<f64>().unwrap_or(0.0)
                            }
                            _ => 0.0,
                        };
                        points.push([x, y]);
                    }
                    70 => closed = v.parse::<i32>().map(|f| f & 1 == 1).unwrap_or(false),
                    _ => {}
                }
                idx += 1;
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
        assert_eq!(
            doc.entities[0],
            DxfEntity::Line {
                start: [0.0, 0.0, 0.0],
                end: [10.0, 0.0, 0.0]
            }
        );
    }

    #[test]
    fn parse_polyline_text_and_point() {
        let src = "0\nSECTION\n2\nENTITIES\n0\nLWPOLYLINE\n90\n3\n70\n1\n10\n0.0\n20\n0.0\n\
                   10\n4.0\n20\n0.0\n10\n4.0\n20\n3.0\n0\nTEXT\n1\nGrid A\n10\n1.0\n20\n2.0\n\
                   0\nPOINT\n10\n7.0\n20\n8.0\n30\n1.5\n0\nENDSEC\n0\nEOF\n";
        let doc = DxfDocument::parse(src).unwrap();
        assert_eq!(doc.len(), 3);
        assert_eq!(
            doc.entities[0],
            DxfEntity::LwPolyline {
                points: vec![[0.0, 0.0], [4.0, 0.0], [4.0, 3.0]],
                closed: true,
            }
        );
        match &doc.entities[1] {
            DxfEntity::Text { p, text } => {
                assert_eq!(*p, [1.0, 2.0, 0.0]);
                assert_eq!(text, "Grid A");
            }
            other => panic!("expected text, got {other:?}"),
        }
        // Closed 3-vertex polyline yields 3 segments (2 edges + closing edge).
        let segs = doc.segments();
        assert_eq!(segs.len(), 3);
    }

    #[test]
    fn missing_entities_section_is_an_error() {
        let src = "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n0\nENDSEC\n0\nEOF\n";
        assert_eq!(DxfDocument::parse(src), Err(DxfError::NoEntities));
    }

    #[test]
    fn odd_number_of_lines_is_eof() {
        assert_eq!(DxfDocument::parse("0\n"), Err(DxfError::UnexpectedEof));
        assert_eq!(DxfDocument::parse("zzz\n1\n"), Err(DxfError::BadGroupCode));
    }
}
