// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! STL file comparison logic

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

/// Tolerance constants (passed to MeshDiff::compare)
pub const BBOX_TOL: f32 = 1e-5;

/// Comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comparison {
    pub vertices_diff: f32,
    pub triangles_diff: f32,
    pub bbox_diff: f32,
    pub checksum_match: bool,
    pub passed: bool,
    pub vertex_count_poly: usize,
    pub vertex_count_openscad: usize,
    pub triangle_count_poly: usize,
    pub triangle_count_openscad: usize,
}

/// Compare two STL files
pub fn compare_stl_files(polyframe_path: &Path, openscad_path: &Path) -> Result<Comparison> {
    use crate::cli::{MeshDiff, Runner};

    let runner = Runner::new();

    // Load both meshes
    let poly_mesh = runner.load_stl(polyframe_path)?;
    let openscad_mesh = runner.load_stl(openscad_path)?;

    // Use the same comparison logic as the compare command (with sophisticated tolerances)
    let comparison_result = MeshDiff::compare(&poly_mesh, &openscad_mesh, BBOX_TOL);

    // Checksum comparison
    let poly_checksum = calc_mesh_checksum(&poly_mesh);
    let openscad_checksum = calc_mesh_checksum(&openscad_mesh);
    let checksum_match = poly_checksum == openscad_checksum;

    Ok(Comparison {
        vertices_diff: comparison_result.vertex_delta as f32,
        triangles_diff: comparison_result.triangle_delta as f32,
        bbox_diff: comparison_result.bbox_delta as f32,
        checksum_match,
        passed: comparison_result.passed,
        vertex_count_poly: poly_mesh.vertex_count(),
        vertex_count_openscad: openscad_mesh.vertex_count(),
        triangle_count_poly: poly_mesh.triangle_count(),
        triangle_count_openscad: openscad_mesh.triangle_count(),
    })
}

/// Calculate SHA256 checksum of mesh data
fn calc_mesh_checksum(mesh: &crate::geometry::Mesh) -> String {
    let mut hasher = Sha256::new();

    // Sort and hash vertices
    let mut positions: Vec<_> = mesh
        .vertices
        .iter()
        .map(|v| (v.position.x, v.position.y, v.position.z))
        .collect();
    positions.sort_by(|a, b| {
        a.0.partial_cmp(&b.0)
            .unwrap()
            .then(a.1.partial_cmp(&b.1).unwrap())
            .then(a.2.partial_cmp(&b.2).unwrap())
    });

    for (x, y, z) in positions {
        hasher.update(x.to_le_bytes());
        hasher.update(y.to_le_bytes());
        hasher.update(z.to_le_bytes());
    }

    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comparison_uses_mesh_diff() {
        // Test that we're using the MeshDiff logic (which has its own comprehensive tests)
        // This is just a smoke test to ensure the integration works
        use crate::geometry::Primitive;
        use nalgebra::Vector3;
        use tempfile::NamedTempFile;

        let mesh_a = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
        let mesh_b = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();

        // Create temp files with .stl extension (cross-platform)
        let temp_file_a = NamedTempFile::new().unwrap();
        let temp_file_b = NamedTempFile::new().unwrap();
        let path_a = temp_file_a.path().with_extension("stl");
        let path_b = temp_file_b.path().with_extension("stl");

        crate::io::export_stl(&mesh_a, path_a.to_str().unwrap()).unwrap();
        crate::io::export_stl(&mesh_b, path_b.to_str().unwrap()).unwrap();

        let comparison = compare_stl_files(&path_a, &path_b).unwrap();

        // Identical meshes should pass
        assert!(comparison.passed);
        assert_eq!(comparison.vertices_diff, 0.0);

        // Cleanup
        let _ = std::fs::remove_file(&path_a);
        let _ = std::fs::remove_file(&path_b);
    }
}
