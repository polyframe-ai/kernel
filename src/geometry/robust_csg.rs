// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Robust CSG operations using mesh-to-mesh operations
//! This provides a fallback for cases where BSP tree CSG fails

use super::{Mesh, Triangle};
use anyhow::Result;
use nalgebra::{Point3, Vector3};

/// Perform robust CSG union - simply merge meshes
pub fn robust_union(a: &Mesh, b: &Mesh) -> Result<Mesh> {
    let mut result = a.clone();
    result.merge(b);
    Ok(result)
}

/// Perform robust CSG difference using mesh-based approach
/// This uses a winding-number based classification that's more robust than BSP for curved surfaces
pub fn robust_difference(a: &Mesh, b: &Mesh) -> Result<Mesh> {
    // Check if meshes are empty
    if a.vertices.is_empty() {
        return Ok(Mesh::empty());
    }
    if b.vertices.is_empty() {
        return Ok(a.clone());
    }

    // Strategy: Use a robustness check to decide algorithm
    // 1. Simple planar cases: use fast BSP
    // 2. Complex curved surfaces: use winding number approach

    let complexity_score = estimate_complexity(a, b);

    if complexity_score > 0.5 {
        // High complexity - use winding number approach
        mesh_difference_winding(a, b)
    } else {
        // Low complexity - try BSP first
        match super::csg::csg_difference_bsp_internal(a, b) {
            Ok(result) if result.vertex_count() > 0 => Ok(result),
            _ => {
                // BSP failed, fall back to winding number
                mesh_difference_winding(a, b)
            }
        }
    }
}

/// Estimate complexity of CSG operation (0.0 = simple, 1.0 = very complex)
fn estimate_complexity(a: &Mesh, b: &Mesh) -> f32 {
    let mut score: f32 = 0.0;

    // Factor 1: Mesh sizes
    if a.triangle_count() > 100 || b.triangle_count() > 100 {
        score += 0.3;
    }

    // Factor 2: Curved surfaces
    if has_curved_surfaces(a) {
        score += 0.2;
    }
    if has_curved_surfaces(b) {
        score += 0.3;
    }

    // Factor 3: Both have curved surfaces
    if has_curved_surfaces(a) && has_curved_surfaces(b) {
        score += 0.3;
    }

    score.min(1.0)
}

/// Mesh difference using winding number for robustness
fn mesh_difference_winding(a: &Mesh, b: &Mesh) -> Result<Mesh> {
    // Simplified winding number approach:
    // Keep triangles from A that are outside B
    // Add inverted triangles from B that are inside A

    let mut result = Mesh::new();

    // Add triangles from A that are mostly outside B
    for tri in &a.triangles {
        let v0 = &a.vertices[tri.indices[0]];
        let v1 = &a.vertices[tri.indices[1]];
        let v2 = &a.vertices[tri.indices[2]];

        // Check if triangle center is outside B
        let center = (v0.position.coords + v1.position.coords + v2.position.coords) / 3.0;
        let center_point = Point3::from(center);

        if !is_point_inside_mesh(&center_point, b) {
            let i0 = result.add_vertex(*v0);
            let i1 = result.add_vertex(*v1);
            let i2 = result.add_vertex(*v2);
            result.add_triangle(Triangle::new([i0, i1, i2]));
        }
    }

    // Add inverted triangles from B that are inside A
    for tri in &b.triangles {
        let v0 = &b.vertices[tri.indices[0]];
        let v1 = &b.vertices[tri.indices[1]];
        let v2 = &b.vertices[tri.indices[2]];

        let center = (v0.position.coords + v1.position.coords + v2.position.coords) / 3.0;
        let center_point = Point3::from(center);

        if is_point_inside_mesh(&center_point, a) {
            // Add inverted triangle (flip winding)
            let mut v0_inv = *v0;
            let mut v1_inv = *v1;
            let mut v2_inv = *v2;
            v0_inv.normal = -v0_inv.normal;
            v1_inv.normal = -v1_inv.normal;
            v2_inv.normal = -v2_inv.normal;

            let i0 = result.add_vertex(v0_inv);
            let i1 = result.add_vertex(v2_inv); // Swap to invert winding
            let i2 = result.add_vertex(v1_inv);
            result.add_triangle(Triangle::new([i0, i1, i2]));
        }
    }

    Ok(result)
}

/// Check if a point is inside a mesh using ray casting
fn is_point_inside_mesh(point: &Point3<f32>, mesh: &Mesh) -> bool {
    // Ray casting algorithm: cast ray in +X direction and count intersections
    let ray_dir = Vector3::new(1.0, 0.0, 0.0);
    let mut intersection_count = 0;

    for tri in &mesh.triangles {
        let v0 = &mesh.vertices[tri.indices[0]].position;
        let v1 = &mesh.vertices[tri.indices[1]].position;
        let v2 = &mesh.vertices[tri.indices[2]].position;

        if ray_intersects_triangle(point, &ray_dir, v0, v1, v2) {
            intersection_count += 1;
        }
    }

    // Odd number of intersections = inside
    intersection_count % 2 == 1
}

/// Test if a ray intersects a triangle using Möller-Trumbore algorithm
fn ray_intersects_triangle(
    origin: &Point3<f32>,
    direction: &Vector3<f32>,
    v0: &Point3<f32>,
    v1: &Point3<f32>,
    v2: &Point3<f32>,
) -> bool {
    const EPSILON: f32 = 0.000001;

    let edge1 = v1 - v0;
    let edge2 = v2 - v0;
    let h = direction.cross(&edge2);
    let a = edge1.dot(&h);

    if a > -EPSILON && a < EPSILON {
        return false; // Ray parallel to triangle
    }

    let f = 1.0 / a;
    let s = origin - v0;
    let u = f * s.dot(&h);

    if u < 0.0 || u > 1.0 {
        return false;
    }

    let q = s.cross(&edge1);
    let v = f * direction.dot(&q);

    if v < 0.0 || u + v > 1.0 {
        return false;
    }

    let t = f * edge2.dot(&q);
    t > EPSILON // Only count forward intersections
}

/// Perform robust CSG intersection
pub fn robust_intersection(a: &Mesh, b: &Mesh) -> Result<Mesh> {
    // For intersection, we need both meshes
    if a.vertices.is_empty() || b.vertices.is_empty() {
        return Ok(Mesh::empty());
    }

    // Use BSP for now
    super::csg::csg_intersection(a, b)
}

/// Detect if a mesh likely has curved surfaces based on vertex normals
fn has_curved_surfaces(mesh: &Mesh) -> bool {
    if mesh.vertices.len() < 10 {
        return false;
    }

    // Strategy: Count approximately how many distinct normal directions exist
    // Cubes have ~6 normals (faces), spheres have many smoothly varying normals

    const EPSILON: f32 = 0.01;
    let mut distinct_normals = vec![mesh.vertices[0].normal];
    let sample_size = mesh.vertices.len().min(100);

    for i in 1..sample_size {
        let normal = &mesh.vertices[i].normal;
        let mut is_new = true;

        for existing in &distinct_normals {
            if (normal - existing).norm() < EPSILON {
                is_new = false;
                break;
            }
        }

        if is_new {
            distinct_normals.push(*normal);
        }

        // Early exit: if we have many distinct normals, it's curved
        if distinct_normals.len() > 12 {
            return true;
        }
    }

    // Cubes have ~6 distinct normals, curved surfaces have many more
    distinct_normals.len() > 12
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Primitive;
    use nalgebra::Vector3;

    #[test]
    fn test_robust_union() {
        let mesh_a = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
        let mesh_b = Primitive::cube(Vector3::new(5.0, 5.0, 5.0), false).to_mesh();

        let result = robust_union(&mesh_a, &mesh_b);
        assert!(result.is_ok());
        let mesh = result.unwrap();
        assert!(mesh.vertex_count() > 0);
    }

    #[test]
    fn test_curved_surface_detection() {
        let sphere = Primitive::sphere(10.0, 16).to_mesh();
        assert!(has_curved_surfaces(&sphere));

        let cube = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
        assert!(!has_curved_surfaces(&cube));
    }
}
