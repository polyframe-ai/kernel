// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! STL file comparison logic

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

/// Tolerance constants
pub const VERTEX_TOL: f32 = 0.01; // 1%
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
    use crate::cli::Runner;

    let runner = Runner::new();

    // Load both meshes
    let poly_mesh = runner.load_stl(polyframe_path)?;
    let openscad_mesh = runner.load_stl(openscad_path)?;

    // Calculate differences
    let vertices_diff = calc_diff_ratio(poly_mesh.vertex_count(), openscad_mesh.vertex_count());

    let triangles_diff =
        calc_diff_ratio(poly_mesh.triangle_count(), openscad_mesh.triangle_count());

    // Bounding box comparison
    let poly_bbox = poly_mesh.bounding_box();
    let openscad_bbox = openscad_mesh.bounding_box();
    let bbox_diff = calc_bbox_diff(&poly_bbox, &openscad_bbox);

    // Checksum comparison
    let poly_checksum = calc_mesh_checksum(&poly_mesh);
    let openscad_checksum = calc_mesh_checksum(&openscad_mesh);
    let checksum_match = poly_checksum == openscad_checksum;

    // Determine pass/fail
    let passed = vertices_diff < VERTEX_TOL && bbox_diff < BBOX_TOL;

    Ok(Comparison {
        vertices_diff,
        triangles_diff,
        bbox_diff,
        checksum_match,
        passed,
        vertex_count_poly: poly_mesh.vertex_count(),
        vertex_count_openscad: openscad_mesh.vertex_count(),
        triangle_count_poly: poly_mesh.triangle_count(),
        triangle_count_openscad: openscad_mesh.triangle_count(),
    })
}

/// Calculate ratio difference between two counts
fn calc_diff_ratio(a: usize, b: usize) -> f32 {
    if a == 0 && b == 0 {
        return 0.0;
    }

    let max = a.max(b) as f32;
    let diff = (a as i64 - b as i64).abs() as f32;

    diff / max
}

/// Calculate bounding box difference
fn calc_bbox_diff(
    bbox_a: &crate::geometry::BoundingBox,
    bbox_b: &crate::geometry::BoundingBox,
) -> f32 {
    let min_dist = (bbox_a.min - bbox_b.min).norm();
    let max_dist = (bbox_a.max - bbox_b.max).norm();

    (min_dist + max_dist) / 2.0
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
    fn test_diff_ratio() {
        assert_eq!(calc_diff_ratio(100, 100), 0.0);
        assert!((calc_diff_ratio(100, 110) - 0.0909).abs() < 0.001);
        assert_eq!(calc_diff_ratio(0, 0), 0.0);
    }
}
