// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Mesh comparison utilities for I/O equivalence testing

use crate::geometry::Mesh;
use serde::{Deserialize, Serialize};

/// Result of mesh comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshComparison {
    pub vertex_count_match: bool,
    pub triangle_count_match: bool,
    pub bbox_match: bool,
    pub vertex_count_diff: i32,
    pub triangle_count_diff: i32,
    pub bbox_tolerance: f32,
    pub passed: bool,
}

impl MeshComparison {
    pub fn new() -> Self {
        Self {
            vertex_count_match: false,
            triangle_count_match: false,
            bbox_match: false,
            vertex_count_diff: 0,
            triangle_count_diff: 0,
            bbox_tolerance: 0.0,
            passed: false,
        }
    }
}

impl Default for MeshComparison {
    fn default() -> Self {
        Self::new()
    }
}

/// Compare two meshes for equivalence
pub fn compare_meshes(mesh_a: &Mesh, mesh_b: &Mesh, tolerance: f32) -> MeshComparison {
    let mut comparison = MeshComparison::new();

    // Compare vertex counts
    let vertex_diff = mesh_a.vertex_count() as i32 - mesh_b.vertex_count() as i32;
    comparison.vertex_count_diff = vertex_diff;
    comparison.vertex_count_match = vertex_diff.abs() == 0;

    // Compare triangle counts
    let triangle_diff = mesh_a.triangle_count() as i32 - mesh_b.triangle_count() as i32;
    comparison.triangle_count_diff = triangle_diff;
    comparison.triangle_count_match = triangle_diff.abs() == 0;

    // Compare bounding boxes
    let bbox_a = mesh_a.bounding_box();
    let bbox_b = mesh_b.bounding_box();
    comparison.bbox_match = bbox_a.approx_eq(&bbox_b, tolerance);
    comparison.bbox_tolerance = tolerance;

    // Overall pass if all checks pass
    comparison.passed =
        comparison.vertex_count_match && comparison.triangle_count_match && comparison.bbox_match;

    comparison
}

/// Compare triangle count with percentage tolerance
#[allow(dead_code)]
pub fn compare_triangle_count_with_tolerance(
    mesh_a: &Mesh,
    mesh_b: &Mesh,
    tolerance_percent: f32,
) -> bool {
    let count_a = mesh_a.triangle_count() as f32;
    let count_b = mesh_b.triangle_count() as f32;

    if count_a == 0.0 && count_b == 0.0 {
        return true;
    }

    let diff = (count_a - count_b).abs();
    let max_count = count_a.max(count_b);
    let percent_diff = (diff / max_count) * 100.0;

    percent_diff <= tolerance_percent
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Primitive;
    use nalgebra::Vector3;

    #[test]
    fn test_compare_identical_meshes() {
        let mesh_a = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), true).to_mesh();
        let mesh_b = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), true).to_mesh();

        let comparison = compare_meshes(&mesh_a, &mesh_b, 0.001);
        assert!(comparison.passed);
        assert!(comparison.vertex_count_match);
        assert!(comparison.triangle_count_match);
        assert!(comparison.bbox_match);
    }

    #[test]
    fn test_compare_different_sizes() {
        let mesh_a = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), true).to_mesh();
        let mesh_b = Primitive::cube(Vector3::new(20.0, 20.0, 20.0), true).to_mesh();

        let comparison = compare_meshes(&mesh_a, &mesh_b, 0.001);
        assert!(!comparison.bbox_match);
    }

    #[test]
    fn test_triangle_count_tolerance() {
        let mesh_a = Primitive::sphere(10.0, 32).to_mesh();
        let mesh_b = Primitive::sphere(10.0, 16).to_mesh();

        // Should fail with 0% tolerance
        assert!(!compare_triangle_count_with_tolerance(
            &mesh_a, &mesh_b, 0.0
        ));

        // Should pass with high tolerance
        assert!(compare_triangle_count_with_tolerance(
            &mesh_a, &mesh_b, 100.0
        ));
    }
}
