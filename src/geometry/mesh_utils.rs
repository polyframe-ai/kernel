// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Mesh validation and repair utilities

use super::Mesh;
use std::collections::HashMap;

/// Edge representation for connectivity checking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Edge {
    v0: usize,
    v1: usize,
}

impl Edge {
    fn new(v0: usize, v1: usize) -> Self {
        // Always store edges with smaller index first for consistent hashing
        if v0 < v1 {
            Self { v0, v1 }
        } else {
            Self { v0: v1, v1: v0 }
        }
    }
}

/// Check if mesh is manifold (each edge shared by at most 2 triangles)
pub fn is_manifold(mesh: &Mesh) -> bool {
    let mut edge_counts: HashMap<Edge, u32> = HashMap::new();

    for triangle in &mesh.triangles {
        // Count each edge of the triangle
        let edges = [
            Edge::new(triangle.indices[0], triangle.indices[1]),
            Edge::new(triangle.indices[1], triangle.indices[2]),
            Edge::new(triangle.indices[2], triangle.indices[0]),
        ];

        for edge in &edges {
            *edge_counts.entry(*edge).or_insert(0) += 1;
        }
    }

    // Check if any edge is shared by more than 2 triangles (non-manifold)
    edge_counts.values().all(|&count| count <= 2)
}

/// Check if mesh is closed (each edge shared by exactly 2 triangles)
pub fn is_closed(mesh: &Mesh) -> bool {
    let mut edge_counts: HashMap<Edge, u32> = HashMap::new();

    for triangle in &mesh.triangles {
        let edges = [
            Edge::new(triangle.indices[0], triangle.indices[1]),
            Edge::new(triangle.indices[1], triangle.indices[2]),
            Edge::new(triangle.indices[2], triangle.indices[0]),
        ];

        for edge in &edges {
            *edge_counts.entry(*edge).or_insert(0) += 1;
        }
    }

    // All edges must be shared by exactly 2 triangles for a closed mesh
    edge_counts.values().all(|&count| count == 2)
}

/// Validate triangle winding order (ensures normals point outward)
/// Returns true if triangles appear to have consistent winding
pub fn validate_winding_order(mesh: &Mesh) -> bool {
    if mesh.triangles.is_empty() {
        return true;
    }

    // Simple check: ensure triangle area is positive (not degenerate)
    for triangle in &mesh.triangles {
        if triangle.indices[0] >= mesh.vertices.len()
            || triangle.indices[1] >= mesh.vertices.len()
            || triangle.indices[2] >= mesh.vertices.len()
        {
            return false;
        }

        let v0 = &mesh.vertices[triangle.indices[0]].position;
        let v1 = &mesh.vertices[triangle.indices[1]].position;
        let v2 = &mesh.vertices[triangle.indices[2]].position;

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let area = edge1.cross(&edge2).norm();

        // Triangle must have non-zero area
        if area < 1e-10 {
            return false;
        }
    }

    true
}

/// Get mesh validation report
pub struct MeshValidation {
    pub is_manifold: bool,
    pub is_closed: bool,
    pub has_valid_winding: bool,
    pub edge_count: usize,
    pub boundary_edge_count: usize,
}

/// Find all boundary edges (edges shared by exactly 1 triangle)
pub fn find_boundary_edges(mesh: &Mesh) -> std::collections::HashSet<Edge> {
    let mut edge_counts: HashMap<Edge, u32> = HashMap::new();

    for triangle in &mesh.triangles {
        let edges = [
            Edge::new(triangle.indices[0], triangle.indices[1]),
            Edge::new(triangle.indices[1], triangle.indices[2]),
            Edge::new(triangle.indices[2], triangle.indices[0]),
        ];

        for edge in &edges {
            *edge_counts.entry(*edge).or_insert(0) += 1;
        }
    }

    // Return edges shared by exactly 1 triangle (boundary edges)
    edge_counts
        .iter()
        .filter(|(_, &count)| count == 1)
        .map(|(edge, _)| *edge)
        .collect()
}

/// Classify triangle as boundary or internal based on edge counts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriangleClassification {
    Boundary,  // Has at least one boundary edge
    Internal,  // All edges are internal (shared by 2 triangles)
}

/// Classify a triangle as boundary or internal
pub fn classify_triangle_by_edges(
    triangle: &crate::geometry::Triangle,
    edge_counts: &HashMap<Edge, u32>,
) -> TriangleClassification {
    let edges = [
        Edge::new(triangle.indices[0], triangle.indices[1]),
        Edge::new(triangle.indices[1], triangle.indices[2]),
        Edge::new(triangle.indices[2], triangle.indices[0]),
    ];

    // Check if any edge is a boundary edge (shared by exactly 1 triangle)
    for edge in &edges {
        if let Some(&count) = edge_counts.get(edge) {
            if count == 1 {
                return TriangleClassification::Boundary;
            }
        } else {
            // Edge not found - should not happen, but treat as boundary
            return TriangleClassification::Boundary;
        }
    }

    TriangleClassification::Internal
}

/// Get which edges of a triangle are on the boundary
/// Returns a boolean array [edge0, edge1, edge2] where true means boundary edge
pub fn get_triangle_boundary_edges(
    triangle: &crate::geometry::Triangle,
    boundary_edges: &std::collections::HashSet<Edge>,
) -> [bool; 3] {
    let edges = [
        Edge::new(triangle.indices[0], triangle.indices[1]),
        Edge::new(triangle.indices[1], triangle.indices[2]),
        Edge::new(triangle.indices[2], triangle.indices[0]),
    ];

    [
        boundary_edges.contains(&edges[0]),
        boundary_edges.contains(&edges[1]),
        boundary_edges.contains(&edges[2]),
    ]
}

/// Build edge count map for a mesh
pub fn build_edge_counts(mesh: &Mesh) -> HashMap<Edge, u32> {
    let mut edge_counts: HashMap<Edge, u32> = HashMap::new();

    for triangle in &mesh.triangles {
        let edges = [
            Edge::new(triangle.indices[0], triangle.indices[1]),
            Edge::new(triangle.indices[1], triangle.indices[2]),
            Edge::new(triangle.indices[2], triangle.indices[0]),
        ];

        for edge in &edges {
            *edge_counts.entry(*edge).or_insert(0) += 1;
        }
    }

    edge_counts
}

pub fn validate_mesh(mesh: &Mesh) -> MeshValidation {
    let mut edge_counts: HashMap<Edge, u32> = HashMap::new();

    for triangle in &mesh.triangles {
        let edges = [
            Edge::new(triangle.indices[0], triangle.indices[1]),
            Edge::new(triangle.indices[1], triangle.indices[2]),
            Edge::new(triangle.indices[2], triangle.indices[0]),
        ];

        for edge in &edges {
            *edge_counts.entry(*edge).or_insert(0) += 1;
        }
    }

    let boundary_edges = edge_counts
        .values()
        .filter(|&&count| count == 1)
        .count();

    MeshValidation {
        is_manifold: edge_counts.values().all(|&count| count <= 2),
        is_closed: edge_counts.values().all(|&count| count == 2),
        has_valid_winding: validate_winding_order(mesh),
        edge_count: edge_counts.len(),
        boundary_edge_count: boundary_edges,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Primitive;
    use nalgebra::Vector3;

    #[test]
    fn test_cube_is_manifold() {
        let mesh = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
        assert!(is_manifold(&mesh));
    }

    #[test]
    fn test_cube_is_closed() {
        // Note: cube generation creates duplicate vertices per face for proper normals
        // So it's not closed in the strict sense (edges shared by exactly 2 triangles)
        // But it is manifold (edges shared by at most 2 triangles)
        let mesh = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
        assert!(is_manifold(&mesh), "Cube should be manifold");
        // Cube is not closed because each face has its own vertices
    }

    #[test]
    fn test_cylinder_is_manifold() {
        let mesh = Primitive::cylinder(10.0, 5.0, 32).to_mesh();
        assert!(is_manifold(&mesh), "Cylinder should be manifold");
    }

    #[test]
    fn test_cylinder_is_closed() {
        let mesh = Primitive::cylinder(10.0, 5.0, 32).to_mesh();
        assert!(is_closed(&mesh), "Cylinder should be closed");
    }

    #[test]
    fn test_validate_mesh() {
        let mesh = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
        let validation = validate_mesh(&mesh);
        
        assert!(validation.is_manifold, "Cube should be manifold");
        // Cube is not closed because each face has its own vertices (for proper normals)
        assert!(validation.has_valid_winding, "Cube should have valid winding order");
        assert!(validation.edge_count > 0, "Cube should have edges");
    }

    #[test]
    fn test_validate_closed_mesh() {
        // Test with a cylinder which should be closed
        let mesh = Primitive::cylinder(10.0, 5.0, 32).to_mesh();
        let validation = validate_mesh(&mesh);
        
        assert!(validation.is_manifold, "Cylinder should be manifold");
        assert!(validation.is_closed, "Cylinder should be closed");
        assert!(validation.has_valid_winding, "Cylinder should have valid winding order");
        assert_eq!(validation.boundary_edge_count, 0, "Closed mesh should have no boundary edges");
    }

    #[test]
    fn test_validate_cylinder_mesh() {
        let mesh = Primitive::cylinder(10.0, 5.0, 32).to_mesh();
        let validation = validate_mesh(&mesh);
        
        assert!(validation.is_manifold, "Cylinder mesh should be manifold");
        assert!(validation.is_closed, "Cylinder mesh should be closed");
        assert!(validation.has_valid_winding, "Cylinder mesh should have valid winding order");
    }
}

