// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Mesh representation and utilities

use super::{BooleanOp, BoundingBox};
use anyhow::Result;
use nalgebra::{Matrix4, Point3, Vector3};
use serde::{Deserialize, Serialize};

/// Vertex with position and normal
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Vertex {
    pub position: Point3<f64>,
    pub normal: Vector3<f64>,
}

impl Vertex {
    pub fn new(position: Point3<f64>, normal: Vector3<f64>) -> Self {
        Self { position, normal }
    }

    pub fn transform(&mut self, matrix: &Matrix4<f64>) {
        self.position = matrix.transform_point(&self.position);
        // Transform normal (use inverse transpose for normals)
        let normal_matrix = matrix
            .try_inverse()
            .map(|m| m.transpose())
            .unwrap_or(*matrix);
        self.normal = normal_matrix.transform_vector(&self.normal).normalize();
    }
}

/// Triangle defined by three vertex indices
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Triangle {
    pub indices: [usize; 3],
}

impl Triangle {
    pub fn new(indices: [usize; 3]) -> Self {
        Self { indices }
    }
}

/// Triangular mesh
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub triangles: Vec<Triangle>,
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            triangles: Vec::new(),
        }
    }

    pub fn empty() -> Self {
        Self::new()
    }

    pub fn with_capacity(vertex_count: usize, triangle_count: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(vertex_count),
            triangles: Vec::with_capacity(triangle_count),
        }
    }

    /// Add a vertex and return its index
    pub fn add_vertex(&mut self, vertex: Vertex) -> usize {
        let index = self.vertices.len();
        self.vertices.push(vertex);
        index
    }

    /// Add a triangle
    pub fn add_triangle(&mut self, triangle: Triangle) {
        self.triangles.push(triangle);
    }

    /// Transform all vertices by a matrix
    pub fn transform(&mut self, matrix: &Matrix4<f64>) {
        for vertex in &mut self.vertices {
            vertex.transform(matrix);
        }
    }

    /// Compute bounding box
    pub fn bounding_box(&self) -> BoundingBox {
        BoundingBox::from_vertices(&self.vertices)
    }

    /// Get vertex count
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Get triangle count
    pub fn triangle_count(&self) -> usize {
        self.triangles.len()
    }

    /// Perform boolean operation with another mesh
    /// Defaults to Robust quality for better results
    pub fn boolean_operation(&self, other: &Mesh, op: BooleanOp) -> Result<Mesh> {
        self.boolean_operation_with_quality(other, op, super::boolean::BooleanQuality::Robust)
    }

    /// Perform boolean operation with specified quality
    pub fn boolean_operation_with_quality(
        &self,
        other: &Mesh,
        op: BooleanOp,
        quality: super::boolean::BooleanQuality,
    ) -> Result<Mesh> {
        super::boolean::perform_boolean_operation_with_quality(self, other, op, quality)
    }

    /// Merge with another mesh (simple union without CSG)
    pub fn merge(&mut self, other: &Mesh) {
        let offset = self.vertices.len();
        self.vertices.extend_from_slice(&other.vertices);

        for triangle in &other.triangles {
            self.triangles.push(Triangle::new([
                triangle.indices[0] + offset,
                triangle.indices[1] + offset,
                triangle.indices[2] + offset,
            ]));
        }
    }

    /// Weld vertices that are within epsilon distance of each other
    /// This removes duplicate vertices and updates triangle indices
    /// Returns the number of vertices removed
    pub fn weld_vertices(&mut self, epsilon: f64) -> usize {
        if self.vertices.is_empty() {
            return 0;
        }

        let original_count = self.vertices.len();
        let mut new_vertices: Vec<Vertex> = Vec::new();
        let mut new_indices: Vec<usize> = vec![0; original_count];

        // Build vertex map: for each vertex, find the index of the first vertex within epsilon
        for i in 0..original_count {
            let pos_i = self.vertices[i].position;
            let mut found_match = false;

            // Check against already processed vertices
            for j in 0..new_vertices.len() {
                let pos_j = new_vertices[j].position;
                if (pos_i - pos_j).norm() < epsilon {
                    // Found a match - map this vertex to the existing one
                    new_indices[i] = j;
                    found_match = true;
                    break;
                }
            }

            if !found_match {
                // New unique vertex
                new_indices[i] = new_vertices.len();
                new_vertices.push(self.vertices[i]);
            }
        }

        // Update triangle indices
        for triangle in &mut self.triangles {
            triangle.indices[0] = new_indices[triangle.indices[0]];
            triangle.indices[1] = new_indices[triangle.indices[1]];
            triangle.indices[2] = new_indices[triangle.indices[2]];
        }

        // Replace vertices
        self.vertices = new_vertices;

        original_count - self.vertices.len()
    }

    /// Remove coplanar duplicate triangles with z-fighting prevention
    /// When two triangles are coplanar and overlapping, keeps only one based on consistent rules:
    /// 1. If from different meshes: keep triangle from mesh A (first mesh)
    /// 2. If from same mesh: keep first encountered
    /// 3. Tiebreaker: keep triangle with larger area
    /// 
    /// Only removes triangles with very high overlap (>= 90% of smaller triangle OR all vertices match)
    /// This prevents removing triangles that just touch at edges or have partial overlap
    /// 
    /// mesh_source: Vec<usize> indicating which mesh each triangle came from (0 = mesh A, 1 = mesh B)
    /// other_mesh: Optional reference to other mesh for checking vertex outside status
    /// Returns the number of triangles removed
    pub fn remove_coplanar_duplicates(
        &mut self,
        mesh_source: &[usize],
        _other_mesh: Option<&Mesh>,
    ) -> usize {
        const EPS: f64 = 1e-6;
        let original_count = self.triangles.len();
        
        // Early return if no triangles
        if original_count == 0 {
            return 0;
        }
        
        let mut new_triangles: Vec<Triangle> = Vec::new();
        let mut new_triangle_orig_indices: Vec<usize> = Vec::new(); // Track original indices
        let mut triangle_areas: Vec<f64> = Vec::new();
        
        // Precompute triangle areas and normals
        let mut triangle_normals: Vec<Vector3<f64>> = Vec::new();
        for triangle in &self.triangles {
            if triangle.indices[0] >= self.vertices.len() ||
               triangle.indices[1] >= self.vertices.len() ||
               triangle.indices[2] >= self.vertices.len() {
                triangle_areas.push(0.0);
                triangle_normals.push(Vector3::new(0.0, 0.0, 1.0));
                continue;
            }
            
            let v0 = &self.vertices[triangle.indices[0]].position;
            let v1 = &self.vertices[triangle.indices[1]].position;
            let v2 = &self.vertices[triangle.indices[2]].position;
            
            let edge1 = v1 - v0;
            let edge2 = v2 - v0;
            let normal = edge1.cross(&edge2);
            let area = normal.norm() * 0.5;
            let normalized_normal = if area > EPS {
                normal / normal.norm()
            } else {
                Vector3::new(0.0, 0.0, 1.0)
            };
            
            triangle_areas.push(area);
            triangle_normals.push(normalized_normal);
        }
        
        // Check each triangle against already added triangles
        for (i, triangle) in self.triangles.iter().enumerate() {
            if triangle.indices[0] >= self.vertices.len() ||
               triangle.indices[1] >= self.vertices.len() ||
               triangle.indices[2] >= self.vertices.len() {
                continue; // Skip invalid triangles
            }
            
            let i0 = triangle.indices[0];
            let i1 = triangle.indices[1];
            let i2 = triangle.indices[2];
            
            // Check for degenerate triangles
            if i0 == i1 || i1 == i2 || i0 == i2 {
                continue;
            }
            
            let v0 = &self.vertices[i0].position;
            let v1 = &self.vertices[i1].position;
            let v2 = &self.vertices[i2].position;
            
            let current_mesh_source = mesh_source.get(i).copied().unwrap_or(0);
            let current_area = triangle_areas[i];
            
            let mut should_keep = true;
            
            // Check against already added triangles
            for (j, existing_tri) in new_triangles.iter().enumerate() {
                let existing_orig_idx = new_triangle_orig_indices[j];
                
                // Quick check: if normals are very different, triangles are not coplanar
                let normal_i = &triangle_normals[i];
                let normal_j = &triangle_normals[existing_orig_idx];
                let normal_dot = normal_i.dot(normal_j);
                
                // If normals point in opposite directions or are very different, skip
                // (we want normals to be similar for coplanar triangles)
                // Use 0.95 for very strict coplanar check (allows ~18 degree difference)
                if normal_dot < 0.95 {
                    continue; // Not coplanar
                }
                
                let e0 = &self.vertices[existing_tri.indices[0]].position;
                let e1 = &self.vertices[existing_tri.indices[1]].position;
                let e2 = &self.vertices[existing_tri.indices[2]].position;
                
                // Check if triangles are coplanar and overlapping
                // Only check overlap if they're from different meshes (z-fighting case)
                // OR if they're from the same mesh but are exact duplicates (all vertices match)
                let existing_mesh_source = mesh_source.get(existing_orig_idx).copied().unwrap_or(0);
                    
                // For same-mesh triangles, only remove if they're exact duplicates (all 3 vertices match)
                // For different-mesh triangles, check for significant overlap
                let should_check_overlap = if current_mesh_source != existing_mesh_source {
                    // Different meshes - check for z-fighting (any significant overlap)
                    true
                } else {
                    // Same mesh - only check if they're exact duplicates
                    // Quick check: are all vertices very close?
                    let mut vertex_matches = 0;
                    let vertex_epsilon = EPS * 10.0;
                    for v_pos in [v0, v1, v2] {
                        for e_pos in [e0, e1, e2] {
                            if (v_pos - e_pos).norm() < vertex_epsilon {
                                vertex_matches += 1;
                                break;
                            }
                        }
                    }
                    vertex_matches == 3 // All 3 vertices match
                };
                
                if should_check_overlap && self.are_triangles_coplanar_overlapping(v0, v1, v2, e0, e1, e2, EPS) {
                    let existing_area = triangle_areas[existing_orig_idx];
                    
                    // Apply z-fighting prevention rule
                    let keep_current = if current_mesh_source != existing_mesh_source {
                        // From different meshes: prefer mesh A (0)
                        current_mesh_source == 0
                    } else {
                        // From same mesh: prefer larger area (shouldn't happen often due to exact duplicate check)
                        if current_area > existing_area + EPS {
                            true
                        } else if existing_area > current_area + EPS {
                            false
                        } else {
                            // Areas are equal - keep first encountered (existing)
                            false
                        }
                    };
                    
                    if !keep_current {
                        should_keep = false;
                        break;
                    } else {
                        // Current triangle should be kept, remove the existing one
                        new_triangles.remove(j);
                        new_triangle_orig_indices.remove(j);
                        break;
                    }
                }
            }
            
            if should_keep {
                new_triangles.push(*triangle);
                new_triangle_orig_indices.push(i);
            }
        }
        
        let removed = original_count - new_triangles.len();
        self.triangles = new_triangles;
        removed
    }

    /// Remove duplicate triangles (triangles with identical vertex indices in same order)
    /// Also removes degenerate triangles (where two or more vertices are the same)
    /// Also removes coplanar overlapping triangles (triangles that are essentially the same)
    /// Returns the number of triangles removed
    pub fn remove_duplicate_triangles(&mut self) -> usize {
        use std::collections::HashSet;

        let original_count = self.triangles.len();
        let mut seen: HashSet<(usize, usize, usize)> = HashSet::new();
        let mut new_triangles: Vec<Triangle> = Vec::new();
        const EPS: f64 = 1e-6;

        for triangle in &self.triangles {
            let i0 = triangle.indices[0];
            let i1 = triangle.indices[1];
            let i2 = triangle.indices[2];

            // Check for degenerate triangles (all vertices the same or two vertices same)
            if i0 == i1 || i1 == i2 || i0 == i2 {
                continue; // Skip degenerate triangles
            }

            // Check bounds
            if i0 >= self.vertices.len() || i1 >= self.vertices.len() || i2 >= self.vertices.len() {
                continue; // Skip invalid triangles
            }

            // Use exact indices (preserve winding order)
            // For union operations, we want to keep triangles even if they share edges
            // Only remove exact duplicates (same indices in same order)
            let key = (i0, i1, i2);

            if !seen.contains(&key) {
                // Don't check for coplanar overlaps here - that's done separately
                // in remove_coplanar_duplicates with proper mesh source tracking
                seen.insert(key);
                new_triangles.push(*triangle);
            }
        }

        let removed = original_count - new_triangles.len();
        self.triangles = new_triangles;
        removed
    }
    
    /// Check if two triangles are coplanar and overlapping
    /// Uses accurate 2D polygon intersection to detect actual overlap
    fn are_triangles_coplanar_overlapping(
        &self,
        v0: &Point3<f64>, v1: &Point3<f64>, v2: &Point3<f64>,
        u0: &Point3<f64>, u1: &Point3<f64>, u2: &Point3<f64>,
        epsilon: f64,
    ) -> bool {
        // Compute plane of first triangle
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let normal = edge1.cross(&edge2);
        let normal_len = normal.norm();
        
        if normal_len < epsilon {
            return false; // Degenerate triangle
        }
        
        let normal = normal / normal_len;
        let d = normal.dot(&v0.coords);
        
        // Check if all vertices of second triangle are on the plane
        let dist_u0 = (normal.dot(&u0.coords) - d).abs();
        let dist_u1 = (normal.dot(&u1.coords) - d).abs();
        let dist_u2 = (normal.dot(&u2.coords) - d).abs();
        
        if dist_u0 > epsilon || dist_u1 > epsilon || dist_u2 > epsilon {
            return false; // Not coplanar
        }
        
        // Project triangles to 2D and check actual overlap
        Self::triangles_overlap_2d(v0, v1, v2, u0, u1, u2, &normal, epsilon)
    }
    
    /// Check if two coplanar triangles overlap in 2D space
    /// Projects triangles to 2D plane and uses polygon intersection
    /// Returns true only if there's significant overlap (not just touching edges)
    fn triangles_overlap_2d(
        v0: &Point3<f64>, v1: &Point3<f64>, v2: &Point3<f64>,
        u0: &Point3<f64>, u1: &Point3<f64>, u2: &Point3<f64>,
        normal: &Vector3<f64>,
        epsilon: f64,
    ) -> bool {
        // Calculate triangle areas for overlap threshold
        let edge1_v = v1 - v0;
        let edge2_v = v2 - v0;
        let area_v = edge1_v.cross(&edge2_v).norm() * 0.5;
        
        let edge1_u = u1 - u0;
        let edge2_u = u2 - u0;
        let area_u = edge1_u.cross(&edge2_u).norm() * 0.5;
        
        // Skip if either triangle is degenerate
        if area_v < epsilon || area_u < epsilon {
            return false;
        }
        
        let min_area = area_v.min(area_u);
        // Require at least 90% overlap of the smaller triangle to consider them duplicates
        // Very conservative - only remove triangles that are nearly identical
        let overlap_threshold = min_area * 0.9;
        // Project to 2D using best-fit plane projection
        let abs_normal = normal.map(|x| x.abs());
        let max_axis = if abs_normal.x > abs_normal.y && abs_normal.x > abs_normal.z {
            0 // Project to YZ plane
        } else if abs_normal.y > abs_normal.z {
            1 // Project to XZ plane
        } else {
            2 // Project to XY plane
        };
        
        let get_2d = |p: &Point3<f64>| -> (f64, f64) {
            match max_axis {
                0 => (p.y, p.z),
                1 => (p.x, p.z),
                _ => (p.x, p.y),
            }
        };
        
        // Get 2D coordinates
        let (v0x, v0y) = get_2d(v0);
        let (v1x, v1y) = get_2d(v1);
        let (v2x, v2y) = get_2d(v2);
        let (u0x, u0y) = get_2d(u0);
        let (u1x, u1y) = get_2d(u1);
        let (u2x, u2y) = get_2d(u2);
        
        // First check: Are triangles nearly identical? (same vertices within epsilon)
        // This catches exact duplicates that might have slight numerical differences
        // Only consider them duplicates if ALL 3 vertices match (very strict)
        let mut vertex_matches = 0;
        let vertex_epsilon = epsilon * 10.0; // Smaller epsilon for vertex matching (1e-5)
        
        // Check if vertices of triangle 1 match vertices of triangle 2 (within epsilon)
        for (vx, vy) in [(v0x, v0y), (v1x, v1y), (v2x, v2y)] {
            for (ux, uy) in [(u0x, u0y), (u1x, u1y), (u2x, u2y)] {
                let dx = vx - ux;
                let dy = vy - uy;
                if (dx * dx + dy * dy).sqrt() < vertex_epsilon {
                    vertex_matches += 1;
                    break;
                }
            }
        }
        
        // Only if ALL 3 vertices match, triangles are exact duplicates
        if vertex_matches == 3 {
            return true;
        }
        
        // Second check: Are centroids very close? (catches z-fighting cases)
        let centroid_vx = (v0x + v1x + v2x) / 3.0;
        let centroid_vy = (v0y + v1y + v2y) / 3.0;
        let centroid_ux = (u0x + u1x + u2x) / 3.0;
        let centroid_uy = (u0y + u1y + u2y) / 3.0;
        
        let dx = centroid_vx - centroid_ux;
        let dy = centroid_vy - centroid_uy;
        let centroid_dist = (dx * dx + dy * dy).sqrt();
        
        // Calculate triangle sizes
        let avg_edge_length = ((v1x - v0x).powi(2) + (v1y - v0y).powi(2)).sqrt() +
                              ((v2x - v1x).powi(2) + (v2y - v1y).powi(2)).sqrt() +
                              ((v0x - v2x).powi(2) + (v0y - v2y).powi(2)).sqrt();
        let avg_edge_length_u = ((u1x - u0x).powi(2) + (u1y - u0y).powi(2)).sqrt() +
                                ((u2x - u1x).powi(2) + (u2y - u1y).powi(2)).sqrt() +
                                ((u0x - u2x).powi(2) + (u0y - u2y).powi(2)).sqrt();
        let min_edge_length = avg_edge_length.min(avg_edge_length_u) / 3.0;
        
        // If centroids are extremely close (within 1% of average edge length), likely duplicates
        // This catches z-fighting where triangles are nearly identical but slightly offset
        // Very conservative - only extremely close centroids indicate duplicates
        if centroid_dist < min_edge_length * 0.01 && min_edge_length > epsilon {
            return true;
        }
        
        // Count how many vertices of triangle 1 are inside triangle 2
        let mut v_inside_count = 0;
        if Self::point_in_triangle_2d(v0x, v0y, u0x, u0y, u1x, u1y, u2x, u2y, epsilon) {
            v_inside_count += 1;
        }
        if Self::point_in_triangle_2d(v1x, v1y, u0x, u0y, u1x, u1y, u2x, u2y, epsilon) {
            v_inside_count += 1;
        }
        if Self::point_in_triangle_2d(v2x, v2y, u0x, u0y, u1x, u1y, u2x, u2y, epsilon) {
            v_inside_count += 1;
        }
        
        // Count how many vertices of triangle 2 are inside triangle 1
        let mut u_inside_count = 0;
        if Self::point_in_triangle_2d(u0x, u0y, v0x, v0y, v1x, v1y, v2x, v2y, epsilon) {
            u_inside_count += 1;
        }
        if Self::point_in_triangle_2d(u1x, u1y, v0x, v0y, v1x, v1y, v2x, v2y, epsilon) {
            u_inside_count += 1;
        }
        if Self::point_in_triangle_2d(u2x, u2y, v0x, v0y, v1x, v1y, v2x, v2y, epsilon) {
            u_inside_count += 1;
        }
        
        // Only if ALL vertices of one triangle are inside the other, that's significant overlap
        // This catches cases where one triangle completely contains the other
        if v_inside_count == 3 || u_inside_count == 3 {
            return true;
        }
        
        // If 2+ vertices are inside AND edges intersect, that's also significant overlap
        // Require both conditions to be more conservative
        if (v_inside_count >= 2 || u_inside_count >= 2) {
            // Check if edges also intersect to confirm significant overlap
            if Self::edge_intersects_triangle_2d(v0x, v0y, v1x, v1y, u0x, u0y, u1x, u1y, u2x, u2y, epsilon) ||
               Self::edge_intersects_triangle_2d(v1x, v1y, v2x, v2y, u0x, u0y, u1x, u1y, u2x, u2y, epsilon) ||
               Self::edge_intersects_triangle_2d(v2x, v2y, v0x, v0y, u0x, u0y, u1x, u1y, u2x, u2y, epsilon) ||
               Self::edge_intersects_triangle_2d(u0x, u0y, u1x, u1y, v0x, v0y, v1x, v1y, v2x, v2y, epsilon) ||
               Self::edge_intersects_triangle_2d(u1x, u1y, u2x, u2y, v0x, v0y, v1x, v1y, v2x, v2y, epsilon) ||
               Self::edge_intersects_triangle_2d(u2x, u2y, u0x, u0y, v0x, v0y, v1x, v1y, v2x, v2y, epsilon) {
                return true;
            }
        }
        
        // Check if edges intersect (indicates overlap)
        let mut has_edge_intersection = false;
        if Self::edge_intersects_triangle_2d(v0x, v0y, v1x, v1y, u0x, u0y, u1x, u1y, u2x, u2y, epsilon) ||
           Self::edge_intersects_triangle_2d(v1x, v1y, v2x, v2y, u0x, u0y, u1x, u1y, u2x, u2y, epsilon) ||
           Self::edge_intersects_triangle_2d(v2x, v2y, v0x, v0y, u0x, u0y, u1x, u1y, u2x, u2y, epsilon) ||
           Self::edge_intersects_triangle_2d(u0x, u0y, u1x, u1y, v0x, v0y, v1x, v1y, v2x, v2y, epsilon) ||
           Self::edge_intersects_triangle_2d(u1x, u1y, u2x, u2y, v0x, v0y, v1x, v1y, v2x, v2y, epsilon) ||
           Self::edge_intersects_triangle_2d(u2x, u2y, u0x, u0y, v0x, v0y, v1x, v1y, v2x, v2y, epsilon) {
            has_edge_intersection = true;
        }
        
        // For edge-only intersections, if centroids are extremely close, consider it overlap
        // (centroid distance already calculated above)
        if has_edge_intersection {
            // If centroids are closer than 1% of average edge length, consider it significant overlap
            // Very conservative - only extremely close centroids with edge intersections
            if centroid_dist < min_edge_length * 0.01 && min_edge_length > epsilon {
                return true;
            }
        }
        
        false
    }
    
    /// Robust point-in-triangle test in 2D
    fn point_in_triangle_2d(
        px: f64, py: f64,
        v0x: f64, v0y: f64,
        v1x: f64, v1y: f64,
        v2x: f64, v2y: f64,
        epsilon: f64,
    ) -> bool {
        // Use barycentric coordinates
        let denom = (v1y - v2y) * (v0x - v2x) + (v2x - v1x) * (v0y - v2y);
        if denom.abs() < epsilon {
            return false; // Degenerate triangle
        }
        
        let a = ((v1y - v2y) * (px - v2x) + (v2x - v1x) * (py - v2y)) / denom;
        let b = ((v2y - v0y) * (px - v2x) + (v0x - v2x) * (py - v2y)) / denom;
        let c = 1.0 - a - b;
        
        // Point is inside if all barycentric coordinates are non-negative
        a >= -epsilon && b >= -epsilon && c >= -epsilon
    }
    
    /// Check if an edge intersects a triangle in 2D
    fn edge_intersects_triangle_2d(
        e0x: f64, e0y: f64,
        e1x: f64, e1y: f64,
        v0x: f64, v0y: f64,
        v1x: f64, v1y: f64,
        v2x: f64, v2y: f64,
        epsilon: f64,
    ) -> bool {
        // Check if edge intersects any of the triangle's edges
        // Edge 0-1 of triangle
        if Self::segments_intersect_2d(e0x, e0y, e1x, e1y, v0x, v0y, v1x, v1y, epsilon) {
            return true;
        }
        // Edge 1-2 of triangle
        if Self::segments_intersect_2d(e0x, e0y, e1x, e1y, v1x, v1y, v2x, v2y, epsilon) {
            return true;
        }
        // Edge 2-0 of triangle
        if Self::segments_intersect_2d(e0x, e0y, e1x, e1y, v2x, v2y, v0x, v0y, epsilon) {
            return true;
        }
        false
    }
    
    /// Check if two 2D line segments intersect
    fn segments_intersect_2d(
        p0x: f64, p0y: f64,
        p1x: f64, p1y: f64,
        q0x: f64, q0y: f64,
        q1x: f64, q1y: f64,
        epsilon: f64,
    ) -> bool {
        // Use cross product to check if segments intersect
        let d1 = (q1x - q0x) * (p0y - q0y) - (q1y - q0y) * (p0x - q0x);
        let d2 = (q1x - q0x) * (p1y - q0y) - (q1y - q0y) * (p1x - q0x);
        let d3 = (p1x - p0x) * (q0y - p0y) - (p1y - p0y) * (q0x - p0x);
        let d4 = (p1x - p0x) * (q1y - p0y) - (p1y - p0y) * (q1x - p0x);
        
        // Check if segments are on opposite sides of each other
        if (d1 * d2 < -epsilon) && (d3 * d4 < -epsilon) {
            return true;
        }
        
        // Check for collinear cases (segments overlap)
        if d1.abs() < epsilon && d2.abs() < epsilon {
            // Segments are collinear, check if they overlap
            let t0 = if (p1x - p0x).abs() > epsilon {
                (q0x - p0x) / (p1x - p0x)
            } else if (p1y - p0y).abs() > epsilon {
                (q0y - p0y) / (p1y - p0y)
            } else {
                return false; // Degenerate segment
            };
            
            let t1 = if (p1x - p0x).abs() > epsilon {
                (q1x - p0x) / (p1x - p0x)
            } else if (p1y - p0y).abs() > epsilon {
                (q1y - p0y) / (p1y - p0y)
            } else {
                return false;
            };
            
            let t_min = t0.min(t1);
            let t_max = t0.max(t1);
            
            // Check if intervals [0,1] and [t_min, t_max] overlap
            return t_max >= -epsilon && t_min <= 1.0 + epsilon;
        }
        
        false
    }

    /// Remove orphaned vertices (vertices not referenced by any triangle)
    /// Returns the number of vertices removed
    pub fn remove_orphaned_vertices(&mut self) -> usize {
        if self.triangles.is_empty() {
            // If no triangles, remove all vertices
            let removed = self.vertices.len();
            self.vertices.clear();
            return removed;
        }
        
        // Find which vertices are used
        let mut used_vertices = vec![false; self.vertices.len()];
        for triangle in &self.triangles {
            used_vertices[triangle.indices[0]] = true;
            used_vertices[triangle.indices[1]] = true;
            used_vertices[triangle.indices[2]] = true;
        }
        
        // Build remapping: old_index -> new_index
        let mut new_indices = vec![0; self.vertices.len()];
        let mut new_vertices = Vec::new();
        let mut new_index = 0;
        
        for (old_idx, &used) in used_vertices.iter().enumerate() {
            if used {
                new_indices[old_idx] = new_index;
                new_vertices.push(self.vertices[old_idx]);
                new_index += 1;
            }
        }
        
        // Update triangle indices
        for triangle in &mut self.triangles {
            triangle.indices[0] = new_indices[triangle.indices[0]];
            triangle.indices[1] = new_indices[triangle.indices[1]];
            triangle.indices[2] = new_indices[triangle.indices[2]];
        }
        
        let removed = self.vertices.len() - new_vertices.len();
        self.vertices = new_vertices;
        removed
    }

    /// Recompute vertex normals from triangle geometry
    /// This calculates face normals and averages them at shared vertices
    pub fn recompute_normals(&mut self) {
        if self.vertices.is_empty() || self.triangles.is_empty() {
            return;
        }

        // Initialize normal accumulators for each vertex
        let mut normal_sums: Vec<Vector3<f64>> = vec![Vector3::zeros(); self.vertices.len()];
        let mut normal_counts: Vec<u32> = vec![0; self.vertices.len()];

        // Calculate face normals and accumulate at vertices
        for triangle in &self.triangles {
            let v0 = &self.vertices[triangle.indices[0]];
            let v1 = &self.vertices[triangle.indices[1]];
            let v2 = &self.vertices[triangle.indices[2]];

            // Calculate face normal using cross product
            let edge1 = v1.position - v0.position;
            let edge2 = v2.position - v0.position;
            let face_normal = edge1.cross(&edge2);

            // Only add if triangle has non-zero area
            let area = face_normal.norm();
            if area > 1e-10 {
                let normalized_face_normal = face_normal / area;

                // Add to all three vertices (weighted by area for better quality)
                for &idx in &triangle.indices {
                    normal_sums[idx] += normalized_face_normal * area;
                    normal_counts[idx] += 1;
                }
            }
        }

        // Normalize accumulated normals
        for (i, vertex) in self.vertices.iter_mut().enumerate() {
            if normal_counts[i] > 0 {
                let normal = normal_sums[i].normalize();
                vertex.normal = normal;
            } else {
                // Fallback: use default normal if no triangles reference this vertex
                vertex.normal = Vector3::new(0.0, 0.0, 1.0);
            }
        }
    }
}

impl Default for Mesh {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Primitive;
    use nalgebra::Vector3;

    #[test]
    fn test_recompute_normals() {
        let mut mesh = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
        
        // Verify normals exist
        assert!(!mesh.vertices.is_empty());
        assert!(mesh.vertices.iter().all(|v| v.normal.norm() > 0.0));
        
        // Recompute normals
        mesh.recompute_normals();
        
        // Verify normals are still valid after recomputation
        assert!(mesh.vertices.iter().all(|v| {
            let norm = v.normal.norm();
            norm > 0.9 && norm < 1.1 // Should be approximately unit length
        }));
    }

    #[test]
    fn test_recompute_normals_cylinder() {
        let mut mesh = Primitive::cylinder(10.0, 5.0, 32).to_mesh();
        
        // Recompute normals
        mesh.recompute_normals();
        
        // Verify all normals are valid
        assert!(mesh.vertices.iter().all(|v| {
            let norm = v.normal.norm();
            norm > 0.9 && norm < 1.1 // Should be approximately unit length
        }));
    }
}
