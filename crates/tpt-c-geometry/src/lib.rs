// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Geometry primitives for construction models.
//!
//! This crate is deliberately self-contained: it does not depend on an external
//! linear-algebra library (the `tpt-math` substrate is optional in this
//! repository). It provides the primitive types shared by the model, IFC, glTF,
//! and earthwork crates: [`Point3`], [`Vec3`], [`Mesh`], [`Solid`], and
//! [`BoundingBox`], plus a lightweight [`SpatialIndex`] for broad-phase queries
//! and clash-detection helpers.

use serde::{Deserialize, Serialize};

use tpt_c_core::ElementId;
use tpt_c_units::{Area, Length, Volume};

/// A point in 3D space, stored in metres.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Point3 {
    /// East/local-X component (m).
    pub x: f64,
    /// North/local-Y component (m).
    pub y: f64,
    /// Elevation/local-Z component (m).
    pub z: f64,
}

impl Point3 {
    /// Build a point.
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// The origin (0, 0, 0).
    pub fn origin() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    /// Translate this point by a vector.
    pub fn translate(self, v: Vec3) -> Self {
        Self::new(self.x + v.x, self.y + v.y, self.z + v.z)
    }
}

impl std::ops::Sub for Point3 {
    type Output = Vec3;
    fn sub(self, rhs: Point3) -> Vec3 {
        Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

/// A free vector in 3D space.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Vec3 {
    /// X component.
    pub x: f64,
    /// Y component.
    pub y: f64,
    /// Z component.
    pub z: f64,
}

impl Vec3 {
    /// Build a vector.
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Zero vector.
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    /// Euclidean distance to another vector.
    pub fn distance(self, other: Vec3) -> f64 {
        (self - other).length()
    }

    /// Dot product.
    pub fn dot(self, other: Vec3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Cross product.
    pub fn cross(self, other: Vec3) -> Vec3 {
        Vec3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    /// Squared length (avoids a sqrt when only comparing magnitudes).
    pub fn length_sq(self) -> f64 {
        self.dot(self)
    }

    /// Euclidean length.
    pub fn length(self) -> f64 {
        self.length_sq().sqrt()
    }

    /// Unit vector in the same direction (zero vector maps to itself).
    pub fn normalize(self) -> Vec3 {
        let len = self.length();
        if len == 0.0 {
            self
        } else {
            Vec3::new(self.x / len, self.y / len, self.z / len)
        }
    }
}

impl std::ops::Add for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Vec3) -> Vec3 {
        Vec3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl std::ops::Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, rhs: Vec3) -> Vec3 {
        Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl std::ops::Mul<f64> for Vec3 {
    type Output = Vec3;
    fn mul(self, s: f64) -> Vec3 {
        Vec3::new(self.x * s, self.y * s, self.z * s)
    }
}

impl From<Point3> for Vec3 {
    fn from(p: Point3) -> Vec3 {
        Vec3::new(p.x, p.y, p.z)
    }
}

/// A triangle defined by three (ordered) vertices.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Triangle {
    /// First vertex.
    pub a: Point3,
    /// Second vertex.
    pub b: Point3,
    /// Third vertex.
    pub c: Point3,
}

impl Triangle {
    /// Build a triangle.
    pub fn new(a: Point3, b: Point3, c: Point3) -> Self {
        Self { a, b, c }
    }

    /// Unsigned area of the triangle.
    pub fn area(&self) -> Area {
        let v = (self.b - self.a).cross(self.c - self.a);
        Area::from_square_meters(0.5 * v.length())
    }

    /// Geometric surface normal (not necessarily unit length).
    pub fn normal(&self) -> Vec3 {
        (self.b - self.a).cross(self.c - self.a)
    }

    /// Unit normal vector.
    pub fn unit_normal(&self) -> Vec3 {
        self.normal().normalize()
    }

    /// Centroid of the triangle.
    pub fn centroid(&self) -> Point3 {
        Point3::new(
            (self.a.x + self.b.x + self.c.x) / 3.0,
            (self.a.y + self.b.y + self.c.y) / 3.0,
            (self.a.z + self.b.z + self.c.z) / 3.0,
        )
    }

    /// Signed volume contribution of this triangle to the origin (used to
    /// compute closed-mesh volume via the divergence theorem).
    fn signed_volume_to_origin(&self) -> f64 {
        (1.0 / 6.0)
            * (self.a.x * self.b.y * self.c.z
                - self.a.x * self.c.y * self.b.z
                - self.b.x * self.a.y * self.c.z
                + self.b.x * self.c.y * self.a.z
                + self.c.x * self.a.y * self.b.z
                - self.c.x * self.b.y * self.a.z)
    }
}

/// An axis-aligned bounding box.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    /// Minimum corner.
    pub min: Point3,
    /// Maximum corner.
    pub max: Point3,
}

impl BoundingBox {
    /// Build a box from its min and max corners.
    pub fn from_corners(min: Point3, max: Point3) -> Self {
        Self { min, max }
    }

    /// An empty box at infinity (union with any point yields that point).
    pub fn empty() -> Self {
        Self {
            min: Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY),
            max: Point3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
        }
    }

    /// A degenerate box containing a single point.
    pub fn from_point(p: Point3) -> Self {
        Self { min: p, max: p }
    }

    /// Grow the box to include `p`.
    pub fn expand_point(&mut self, p: Point3) {
        self.min.x = self.min.x.min(p.x);
        self.min.y = self.min.y.min(p.y);
        self.min.z = self.min.z.min(p.z);
        self.max.x = self.max.x.max(p.x);
        self.max.y = self.max.y.max(p.y);
        self.max.z = self.max.z.max(p.z);
    }

    /// Union of two boxes.
    pub fn union(&self, other: &BoundingBox) -> BoundingBox {
        let mut r = *self;
        r.expand_point(other.min);
        r.expand_point(other.max);
        r
    }

    /// True if the boxes overlap (touching counts as overlap).
    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    /// True if `p` lies inside the box (inclusive bounds).
    pub fn contains_point(&self, p: &Point3) -> bool {
        p.x >= self.min.x
            && p.x <= self.max.x
            && p.y >= self.min.y
            && p.y <= self.max.y
            && p.z >= self.min.z
            && p.z <= self.max.z
    }

    /// Center of the box.
    pub fn center(&self) -> Point3 {
        Point3::new(
            (self.min.x + self.max.x) / 2.0,
            (self.min.y + self.max.y) / 2.0,
            (self.min.z + self.max.z) / 2.0,
        )
    }

    /// Length, width, height as [`Length`] measures (m).
    pub fn extents(&self) -> (Length, Length, Length) {
        (
            Length::from_meters((self.max.x - self.min.x).max(0.0)),
            Length::from_meters((self.max.y - self.min.y).max(0.0)),
            Length::from_meters((self.max.z - self.min.z).max(0.0)),
        )
    }

    /// Enclosed volume.
    pub fn volume(&self) -> Volume {
        let (dx, dy, dz) = self.extents();
        Volume::from_cubic_meters(dx.meters() * dy.meters() * dz.meters())
    }

    /// Total surface area of the six faces.
    pub fn surface_area(&self) -> Area {
        let (dx, dy, dz) = self.extents();
        let (x, y, z) = (dx.meters(), dy.meters(), dz.meters());
        Area::from_square_meters(2.0 * (x * y + x * z + y * z))
    }
}

/// A triangulated mesh. Vertices are shared; triangles index into `vertices`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mesh {
    /// Shared vertex positions.
    pub vertices: Vec<Point3>,
    /// Triangles, each a triple of indices into `vertices`.
    pub triangles: Vec<[usize; 3]>,
    /// Optional owning element, for meshes extracted from a model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<ElementId>,
}

impl Mesh {
    /// Build a mesh from vertices and triangle indices.
    pub fn new(vertices: Vec<Point3>, triangles: Vec<[usize; 3]>) -> Self {
        Self {
            vertices,
            triangles,
            owner: None,
        }
    }

    /// Attach an owning element id.
    pub fn with_owner(mut self, owner: ElementId) -> Self {
        self.owner = Some(owner);
        self
    }

    /// Number of triangles.
    pub fn triangle_count(&self) -> usize {
        self.triangles.len()
    }

    /// Iterate the concrete triangles of the mesh.
    pub fn iter_triangles(&self) -> impl Iterator<Item = Triangle> + '_ {
        self.triangles.iter().map(move |t| Triangle {
            a: self.vertices[t[0]],
            b: self.vertices[t[1]],
            c: self.vertices[t[2]],
        })
    }

    /// Total surface area of all triangles.
    pub fn area(&self) -> Area {
        Area::from_square_meters(
            self.iter_triangles()
                .map(|t| t.area().square_meters())
                .sum(),
        )
    }

    /// Surface area of a single face (index).
    pub fn triangle_area(&self, index: usize) -> Option<Area> {
        let t = self.triangles.get(index)?;
        Some(
            Triangle::new(
                self.vertices[t[0]],
                self.vertices[t[1]],
                self.vertices[t[2]],
            )
            .area(),
        )
    }

    /// Signed volume of a closed manifold mesh about the origin.
    ///
    /// Returns the absolute value for convenience (open meshes yield an
    /// approximation that depends on the winding/origin).
    pub fn volume(&self) -> Volume {
        let v = self
            .iter_triangles()
            .map(|t| t.signed_volume_to_origin())
            .sum::<f64>();
        Volume::from_cubic_meters(v.abs())
    }

    /// Axis-aligned bounding box enclosing all vertices.
    pub fn bounds(&self) -> BoundingBox {
        let mut bb = BoundingBox::empty();
        for v in &self.vertices {
            bb.expand_point(*v);
        }
        bb
    }

    /// Apply a rigid translation to every vertex.
    pub fn translate(&mut self, offset: Vec3) {
        for v in &mut self.vertices {
            *v = v.translate(offset);
        }
    }

    /// Triangulated surfaces of the mesh (a 1:1 pass-through useful for
    /// downstream surface-extraction pipelines).
    pub fn surfaces(&self) -> Vec<Triangle> {
        self.iter_triangles().collect()
    }
}

/// A solid bounded by one or more closed shells (meshes).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Solid {
    /// Closed boundary shells.
    pub shells: Vec<Mesh>,
    /// Optional owning element.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<ElementId>,
}

impl Solid {
    /// Build a solid from one or more shells.
    pub fn new(shells: Vec<Mesh>) -> Self {
        Self {
            shells,
            owner: None,
        }
    }

    /// Attach an owning element id.
    pub fn with_owner(mut self, owner: ElementId) -> Self {
        self.owner = Some(owner);
        self
    }

    /// Bounding box of all shells.
    pub fn bounds(&self) -> BoundingBox {
        let mut bb = BoundingBox::empty();
        for s in &self.shells {
            bb = bb.union(&s.bounds());
        }
        bb
    }

    /// Total enclosed volume (sum of absolute shell volumes).
    pub fn volume(&self) -> Volume {
        Volume::from_cubic_meters(self.shells.iter().map(|s| s.volume().cubic_meters()).sum())
    }

    /// Total surface area across shells.
    pub fn surface_area(&self) -> Area {
        Area::from_square_meters(self.shells.iter().map(|s| s.area().square_meters()).sum())
    }
}

/// A uniform-grid spatial index over axis-aligned boxes.
///
/// Items are bucketed by the grid cells their box overlaps; a query returns
/// every item whose box overlaps the query box (a superset refined by an exact
/// [`BoundingBox::intersects`] test by the caller).
#[derive(Clone, Debug)]
pub struct SpatialIndex<T> {
    cell_size: f64,
    items: Vec<(BoundingBox, T)>,
}

impl<T: Clone> SpatialIndex<T> {
    /// Create an index with the given cubic cell size (m).
    pub fn new(cell_size: f64) -> Self {
        assert!(cell_size > 0.0, "cell_size must be positive");
        Self {
            cell_size,
            items: Vec::new(),
        }
    }

    fn cell_key(&self, v: f64) -> i64 {
        (v / self.cell_size).floor() as i64
    }

    /// Insert an item with its bounding box.
    pub fn insert(&mut self, box_: BoundingBox, item: T) {
        self.items.push((box_, item));
    }

    /// Return items whose box overlaps `query` (candidate set, may include
    /// false positives that the caller should re-test).
    pub fn query(&self, query: &BoundingBox) -> Vec<T> {
        let min_x = self.cell_key(query.min.x);
        let max_x = self.cell_key(query.max.x);
        let min_y = self.cell_key(query.min.y);
        let max_y = self.cell_key(query.max.y);
        let min_z = self.cell_key(query.min.z);
        let max_z = self.cell_key(query.max.z);
        let mut out = Vec::new();
        for (box_, item) in &self.items {
            let ix = self.cell_key(box_.min.x);
            let ixx = self.cell_key(box_.max.x);
            let iy = self.cell_key(box_.min.y);
            let iyy = self.cell_key(box_.max.y);
            let iz = self.cell_key(box_.min.z);
            let izz = self.cell_key(box_.max.z);
            let overlaps_grid = ix <= max_x
                && ixx >= min_x
                && iy <= max_y
                && iyy >= min_y
                && iz <= max_z
                && izz >= min_z;
            if overlaps_grid && box_.intersects(query) {
                out.push(item.clone());
            }
        }
        out
    }
}

/// A clash between two overlapping items identified by `I`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Clash<I: Clone + Eq> {
    /// First item id.
    pub a: I,
    /// Second item id.
    pub b: I,
    /// Approximate overlap volume of the two boxes (m³).
    pub overlap_volume: f64,
}

fn overlap_volume(a: BoundingBox, b: BoundingBox) -> f64 {
    let ox = (a.max.x.min(b.max.x) - a.min.x.max(b.min.x)).max(0.0);
    let oy = (a.max.y.min(b.max.y) - a.min.y.max(b.min.y)).max(0.0);
    let oz = (a.max.z.min(b.max.z) - a.min.z.max(b.min.z)).max(0.0);
    ox * oy * oz
}

/// Detect pairwise box overlaps among items, returning [`Clash`]es keyed by the
/// supplied item identifiers.
pub fn detect_clashes<I: Clone + Eq>(items: &[(I, BoundingBox)]) -> Vec<Clash<I>> {
    let mut clashes = Vec::new();
    for i in 0..items.len() {
        for j in (i + 1)..items.len() {
            if items[i].1.intersects(&items[j].1) {
                clashes.push(Clash {
                    a: items[i].0.clone(),
                    b: items[j].0.clone(),
                    overlap_volume: overlap_volume(items[i].1, items[j].1),
                });
            }
        }
    }
    clashes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn box_(min: (f64, f64, f64), max: (f64, f64, f64)) -> BoundingBox {
        BoundingBox::from_corners(
            Point3::new(min.0, min.1, min.2),
            Point3::new(max.0, max.1, max.2),
        )
    }

    #[test]
    fn vec_ops() {
        let a = Vec3::new(1.0, 0.0, 0.0);
        let b = Vec3::new(0.0, 2.0, 0.0);
        assert_eq!(a.cross(b), Vec3::new(0.0, 0.0, 2.0));
        assert!((a.distance(b) - 5.0_f64.sqrt()).abs() < 1e-12);
        assert_eq!((a + b).length(), (5.0f64).sqrt());
        assert_eq!((a * 3.0).x, 3.0);
    }

    #[test]
    fn triangle_area_normal() {
        let t = Triangle::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        );
        assert!((t.area().square_meters() - 0.5).abs() < 1e-9);
        assert_eq!(t.normal(), Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(t.unit_normal(), Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn bbox_volume_and_contains() {
        let b = box_((0.0, 0.0, 0.0), (2.0, 3.0, 4.0));
        assert!((b.volume().cubic_meters() - 24.0).abs() < 1e-9);
        assert!((b.surface_area().square_meters() - 52.0).abs() < 1e-9);
        assert!(b.contains_point(&Point3::new(1.0, 1.0, 1.0)));
        assert!(!b.contains_point(&Point3::new(3.0, 1.0, 1.0)));
    }

    #[test]
    fn bbox_union_and_intersect() {
        let a = box_((0.0, 0.0, 0.0), (1.0, 1.0, 1.0));
        let b = box_((0.5, 0.5, 0.5), (2.0, 2.0, 2.0));
        assert!(a.intersects(&b));
        let u = a.union(&b);
        assert_eq!(u.min, Point3::new(0.0, 0.0, 0.0));
        assert_eq!(u.max, Point3::new(2.0, 2.0, 2.0));
        let c = box_((5.0, 5.0, 5.0), (6.0, 6.0, 6.0));
        assert!(!a.intersects(&c));
    }

    #[test]
    fn mesh_volume_area() {
        let v = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(1.0, 0.0, 1.0),
            Point3::new(1.0, 1.0, 1.0),
            Point3::new(0.0, 1.0, 1.0),
        ];
        let idx = vec![
            [0, 1, 2],
            [0, 2, 3],
            [4, 6, 5],
            [4, 7, 6],
            [0, 4, 5],
            [0, 5, 1],
            [1, 5, 6],
            [1, 6, 2],
            [2, 6, 7],
            [2, 7, 3],
            [3, 7, 4],
            [3, 4, 0],
        ];
        let m = Mesh::new(v, idx);
        assert!((m.volume().cubic_meters() - 1.0).abs() < 1e-9);
        assert!((m.area().square_meters() - 6.0).abs() < 1e-6);
        assert_eq!(m.bounds(), box_((0.0, 0.0, 0.0), (1.0, 1.0, 1.0)));
    }

    #[test]
    fn spatial_index_query() {
        let mut idx: SpatialIndex<usize> = SpatialIndex::new(1.0);
        idx.insert(box_((0.0, 0.0, 0.0), (1.0, 1.0, 1.0)), 0);
        idx.insert(box_((5.0, 5.0, 5.0), (6.0, 6.0, 6.0)), 1);
        let near = idx.query(&box_((0.5, 0.5, 0.5), (0.6, 0.6, 0.6)));
        assert_eq!(near, vec![0]);
        let far = idx.query(&box_((5.5, 5.5, 5.5), (5.6, 5.6, 5.6)));
        assert_eq!(far, vec![1]);
    }

    #[test]
    fn clash_detection() {
        let boxes = vec![
            ("a", box_((0.0, 0.0, 0.0), (1.0, 1.0, 1.0))),
            ("b", box_((0.5, 0.5, 0.5), (1.5, 1.5, 1.5))),
            ("c", box_((10.0, 10.0, 10.0), (11.0, 11.0, 11.0))),
        ];
        let clashes = detect_clashes(&boxes);
        assert_eq!(clashes.len(), 1);
        assert_eq!(clashes[0].a, "a");
        assert_eq!(clashes[0].b, "b");
        assert!((clashes[0].overlap_volume - 0.125).abs() < 1e-12);
    }

    #[test]
    fn solid_bounds_volume() {
        let cube = Mesh::new(
            vec![
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(2.0, 0.0, 0.0),
                Point3::new(2.0, 2.0, 0.0),
                Point3::new(0.0, 2.0, 0.0),
                Point3::new(0.0, 0.0, 2.0),
                Point3::new(2.0, 0.0, 2.0),
                Point3::new(2.0, 2.0, 2.0),
                Point3::new(0.0, 2.0, 2.0),
            ],
            vec![
                [0, 1, 2],
                [0, 2, 3],
                [4, 6, 5],
                [4, 7, 6],
                [0, 4, 5],
                [0, 5, 1],
                [1, 5, 6],
                [1, 6, 2],
                [2, 6, 7],
                [2, 7, 3],
                [3, 7, 4],
                [3, 4, 0],
            ],
        );
        let s = Solid::new(vec![cube]);
        assert!((s.volume().cubic_meters() - 8.0).abs() < 1e-9);
        assert!((s.surface_area().square_meters() - 24.0).abs() < 1e-6);
        assert_eq!(s.bounds().max, Point3::new(2.0, 2.0, 2.0));
    }
}
