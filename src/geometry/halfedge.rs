// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Half-edge mesh representation for robust CSG operations
//! This provides topological connectivity information needed for robust mesh operations

use super::{Mesh, Triangle, Vertex};
use nalgebra::{Point3, Vector3};

/// Half-edge in a half-edge mesh
/// Each edge has two half-edges, one for each direction
#[derive(Debug, Clone, Copy)]
pub struct HalfEdge {
    /// Next half-edge in the same face (counter-clockwise)
    pub next: usize,
    /// Previous half-edge in the same face
    pub prev: usize,
    /// Twin half-edge (opposite direction, belongs to adjacent face)
    pub twin: Option<usize>,
    /// Vertex this half-edge points to
    pub vertex: usize,
    /// Face this half-edge belongs to
    pub face: usize,
}

/// Edge connecting two vertices
#[derive(Debug, Clone, Copy)]
pub struct Edge {
    /// First half-edge (direction from vertex_a to vertex_b)
    pub half_edge_a: usize,
    /// Second half-edge (direction from vertex_b to vertex_a)
    pub half_edge_b: Option<usize>,
}

/// Half-edge mesh with full topological connectivity
#[derive(Debug, Clone)]
pub struct HalfEdgeMesh {
    /// Vertex positions
    pub vertices: Vec<Point3<f64>>,
    /// Face indices (triangles)
    pub faces: Vec<[u32; 3]>,
    /// Half-edges
    pub half_edges: Vec<HalfEdge>,
    /// Edges
    pub edges: Vec<Edge>,
}

impl HalfEdgeMesh {
    /// Create an empty half-edge mesh
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            faces: Vec::new(),
            half_edges: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Convert from standard Mesh to HalfEdgeMesh
    pub fn from_mesh(mesh: &Mesh) -> Self {
        let mut he_mesh = Self::new();
        
        // Copy vertices
        he_mesh.vertices = mesh.vertices.iter().map(|v| v.position).collect();
        
        // Copy faces
        he_mesh.faces = mesh.triangles.iter().map(|t| [
            t.indices[0] as u32,
            t.indices[1] as u32,
            t.indices[2] as u32,
        ]).collect();
        
        // Build half-edge structure
        he_mesh.build_topology();
        
        he_mesh
    }

    /// Convert from HalfEdgeMesh to standard Mesh
    pub fn to_mesh(&self) -> Mesh {
        let mut mesh = Mesh::new();
        
        // Add vertices (compute normals later)
        let vertex_indices: Vec<usize> = (0..self.vertices.len())
            .map(|i| {
                let normal = Vector3::new(0.0, 0.0, 1.0); // Placeholder, will be recomputed
                mesh.add_vertex(Vertex::new(self.vertices[i], normal))
            })
            .collect();
        
        // Add triangles
        for face in &self.faces {
            mesh.add_triangle(Triangle::new([
                vertex_indices[face[0] as usize],
                vertex_indices[face[1] as usize],
                vertex_indices[face[2] as usize],
            ]));
        }
        
        // Recompute normals
        mesh.recompute_normals();
        mesh
    }

    /// Build half-edge topology from faces
    fn build_topology(&mut self) {
        self.half_edges.clear();
        self.edges.clear();
        
        // Create half-edges for each face
        for (face_idx, face) in self.faces.iter().enumerate() {
            let v0 = face[0] as usize;
            let v1 = face[1] as usize;
            let v2 = face[2] as usize;
            
            // Create three half-edges for this triangle
            let he0_idx = self.half_edges.len();
            let he1_idx = he0_idx + 1;
            let he2_idx = he0_idx + 2;
            
            // Half-edge 0: v0 -> v1
            self.half_edges.push(HalfEdge {
                next: he1_idx,
                prev: he2_idx,
                twin: None, // Will be set when we find matching edge
                vertex: v1,
                face: face_idx,
            });
            
            // Half-edge 1: v1 -> v2
            self.half_edges.push(HalfEdge {
                next: he2_idx,
                prev: he0_idx,
                twin: None,
                vertex: v2,
                face: face_idx,
            });
            
            // Half-edge 2: v2 -> v0
            self.half_edges.push(HalfEdge {
                next: he0_idx,
                prev: he1_idx,
                twin: None,
                vertex: v0,
                face: face_idx,
            });
        }
        
        // Build edge map and set twin pointers
        self.build_edge_map();
    }

    /// Build edge map and connect twin half-edges
    fn build_edge_map(&mut self) {
        use std::collections::HashMap;
        
        // Map: (min_vertex, max_vertex) -> half_edge_index
        let mut edge_map: HashMap<(usize, usize), usize> = HashMap::new();
        
        // First pass: collect edge pairs
        let mut twin_pairs: Vec<(usize, usize)> = Vec::new();
        
        for (he_idx, he) in self.half_edges.iter().enumerate() {
            let v_from = self.half_edges[he.prev].vertex;
            let v_to = he.vertex;
            
            // Use canonical edge representation (min, max)
            let (v_min, v_max) = if v_from < v_to {
                (v_from, v_to)
            } else {
                (v_to, v_from)
            };
            
            let edge_key = (v_min, v_max);
            
            if let Some(&other_he_idx) = edge_map.get(&edge_key) {
                // Found matching edge - record twin pair
                twin_pairs.push((he_idx, other_he_idx));
            } else {
                // First time seeing this edge
                edge_map.insert(edge_key, he_idx);
            }
        }
        
        // Second pass: set twin pointers
        for (he_idx, other_he_idx) in twin_pairs {
            self.half_edges[he_idx].twin = Some(other_he_idx);
            self.half_edges[other_he_idx].twin = Some(he_idx);
        }
        
        // Build edges list
        for (_, &he_idx) in &edge_map {
            let twin = self.half_edges[he_idx].twin;
            self.edges.push(Edge {
                half_edge_a: he_idx,
                half_edge_b: twin,
            });
        }
    }

    /// Split an edge at a given point
    /// Returns the index of the new vertex
    pub fn split_edge(&mut self, edge_idx: usize, point: Point3<f64>) -> usize {
        let edge = &self.edges[edge_idx];
        let he = &self.half_edges[edge.half_edge_a];
        
        // Get vertices of the edge
        let v_from = self.half_edges[he.prev].vertex;
        let v_to = he.vertex;
        
        // Add new vertex
        let new_vertex_idx = self.vertices.len();
        self.vertices.push(point);
        
        // TODO: Split the edge and update topology
        // This is a complex operation that requires:
        // 1. Creating new half-edges
        // 2. Updating face references
        // 3. Updating edge references
        
        new_vertex_idx
    }

    /// Split a face at a given point (add vertex to face interior)
    /// Returns the index of the new vertex
    pub fn split_face(&mut self, face_idx: usize, point: Point3<f64>) -> usize {
        // Add new vertex
        let new_vertex_idx = self.vertices.len();
        self.vertices.push(point);
        
        // TODO: Split the face into three triangles
        // This requires:
        // 1. Creating new faces
        // 2. Creating new half-edges
        // 3. Updating topology
        
        new_vertex_idx
    }

    /// Get vertex count
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Get face count
    pub fn face_count(&self) -> usize {
        self.faces.len()
    }

    /// Get edge count
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

impl Default for HalfEdgeMesh {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Primitive;

    #[test]
    fn test_halfedge_from_mesh() {
        let mesh = Primitive::cube(nalgebra::Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
        let he_mesh = HalfEdgeMesh::from_mesh(&mesh);
        
        assert!(he_mesh.vertex_count() > 0);
        assert!(he_mesh.face_count() > 0);
        assert!(he_mesh.edge_count() > 0);
    }

    #[test]
    fn test_halfedge_to_mesh() {
        let mesh = Primitive::cube(nalgebra::Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
        let he_mesh = HalfEdgeMesh::from_mesh(&mesh);
        let converted_mesh = he_mesh.to_mesh();
        
        assert_eq!(converted_mesh.vertex_count(), mesh.vertex_count());
        assert_eq!(converted_mesh.triangle_count(), mesh.triangle_count());
    }
}

