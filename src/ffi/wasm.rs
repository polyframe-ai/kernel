// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! WASM bindings using wasm-bindgen

use crate::{io, render};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmMesh {
    inner: crate::geometry::Mesh,
}

#[wasm_bindgen]
impl WasmMesh {
    /// Get vertex count
    pub fn vertex_count(&self) -> usize {
        self.inner.vertex_count()
    }

    /// Get triangle count
    pub fn triangle_count(&self) -> usize {
        self.inner.triangle_count()
    }

    /// Export to STL format (returns binary data)
    pub fn to_stl(&self) -> Result<Vec<u8>, JsValue> {
        let mut buffer = Vec::new();

        // Write to buffer
        use std::io::Cursor;
        let mut cursor = Cursor::new(&mut buffer);

        // Use stl_io to write
        use stl_io::{Normal, Triangle as StlTriangle, Vertex as StlVertex};

        let triangles: Vec<StlTriangle> = self
            .inner
            .triangles
            .iter()
            .map(|tri| {
                let v0 = &self.inner.vertices[tri.indices[0]];
                let v1 = &self.inner.vertices[tri.indices[1]];
                let v2 = &self.inner.vertices[tri.indices[2]];

                // Calculate normal from triangle geometry using cross product
                // This is the correct way for STL format - normals must be computed
                // from the triangle's actual geometry, not averaged vertex normals
                let normal = tri.face_normal(&self.inner);

                StlTriangle {
                    normal: Normal::new([normal.x, normal.y, normal.z]),
                    vertices: [
                        StlVertex::new([v0.position.x, v0.position.y, v0.position.z]),
                        StlVertex::new([v1.position.x, v1.position.y, v1.position.z]),
                        StlVertex::new([v2.position.x, v2.position.y, v2.position.z]),
                    ],
                }
            })
            .collect();

        stl_io::write_stl(&mut cursor, triangles.iter())
            .map_err(|e| JsValue::from_str(&format!("STL export error: {}", e)))?;

        Ok(buffer)
    }
}

/// Parse and render SCAD source code
#[wasm_bindgen]
pub fn render_scad(source: &str) -> Result<WasmMesh, JsValue> {
    let mesh = render(source).map_err(|e| JsValue::from_str(&format!("Render error: {}", e)))?;

    Ok(WasmMesh { inner: mesh })
}

/// Parse SCAD source code and return JSON AST
#[wasm_bindgen]
pub fn parse_scad_to_json(source: &str) -> Result<String, JsValue> {
    let ast =
        io::parse_scad(source).map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

    serde_json::to_string_pretty(&ast)
        .map_err(|e| JsValue::from_str(&format!("JSON serialization error: {}", e)))
}

/// Get version information
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_cube() {
        let result = render_scad("cube([10, 10, 10]);");
        assert!(result.is_ok());
    }
}
