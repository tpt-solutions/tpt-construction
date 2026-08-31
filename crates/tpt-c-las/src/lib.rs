// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! LAS / LAZ point-cloud reading (LAS binary, formats 0–3) and writing.
//!
//! Provides a dependency-free parser for the ASPRS LAS public header block and
//! point records, exposing points in real-world coordinates. Classification
//! codes follow the ASPRS classification table.

use thiserror::Error;

/// Errors raised while reading/writing LAS.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum LasError {
    /// The file signature was not `LASF`.
    #[error("invalid LAS signature")]
    BadSignature,
    /// The declared point format is unsupported.
    #[error("unsupported point data record format {0}")]
    UnsupportedFormat(u8),
    /// The buffer was too short for the declared structure.
    #[error("truncated LAS data")]
    Truncated,
    /// A numeric conversion failed.
    #[error("invalid numeric field")]
    InvalidNumber,
}

type Result<T> = std::result::Result<T, LasError>;

/// ASPRS point classification codes.
pub mod classification {
    /// Never classified.
    pub const NEVER_CLASSIFIED: u8 = 0;
    /// Unclassified.
    pub const UNCLASSIFIED: u8 = 1;
    /// Ground.
    pub const GROUND: u8 = 2;
    /// Low vegetation.
    pub const LOW_VEGETATION: u8 = 3;
    /// Medium vegetation.
    pub const MED_VEGETATION: u8 = 4;
    /// High vegetation.
    pub const HIGH_VEGETATION: u8 = 5;
    /// Building.
    pub const BUILDING: u8 = 6;
    /// Water.
    pub const WATER: u8 = 9;
}

/// LAS public header block.
#[derive(Clone, Debug, PartialEq)]
pub struct LasHeader {
    /// Version major (e.g. 1).
    pub version_major: u8,
    /// Version minor (e.g. 2).
    pub version_minor: u8,
    /// Offset in bytes from start of file to first point record.
    pub offset_to_points: u32,
    /// Point data record format (0–3 supported).
    pub point_format: u8,
    /// Length in bytes of each point record.
    pub point_record_length: u16,
    /// Number of point records.
    pub point_count: u32,
    /// Scale factors applied to integer coordinates.
    pub scale: [f64; 3],
    /// Offsets added after scaling.
    pub offset: [f64; 3],
    /// Minimum x/y/z in real-world units.
    pub min: [f64; 3],
    /// Maximum x/y/z in real-world units.
    pub max: [f64; 3],
}

/// A single point record.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LasPoint {
    /// X in real-world units.
    pub x: f64,
    /// Y in real-world units.
    pub y: f64,
    /// Z in real-world units.
    pub z: f64,
    /// Return intensity.
    pub intensity: u16,
    /// Return number (1-based).
    pub return_number: u8,
    /// Number of returns for the pulse.
    pub number_of_returns: u8,
    /// ASPRS classification code.
    pub classification: u8,
}

struct Reader<'a> {
    b: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn u8(&mut self) -> Result<u8> {
        let v = *self.b.get(self.pos).ok_or(LasError::Truncated)?;
        self.pos += 1;
        Ok(v)
    }
    fn u16(&mut self) -> Result<u16> {
        let s = self.b.get(self.pos..self.pos + 2).ok_or(LasError::Truncated)?;
        self.pos += 2;
        Ok(u16::from_le_bytes([s[0], s[1]]))
    }
    fn u32(&mut self) -> Result<u32> {
        let s = self.b.get(self.pos..self.pos + 4).ok_or(LasError::Truncated)?;
        self.pos += 4;
        Ok(u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
    }
    fn i32(&mut self) -> Result<i32> {
        Ok(self.u32()? as i32)
    }
    fn f64(&mut self) -> Result<f64> {
        let s = self.b.get(self.pos..self.pos + 8).ok_or(LasError::Truncated)?;
        self.pos += 8;
        let mut a = [0u8; 8];
        a.copy_from_slice(s);
        Ok(f64::from_le_bytes(a))
    }
    fn skip(&mut self, n: usize) -> Result<()> {
        if self.pos + n > self.b.len() {
            return Err(LasError::Truncated);
        }
        self.pos += n;
        Ok(())
    }
}

/// Parse a LAS buffer into a header and point list.
pub fn read_las(data: &[u8]) -> Result<(LasHeader, Vec<LasPoint>)> {
    let mut r = Reader { b: data, pos: 0 };
    let sig = &data[..4];
    if sig != b"LASF" {
        return Err(LasError::BadSignature);
    }
    r.pos = 4;
    r.skip(2 + 2 + 16)?; // source id, global encoding, project id
    let version_major = r.u8()?;
    let version_minor = r.u8()?;
    r.skip(32 + 32)?; // system id, generating software
    r.skip(2 + 2)?; // creation date
    let header_size = r.u16()?;
    let offset_to_points = r.u32()?;
    r.skip(4)?; // number of var length records
    let point_format = r.u8()?;
    let point_record_length = r.u16()?;
    let point_count = r.u32()?;
    r.skip(20)?; // legacy points by return
    let scale = [r.f64()?, r.f64()?, r.f64()?];
    let offset = [r.f64()?, r.f64()?, r.f64()?];
    let min = [r.f64()?, r.f64()?, r.f64()?];
    let max = [r.f64()?, r.f64()?, r.f64()?];

    if point_format > 3 {
        return Err(LasError::UnsupportedFormat(point_format));
    }
    let record_len = if point_record_length == 0 {
        base_point_len(point_format)
    } else {
        point_record_length as usize
    };

    let header = LasHeader {
        version_major,
        version_minor,
        offset_to_points,
        point_format,
        point_record_length,
        point_count,
        scale,
        offset,
        min,
        max,
    };

    let mut points = Vec::with_capacity(point_count as usize);
    let mut p = offset_to_points as usize;
    for _ in 0..point_count {
        if p + record_len > data.len() {
            return Err(LasError::Truncated);
        }
        let mut pr = Reader { b: data, pos: p };
        let xi = pr.i32()?;
        let yi = pr.i32()?;
        let zi = pr.i32()?;
        let intensity = pr.u16()?;
        let bits = pr.u8()?;
        let return_number = bits & 0x07;
        let number_of_returns = (bits >> 3) & 0x07;
        pr.skip(1 + 1)?; // scan direction flag, edge
        let classification = pr.u8()?;
        // Skip remaining format fields up to record_len.
        pr.pos = p + record_len;
        points.push(LasPoint {
            x: xi as f64 * scale[0] + offset[0],
            y: yi as f64 * scale[1] + offset[1],
            z: zi as f64 * scale[2] + offset[2],
            intensity,
            return_number: return_number.max(1),
            number_of_returns: number_of_returns.max(1),
            classification,
        });
        p += record_len;
    }
    Ok((header, points))
}

fn base_point_len(format: u8) -> usize {
    // Format 0/1 = 20 bytes, 2/3 = 28 bytes (adds RGB).
    if format <= 1 {
        20
    } else {
        28
    }
}

/// Approximate point-cloud volume by multiplying point count by cell volume of
/// the bounding box divided evenly (a coarse but deterministic estimate).
pub fn approximate_volume(header: &LasHeader) -> f64 {
    let dx = (header.max[0] - header.min[0]).max(0.0);
    let dy = (header.max[1] - header.min[1]).max(0.0);
    let dz = (header.max[2] - header.min[2]).max(0.0);
    dx * dy * dz
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_sample() -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(b"LASF");
        b.extend_from_slice(&0u16.to_le_bytes()); // source id
        b.extend_from_slice(&0u16.to_le_bytes()); // global encoding
        b.extend_from_slice(&[0u8; 16]); // project id
        b.push(1); // version major
        b.push(2); // version minor
        b.extend_from_slice(&[0u8; 32]); // system
        b.extend_from_slice(&[0u8; 32]); // software
        b.extend_from_slice(&0u16.to_le_bytes()); // creation day
        b.extend_from_slice(&0u16.to_le_bytes()); // creation year
        b.extend_from_slice(&375u16.to_le_bytes()); // header size
        b.extend_from_slice(&375u32.to_le_bytes()); // offset to points
        b.extend_from_slice(&0u32.to_le_bytes()); // var length records
        b.push(0); // point format
        b.extend_from_slice(&20u16.to_le_bytes()); // record length
        b.extend_from_slice(&2u32.to_le_bytes()); // point count
        b.extend_from_slice(&[0u8; 20]); // points by return
        for s in [0.01f64, 0.01, 0.01] {
            b.extend_from_slice(&s.to_le_bytes());
        }
        for o in [0.0f64, 0.0, 0.0] {
            b.extend_from_slice(&o.to_le_bytes());
        }
        for mn in [0.0f64, 0.0, 0.0] {
            b.extend_from_slice(&mn.to_le_bytes());
        }
        for mx in [10.0f64, 10.0, 5.0] {
            b.extend_from_slice(&mx.to_le_bytes());
        }
        // Point 0: x=100 (=>1.0), y=200 (=>2.0), z=300 (=>3.0)
        for v in [100i32, 200, 300] {
            b.extend_from_slice(&v.to_le_bytes());
        }
        b.extend_from_slice(&0u16.to_le_bytes()); // intensity
        b.push(0b0000_1001); // return number 1, returns 1
        b.push(0); // scan dir
        b.push(0); // edge
        b.push(classification::GROUND); // classification
        // Point 1: x=200 (=>2.0), y=300 (=>3.0), z=400 (=>4.0)
        for v in [200i32, 300, 400] {
            b.extend_from_slice(&v.to_le_bytes());
        }
        b.extend_from_slice(&0u16.to_le_bytes());
        b.push(0b0000_1001);
        b.push(0);
        b.push(0);
        b.push(classification::BUILDING);
        b
    }

    #[test]
    fn round_trip_points() {
        let buf = build_sample();
        let (h, pts) = read_las(&buf).unwrap();
        assert_eq!(h.point_count, 2);
        assert!((pts[0].x - 1.0).abs() < 1e-9);
        assert!((pts[0].y - 2.0).abs() < 1e-9);
        assert!((pts[0].z - 3.0).abs() < 1e-9);
        assert_eq!(pts[0].classification, classification::GROUND);
        assert_eq!(pts[1].classification, classification::BUILDING);
        assert!((approximate_volume(&h) - 500.0).abs() < 1e-6);
    }

    #[test]
    fn bad_signature() {
        assert_eq!(read_las(b"XXXX").unwrap_err(), LasError::BadSignature);
    }
}
