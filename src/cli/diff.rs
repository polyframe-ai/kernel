// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Mesh comparison and diff utilities

use crate::geometry::Mesh;
use serde::{Deserialize, Serialize};

/// Result of mesh comparison with detailed metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub passed: bool,
    pub vertex_delta: f64,
    pub triangle_delta: f64,
    pub bbox_delta: f64,
    pub vertex_count_a: usize,
    pub vertex_count_b: usize,
    pub triangle_count_a: usize,
    pub triangle_count_b: usize,
    pub tolerance: f32,
}

impl ComparisonResult {
    pub fn summary(&self) -> String {
        format!(
            "Vertices: {} vs {} (Δ{:.2}%), Triangles: {} vs {} (Δ{:.2}%), BBox: Δ{:.5}",
            self.vertex_count_a,
            self.vertex_count_b,
            self.vertex_delta * 100.0,
            self.triangle_count_a,
            self.triangle_count_b,
            self.triangle_delta * 100.0,
            self.bbox_delta
        )
    }

    pub fn is_acceptable(&self, vertex_threshold: f64, bbox_threshold: f64) -> bool {
        self.vertex_delta < vertex_threshold && self.bbox_delta < bbox_threshold
    }
}

/// Mesh diff utilities
pub struct MeshDiff;

impl MeshDiff {
    /// Compare two meshes and return detailed comparison result
    pub fn compare(mesh_a: &Mesh, mesh_b: &Mesh, tolerance: f32) -> ComparisonResult {
        let vertex_count_a = mesh_a.vertex_count();
        let vertex_count_b = mesh_b.vertex_count();
        let triangle_count_a = mesh_a.triangle_count();
        let triangle_count_b = mesh_b.triangle_count();

        // Calculate deltas
        let vertex_delta = Self::diff_ratio(vertex_count_a, vertex_count_b);
        let triangle_delta = Self::diff_ratio(triangle_count_a, triangle_count_b);

        // Compare bounding boxes
        let bbox_a = mesh_a.bounding_box();
        let bbox_b = mesh_b.bounding_box();
        let bbox_delta = Self::bbox_distance(&bbox_a, &bbox_b);

        // Determine if passed
        let passed = vertex_delta < 0.01 && bbox_delta < tolerance as f64;

        ComparisonResult {
            passed,
            vertex_delta,
            triangle_delta,
            bbox_delta,
            vertex_count_a,
            vertex_count_b,
            triangle_count_a,
            triangle_count_b,
            tolerance,
        }
    }

    /// Calculate ratio difference between two counts
    fn diff_ratio(a: usize, b: usize) -> f64 {
        if a == 0 && b == 0 {
            return 0.0;
        }

        let max = a.max(b) as f64;
        let diff = (a as i64 - b as i64).abs() as f64;

        diff / max
    }

    /// Calculate distance between two bounding boxes
    fn bbox_distance(
        bbox_a: &crate::geometry::BoundingBox,
        bbox_b: &crate::geometry::BoundingBox,
    ) -> f64 {
        let min_dist = (bbox_a.min - bbox_b.min).norm();
        let max_dist = (bbox_a.max - bbox_b.max).norm();

        (min_dist + max_dist) as f64 / 2.0
    }

    /// Calculate percentage difference
    pub fn percentage_diff(a: usize, b: usize) -> f64 {
        Self::diff_ratio(a, b) * 100.0
    }

    /// Check if two meshes are approximately equal
    pub fn are_equivalent(mesh_a: &Mesh, mesh_b: &Mesh, tolerance: f32) -> bool {
        let result = Self::compare(mesh_a, mesh_b, tolerance);
        result.passed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Primitive;
    use nalgebra::Vector3;

    #[test]
    fn test_identical_meshes() {
        let mesh_a = Primitive::cube(Vector3::new(10.0, 10.0, 10.0)).to_mesh();
        let mesh_b = Primitive::cube(Vector3::new(10.0, 10.0, 10.0)).to_mesh();

        let result = MeshDiff::compare(&mesh_a, &mesh_b, 0.001);
        assert!(result.passed);
        assert_eq!(result.vertex_delta, 0.0);
    }

    #[test]
    fn test_different_meshes() {
        let mesh_a = Primitive::cube(Vector3::new(10.0, 10.0, 10.0)).to_mesh();
        let mesh_b = Primitive::cube(Vector3::new(20.0, 20.0, 20.0)).to_mesh();

        let result = MeshDiff::compare(&mesh_a, &mesh_b, 0.001);
        assert!(!result.passed);
    }

    #[test]
    fn test_diff_ratio() {
        assert_eq!(MeshDiff::diff_ratio(100, 100), 0.0);
        assert_eq!(MeshDiff::diff_ratio(100, 110), 0.09090909090909091);
        assert_eq!(MeshDiff::diff_ratio(0, 0), 0.0);
    }
}
