// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Geospatial coordinate handling for construction sites.
//!
//! Provides coordinate reference systems, geographic (lat/long/elevation)
//! coordinates, a local site grid (east/north/elevation), datums, and the
//! transform between a project's local grid and WGS84 via a small-area tangent
//! plane approximation suitable for site and corridor work.

use serde::{Deserialize, Serialize};

/// Mean Earth radius in metres (WGS84 mean).
const EARTH_RADIUS_M: f64 = 6_371_008.8;

/// A coordinate reference system, identified by EPSG code.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Crs {
    /// EPSG numeric code (e.g. 4326 for WGS84).
    pub epsg: u32,
    /// Human-readable name.
    pub name: &'static str,
}

impl Crs {
    /// WGS84 geographic CRS (EPSG:4326).
    pub const WGS84: Crs = Crs {
        epsg: 4326,
        name: "WGS 84",
    };
    /// Web Mercator (EPSG:3857).
    pub const WEB_MERCATOR: Crs = Crs {
        epsg: 3857,
        name: "WGS 84 / Pseudo-Mercator",
    };
    /// Build a CRS from an EPSG code.
    pub fn from_epsg(epsg: u32) -> Self {
        Self {
            epsg,
            name: "custom",
        }
    }
}

/// A geographic position: latitude, longitude (degrees) and elevation (metres).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeographicCoordinate {
    /// Latitude in decimal degrees.
    pub latitude: f64,
    /// Longitude in decimal degrees.
    pub longitude: f64,
    /// Elevation in metres above the datum.
    pub elevation: f64,
}

/// A position in a project's local grid.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct LocalGridCoordinate {
    /// Easting in metres from the grid origin.
    pub east: f64,
    /// Northing in metres from the grid origin.
    pub north: f64,
    /// Elevation in metres above the datum.
    pub elevation: f64,
}

/// A vertical/horizontal reference datum.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Datum {
    /// World Geodetic System 1984.
    Wgs84,
    /// North American Datum 1983.
    Nad83,
    /// Ordnance Survey Great Britain.
    Osgb36,
    /// A project-specific local datum.
    Local,
}

/// The origin and orientation of a project's local grid, anchored to a
/// geographic point. Rotation is clockwise from grid north (degrees).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SiteDatum {
    /// Datum convention in use.
    pub datum: Datum,
    /// Geographic origin of the grid.
    pub origin: GeographicCoordinate,
    /// Grid rotation in degrees (clockwise from true north).
    pub rotation_deg: f64,
}

impl SiteDatum {
    /// Build a site datum at `origin` with no rotation.
    pub fn new(origin: GeographicCoordinate) -> Self {
        Self {
            datum: Datum::Local,
            origin,
            rotation_deg: 0.0,
        }
    }

    /// Set the grid rotation.
    pub fn with_rotation(mut self, rotation_deg: f64) -> Self {
        self.rotation_deg = rotation_deg;
        self
    }

    /// Transform a geographic coordinate into the local grid.
    pub fn to_local(&self, geo: GeographicCoordinate) -> LocalGridCoordinate {
        let lat0 = self.origin.latitude.to_radians();
        let d_lat = (geo.latitude - self.origin.latitude).to_radians();
        let d_lon = (geo.longitude - self.origin.longitude).to_radians();
        // Tangent-plane east/north (metres).
        let north_0 = EARTH_RADIUS_M * d_lat;
        let east_0 = EARTH_RADIUS_M * d_lon * lat0.cos();
        // Apply clockwise grid rotation about the origin.
        let r = self.rotation_deg.to_radians();
        let east = east_0 * r.cos() + north_0 * r.sin();
        let north = -east_0 * r.sin() + north_0 * r.cos();
        LocalGridCoordinate {
            east,
            north,
            elevation: geo.elevation - self.origin.elevation,
        }
    }

    /// Transform a local grid coordinate back to geographic.
    pub fn to_geo(&self, local: LocalGridCoordinate) -> GeographicCoordinate {
        let r = self.rotation_deg.to_radians();
        let east_0 = local.east * r.cos() - local.north * r.sin();
        let north_0 = local.east * r.sin() + local.north * r.cos();
        let lat0 = self.origin.latitude.to_radians();
        let d_lat = north_0 / EARTH_RADIUS_M;
        let d_lon = east_0 / (EARTH_RADIUS_M * lat0.cos());
        GeographicCoordinate {
            latitude: self.origin.latitude + d_lat.to_degrees(),
            longitude: self.origin.longitude + d_lon.to_degrees(),
            elevation: local.elevation + self.origin.elevation,
        }
    }
}

/// A survey grid line definition for staking (e.g. grid A / grid 1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GridLine {
    /// Label, e.g. `A` or `10`.
    pub label: String,
    /// Whether the line runs north–south (true) or east–west (false).
    pub north_south: bool,
    /// Coordinate of the line: easting if north–south, northing if east–west.
    pub position: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_local_geo() {
        let datum = SiteDatum::new(GeographicCoordinate {
            latitude: 40.0,
            longitude: -75.0,
            elevation: 10.0,
        });
        let geo = GeographicCoordinate {
            latitude: 40.0009,
            longitude: -74.9987,
            elevation: 12.5,
        };
        let local = datum.to_local(geo);
        let back = datum.to_geo(local);
        assert!((back.latitude - geo.latitude).abs() < 1e-6);
        assert!((back.longitude - geo.longitude).abs() < 1e-6);
        assert!((back.elevation - geo.elevation).abs() < 1e-9);
        assert!((local.elevation - 2.5).abs() < 1e-9);
    }

    #[test]
    fn rotation_identity_at_origin() {
        let datum = SiteDatum::new(GeographicCoordinate {
            latitude: 0.0,
            longitude: 0.0,
            elevation: 0.0,
        })
        .with_rotation(30.0);
        // A point due east should map to a rotated local coordinate.
        let local = datum.to_local(GeographicCoordinate {
            latitude: 0.0,
            longitude: 0.01,
            elevation: 0.0,
        });
        assert!(local.east > 0.0);
    }
}
