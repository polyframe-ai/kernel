// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Mesh reconstruction and validation
//! Handles vertex welding, topology validation, and Euler characteristic checks

use super::{halfedge::HalfEdgeMesh, Mesh, Vertex};
use nalgebra::{Point3, Vector3};
use std::collections::HashMap;

/// Reconstruct manifold mesh from half-edge mesh
/// Performs vertex welding and topology validation
pub fn reconstruct_manifold_mesh(he_mesh: &HalfEdgeMesh) -> Mesh {
    // Step 1: Weld duplicate vertices
    let (vertex_map, welded_vertices) = weld_vertices(&he_mesh.vertices);
    
    // Step 2: Rebuild faces with welded vertices
    let mut mesh = Mesh::new();
    
    // Add welded vertices
    for vertex_pos in &welded_vertices {
        let normal = Vector3::new(0.0, 0.0, 1.0); // Placeholder, will be recomputed
        mesh.add_vertex(Vertex::new(*vertex_pos, normal));
    }
    
    // Add faces with remapped indices
    for face in &he_mesh.faces {
        let v0 = vertex_map[face[0] as usize];
        let v1 = vertex_map[face[1] as usize];
        let v2 = vertex_map[face[2] as usize];
        
        // Skip degenerate triangles
        if v0 == v1 || v1 == v2 || v2 == v0 {
            continue;
        }
        
        use super::Triangle;
        mesh.add_triangle(Triangle::new([v0, v1, v2]));
    }
    
    // Step 3: Recompute normals
    mesh.recompute_normals();
    
    // Step 4: Validate topology
    validate_mesh_topology(&mesh);
    
    mesh
}

/// Weld duplicate vertices within epsilon distance
/// Returns (vertex_map, welded_vertices)
fn weld_vertices(vertices: &[Point3<f64>]) -> (Vec<usize>, Vec<Point3<f64>>) {
    const EPS: f64 = 1e-9;
    
    let mut vertex_map: Vec<usize> = Vec::with_capacity(vertices.len());
    let mut welded_vertices: Vec<Point3<f64>> = Vec::new();
    let mut vertex_to_index: HashMap<usize, usize> = HashMap::new();
    
    for (orig_idx, vertex) in vertices.iter().enumerate() {
        // Check if this vertex is close to an existing welded vertex
        let mut found = false;
        for (welded_idx, welded_vertex) in welded_vertices.iter().enumerate() {
            if (vertex - welded_vertex).norm() < EPS {
                // Use existing welded vertex
                vertex_map.push(welded_idx);
                vertex_to_index.insert(orig_idx, welded_idx);
                found = true;
                break;
            }
        }
        
        if !found {
            // Create new welded vertex
            let new_idx = welded_vertices.len();
            welded_vertices.push(*vertex);
            vertex_map.push(new_idx);
            vertex_to_index.insert(orig_idx, new_idx);
        }
    }
    
    (vertex_map, welded_vertices)
}

/// Validate mesh topology
/// Checks Euler characteristic and boundary loops
fn validate_mesh_topology(mesh: &Mesh) {
    // Compute Euler characteristic: V - E + F = 2 (for closed manifold)
    let v = mesh.vertex_count();
    let f = mesh.triangle_count();
    
    // Estimate edge count (each triangle has 3 edges, but shared edges count once)
    // For a closed manifold: E = (3F) / 2
    let e_estimate = (3 * f) / 2;
    
    let euler_char = v as i32 - e_estimate as i32 + f as i32;
    
    // For closed manifold, Euler characteristic should be 2
    // Allow some tolerance for non-manifold cases
    if euler_char != 2 && euler_char != 0 {
        // Log warning (in real implementation, use proper logging)
        eprintln!(
            "Warning: Mesh topology validation - Euler characteristic = {} (expected 2 for closed manifold, 0 for open)",
            euler_char
        );
    }
    
    // Check for boundary edges (edges with only one incident face)
    // This is a simplified check - full implementation would use half-edge structure
    let boundary_edges = count_boundary_edges(mesh);
    if boundary_edges > 0 {
        eprintln!(
            "Warning: Mesh has {} boundary edges (not a closed manifold)",
            boundary_edges
        );
    }
}

/// Count boundary edges (edges with only one incident face)
fn count_boundary_edges(mesh: &Mesh) -> usize {
    use std::collections::HashMap;
    
    // Map: (min_vertex, max_vertex) -> count
    let mut edge_counts: HashMap<(usize, usize), usize> = HashMap::new();
    
    for tri in &mesh.triangles {
        let v0 = tri.indices[0];
        let v1 = tri.indices[1];
        let v2 = tri.indices[2];
        
        // Add three edges
        let edges = [
            (v0.min(v1), v0.max(v1)),
            (v1.min(v2), v1.max(v2)),
            (v2.min(v0), v2.max(v0)),
        ];
        
        for edge in &edges {
            *edge_counts.entry(*edge).or_insert(0) += 1;
        }
    }
    
    // Count edges that appear only once (boundary edges)
    edge_counts.values().filter(|&&count| count == 1).count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Primitive;

    #[test]
    fn test_reconstruct_manifold_mesh() {
        let mesh = Primitive::cube(nalgebra::Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
        let he_mesh = HalfEdgeMesh::from_mesh(&mesh);
        let reconstructed = reconstruct_manifold_mesh(&he_mesh);
        
        assert!(reconstructed.vertex_count() > 0);
        assert!(reconstructed.triangle_count() > 0);
    }

    #[test]
    fn test_weld_vertices() {
        let vertices = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1e-10, 0.0, 0.0), // Very close to first
            Point3::new(1.0, 0.0, 0.0),
        ];
        
        let (vertex_map, welded) = weld_vertices(&vertices);
        
        // First two should be welded together
        assert_eq!(vertex_map[0], vertex_map[1]);
        assert!(welded.len() <= 2); // Should have at most 2 unique vertices
    }
}

