// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Node.js bindings using napi-rs

#[cfg(feature = "napi")]
use napi::bindgen_prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;

#[cfg(feature = "napi")]
#[napi]
pub struct JsMesh {
    inner: crate::geometry::Mesh,
}

#[cfg(feature = "napi")]
#[napi]
impl JsMesh {
    /// Get vertex count
    #[napi]
    pub fn vertex_count(&self) -> u32 {
        self.inner.vertex_count() as u32
    }

    /// Get triangle count
    #[napi]
    pub fn triangle_count(&self) -> u32 {
        self.inner.triangle_count() as u32
    }

    /// Export to STL file
    #[napi]
    pub fn export_stl(&self, path: String) -> Result<()> {
        crate::io::export_stl(&self.inner, &path)
            .map_err(|e| Error::from_reason(format!("Export error: {}", e)))
    }

    /// Get bounding box as array [minX, minY, minZ, maxX, maxY, maxZ]
    #[napi]
    pub fn bounding_box(&self) -> Vec<f32> {
        let bbox = self.inner.bounding_box();
        vec![
            bbox.min.x, bbox.min.y, bbox.min.z,
            bbox.max.x, bbox.max.y, bbox.max.z,
        ]
    }
}

/// Parse and render SCAD source code
#[cfg(feature = "napi")]
#[napi]
pub fn render(source: String) -> Result<JsMesh> {
    let mesh = crate::render(&source)
        .map_err(|e| Error::from_reason(format!("Render error: {}", e)))?;
    
    Ok(JsMesh { inner: mesh })
}

/// Render SCAD file
#[cfg(feature = "napi")]
#[napi]
pub fn render_file(path: String) -> Result<JsMesh> {
    let mesh = crate::render_file(&path)
        .map_err(|e| Error::from_reason(format!("Render error: {}", e)))?;
    
    Ok(JsMesh { inner: mesh })
}

/// Parse SCAD and return JSON AST
#[cfg(feature = "napi")]
#[napi]
pub fn parse_scad(source: String) -> Result<String> {
    let ast = crate::io::parse_scad(&source)
        .map_err(|e| Error::from_reason(format!("Parse error: {}", e)))?;
    
    serde_json::to_string_pretty(&ast)
        .map_err(|e| Error::from_reason(format!("JSON error: {}", e)))
}

/// Get version
#[cfg(feature = "napi")]
#[napi]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

