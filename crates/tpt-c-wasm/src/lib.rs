// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! WASM bindings for TPT Construction engines.
//!
//! This crate re-exports the pure-Rust engines from sibling crates and provides
//! optional `wasm-bindgen` wrappers so they can be consumed from JavaScript.
//! The core functionality is always available; the WASM glue is gated behind
//! the `js` feature so native Rust consumers do not pull in `wasm-bindgen`.

use serde::{Deserialize, Serialize};

/// Re-export the neutral project model for WASM consumers.
pub use tpt_c_model::{Element, Project, PropertySet, PropertyValue, Quantity, QuantitySet};

/// Re-export IFC parsing results.
pub use tpt_c_ifc::{parse, to_model, IfcError};

/// Re-export quantity takeoff types.
pub use tpt_c_quantities::{TakeoffEngine, TakeoffResult};

/// Re-export schedule types.
pub use tpt_c_schedule::{Activity, Schedule};

/// Re-export geometry types.
pub use tpt_c_geometry::{BoundingBox, Mesh, Point3, Vec3};

/// Re-export glTF export.
pub use tpt_c_gltf::export;

/// A simplified result wrapper for WASM boundaries.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WasmResult<T> {
    /// Whether the operation succeeded.
    pub ok: bool,
    /// The result value, if successful.
    pub value: Option<T>,
    /// Error message, if failed.
    pub error: Option<String>,
}

impl<T> WasmResult<T> {
    /// Build a successful result.
    pub fn ok(value: T) -> Self {
        Self {
            ok: true,
            value: Some(value),
            error: None,
        }
    }

    /// Build a failed result.
    pub fn err(error: impl Into<String>) -> Self {
        Self {
            ok: false,
            value: None,
            error: Some(error.into()),
        }
    }
}

#[cfg(feature = "js")]
mod js_bindings {
    use wasm_bindgen::prelude::*;

    /// Parse IFC text into a neutral project model (WASM boundary).
    #[wasm_bindgen]
    pub fn parse_ifc(source: &str) -> Result<JsValue, JsValue> {
        let doc = super::parse(source).map_err(|e| e.to_string())?;
        let project = super::to_model(&doc).map_err(|e| e.to_string())?;
        Ok(serde_wasm_bindgen::to_value(&project).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wasm_result_ok() {
        let r: WasmResult<i32> = WasmResult::ok(42);
        assert!(r.ok);
        assert_eq!(r.value, Some(42));
        assert!(r.error.is_none());
    }

    #[test]
    fn wasm_result_err() {
        let r: WasmResult<i32> = WasmResult::err("boom");
        assert!(!r.ok);
        assert!(r.value.is_none());
        assert_eq!(r.error, Some("boom".to_string()));
    }
}
