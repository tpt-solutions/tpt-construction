// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export neutral construction models to glTF 2.0 for web/mesh consumption.
//!
//! The neutral [`tpt_c_model::Project`] carries no triangle soup, so this
//! exporter builds a glTF scene where each element becomes a node referencing a
//! shared unit-cube mesh, scaled to the element's dimensions (read from its
//! property sets when present) and annotated with its metadata in `extras`.
//! Output is standards-compliant glTF 2.0 JSON suitable for `<model-viewer>`,
//! three.js, and other web viewers.

use std::collections::HashMap;

use serde::Serialize;
use serde_json::{json, Value};
use tpt_c_model::{Element, Project};

/// glTF component type: 5126 = FLOAT.
const CT_FLOAT: u32 = 5126;
/// glTF component type: 5123 = UNSIGNED_SHORT.
const CT_USHORT: u32 = 5123;
/// glTF buffer view target: ARRAY_BUFFER (vertices).
const TARGET_ARRAY: u32 = 34962;
/// glTF buffer view target: ELEMENT_ARRAY_BUFFER (indices).
const TARGET_ELEMENT: u32 = 34963;

/// A glTF 2.0 document.
#[derive(Debug, Clone, Serialize)]
pub struct GltfDocument {
    /// Asset header.
    pub asset: Asset,
    /// Index of the default scene.
    pub scene: usize,
    /// Scene definitions.
    pub scenes: Vec<Scene>,
    /// All nodes (one per element, plus the scene root).
    pub nodes: Vec<Node>,
    /// Shared cube mesh.
    pub meshes: Vec<Mesh>,
    /// Geometry accessors.
    pub accessors: Vec<Accessor>,
    /// Buffer views into the single buffer.
    pub buffer_views: Vec<BufferView>,
    /// The single geometry buffer (base64 data URI).
    pub buffers: Vec<Buffer>,
}

/// glTF asset metadata.
#[derive(Debug, Clone, Serialize)]
pub struct Asset {
    /// glTF version.
    pub version: String,
    /// Generator name.
    pub generator: String,
}

/// A glTF scene.
#[derive(Debug, Clone, Serialize)]
pub struct Scene {
    /// Root node indices.
    pub nodes: Vec<usize>,
}

/// A glTF node (one per model element).
#[derive(Debug, Clone, Serialize)]
pub struct Node {
    /// Node name (element name).
    pub name: String,
    /// Referenced mesh.
    pub mesh: usize,
    /// Per-axis scale from element dimensions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<[f64; 3]>,
    /// World translation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation: Option<[f64; 3]>,
    /// Element metadata mirrored from the model.
    pub extras: Value,
}

/// A glTF mesh.
#[derive(Debug, Clone, Serialize)]
pub struct Mesh {
    /// Mesh primitives.
    pub primitives: Vec<Primitive>,
}

/// A glTF primitive.
#[derive(Debug, Clone, Serialize)]
pub struct Primitive {
    /// Vertex attribute accessors by semantic.
    pub attributes: HashMap<String, usize>,
    /// Index accessor.
    pub indices: usize,
    /// Primitive metadata.
    pub extras: Value,
}

/// A glTF accessor.
#[derive(Debug, Clone, Serialize)]
pub struct Accessor {
    /// Owning buffer view.
    pub buffer_view: usize,
    /// Component type.
    pub component_type: u32,
    /// Element count.
    pub count: usize,
    /// Attribute type (`"VEC3"` / `"SCALAR"`).
    #[serde(rename = "type")]
    pub ty: String,
    /// Minimum values (POSITION only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<Vec<f64>>,
    /// Maximum values (POSITION only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<Vec<f64>>,
}

/// A glTF buffer view.
#[derive(Debug, Clone, Serialize)]
pub struct BufferView {
    /// Owning buffer.
    pub buffer: usize,
    /// Byte offset into the buffer.
    pub byte_offset: usize,
    /// Byte length.
    pub byte_length: usize,
    /// WebGL target.
    pub target: u32,
}

/// A glTF buffer.
#[derive(Debug, Clone, Serialize)]
pub struct Buffer {
    /// Total byte length.
    pub byte_length: usize,
    /// Embedded data URI.
    pub uri: String,
}

/// Exports a [`Project`] to a glTF scene.
pub fn export(project: &Project) -> GltfDocument {
    // Unit cube: 8 vertices, 12 triangles (36 indices).
    let positions: [f32; 24] = [
        0.0, 0.0, 0.0, // 0
        1.0, 0.0, 0.0, // 1
        1.0, 1.0, 0.0, // 2
        0.0, 1.0, 0.0, // 3
        0.0, 0.0, 1.0, // 4
        1.0, 0.0, 1.0, // 5
        1.0, 1.0, 1.0, // 6
        0.0, 1.0, 1.0, // 7
    ];
    let indices: [u16; 36] = [
        0, 1, 2, 0, 2, 3, // bottom
        4, 6, 5, 4, 7, 6, // top
        0, 4, 5, 0, 5, 1, // front
        1, 5, 6, 1, 6, 2, // right
        2, 6, 7, 2, 7, 3, // back
        3, 7, 4, 3, 4, 0, // left
    ];

    let mut bytes: Vec<u8> = Vec::new();
    for p in positions.iter() {
        bytes.extend_from_slice(&p.to_le_bytes());
    }
    let pos_len = bytes.len();
    for i in indices.iter() {
        bytes.extend_from_slice(&i.to_le_bytes());
    }

    let buffer_uri = format!("data:application/octet-stream;base64,{}", base64_encode(&bytes));

    let mut nodes = Vec::new();
    for (idx, element) in project.elements.iter().enumerate() {
        nodes.push(element_node(element, idx));
    }
    let root_index = nodes.len();
    nodes.push(Node {
        name: project.name.clone(),
        mesh: 0,
        scale: None,
        translation: None,
        extras: json!({ "type": "Project", "id": project.id.to_string() }),
    });

    GltfDocument {
        asset: Asset {
            version: "2.0".to_string(),
            generator: "tpt-c-gltf".to_string(),
        },
        scene: 0,
        scenes: vec![Scene {
            nodes: vec![root_index],
        }],
        nodes,
        meshes: vec![Mesh {
            primitives: vec![Primitive {
                attributes: {
                    let mut m = HashMap::new();
                    m.insert("POSITION".to_string(), 0);
                    m
                },
                indices: 1,
                extras: json!({ "kind": "unit-cube" }),
            }],
        }],
        accessors: vec![
            Accessor {
                buffer_view: 0,
                component_type: CT_FLOAT,
                count: 8,
                ty: "VEC3".to_string(),
                min: Some(vec![0.0, 0.0, 0.0]),
                max: Some(vec![1.0, 1.0, 1.0]),
            },
            Accessor {
                buffer_view: 1,
                component_type: CT_USHORT,
                count: indices.len(),
                ty: "SCALAR".to_string(),
                min: None,
                max: None,
            },
        ],
        buffer_views: vec![
            BufferView {
                buffer: 0,
                byte_offset: 0,
                byte_length: pos_len,
                target: TARGET_ARRAY,
            },
            BufferView {
                buffer: 0,
                byte_offset: pos_len,
                byte_length: bytes.len() - pos_len,
                target: TARGET_ELEMENT,
            },
        ],
        buffers: vec![Buffer {
            byte_length: bytes.len(),
            uri: buffer_uri,
        }],
    }
}

fn element_node(element: &Element, mesh: usize) -> Node {
    let scale = element_dimensions(element);
    let mut extras = json!({
        "category": element.category,
    });
    if let Some(s) = &element.storey_id {
        extras["storey"] = json!(s);
    }
    if let Some(ext) = &element.external_id {
        extras["external_id"] = json!(ext.value());
    }
    if let Some(c) = &element.classification {
        extras["classification"] = json!({ "system": format!("{:?}", c.system), "code": c.code });
    }
    Node {
        name: element.name.clone(),
        mesh,
        scale: Some(scale),
        translation: None,
        extras,
    }
}

/// Read element dimensions (Length/Width/Height) from its property sets,
/// defaulting to a unit cube when absent.
fn element_dimensions(element: &Element) -> [f64; 3] {
    let mut dims = [1.0_f64, 1.0, 1.0];
    let set = |slot: &mut f64, name: &str| {
        if let Some(v) = find_number(element, name) {
            *slot = v.abs().max(1e-3);
        }
    };
    set(&mut dims[0], "Length");
    set(&mut dims[1], "Height");
    set(&mut dims[2], "Width");
    dims
}

fn find_number(element: &Element, name: &str) -> Option<f64> {
    for ps in &element.property_sets {
        for p in &ps.properties {
            if p.name.eq_ignore_ascii_case(name) {
                if let tpt_c_model::PropertyValue::Number(v) = p.value {
                    return Some(v);
                }
            }
        }
    }
    None
}

/// Encode bytes as standard base64 (no external crate needed).
fn base64_encode(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in input.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | (b[2] as u32);
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((n >> 6) & 63) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(n & 63) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

impl GltfDocument {
    /// Serialize the document to glTF 2.0 JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("glTF serialization")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_c_core::ProjectId;
    use tpt_c_ids::IdFactory;
    use tpt_c_model::{PropertySet, PropertyValue, Quantity, QuantitySet};

    #[test]
    fn export_produces_valid_json() {
        let mut p = Project::new(ProjectId::nil(), "Demo");
        let e = tpt_c_model::Element::new(IdFactory::element(), "W1", "Wall")
            .with_property_set(
                PropertySet::new("Dims")
                    .with("Length", PropertyValue::Number(10.0))
                    .with("Height", PropertyValue::Number(3.0))
                    .with("Width", PropertyValue::Number(0.3)),
            )
            .with_quantity_set(
                QuantitySet::new("Q").with("Length", Quantity::Length(tpt_c_units::Length::from_meters(10.0))),
            );
        p.add_element(e);

        let doc = export(&p);
        let json = doc.to_json();
        let value: Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(value["asset"]["version"], "2.0");
        // One element node + one project root node.
        assert_eq!(value["nodes"].as_array().unwrap().len(), 2);
        let wall = &value["nodes"][0];
        assert_eq!(wall["name"], "W1");
        assert_eq!(wall["scale"][0], 10.0);
        assert_eq!(wall["scale"][1], 3.0);
        assert_eq!(wall["scale"][2], 0.3);
        assert_eq!(wall["extras"]["category"], "Wall");
    }

    #[test]
    fn base64_smoke() {
        // "Man" -> TWFu
        assert_eq!(base64_encode(b"Man"), "TWFu");
        assert_eq!(base64_encode(b"Ma"), "TWE=");
        assert_eq!(base64_encode(b"M"), "TQ==");
    }
}
