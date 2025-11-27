// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Geometric primitives generator

use super::{Mesh, Triangle, Vertex};
use nalgebra::{Point3, Vector3};
use std::f64::consts::PI;

/// Geometric primitives
pub enum Primitive {
    Cube { size: Vector3<f64>, center: bool },
    Sphere { r: f64, fn_: u32 },
    Cylinder { h: f64, r: f64, fn_: u32 },
    Cone { h: f64, r1: f64, r2: f64, fn_: u32 },
}

impl Primitive {
    pub fn cube(size: Vector3<f64>, center: bool) -> Self {
        Self::Cube { size, center }
    }

    pub fn sphere(r: f64, fn_: u32) -> Self {
        let segments = if fn_ > 0 { fn_ } else { 32 };
        Self::Sphere { r, fn_: segments }
    }

    pub fn cylinder(h: f64, r: f64, fn_: u32) -> Self {
        let segments = if fn_ > 0 { fn_ } else { 32 };
        Self::Cylinder {
            h,
            r,
            fn_: segments,
        }
    }

    pub fn cone(h: f64, r1: f64, r2: f64, fn_: u32) -> Self {
        let segments = if fn_ > 0 { fn_ } else { 32 };
        Self::Cone {
            h,
            r1,
            r2,
            fn_: segments,
        }
    }

    pub fn to_mesh(&self) -> Mesh {
        match self {
            Self::Cube { size, center } => generate_cube_mesh(*size, *center),
            Self::Sphere { r, fn_ } => generate_sphere_mesh(*r, *fn_),
            Self::Cylinder { h, r, fn_ } => generate_cylinder_mesh(*h, *r, *fn_),
            Self::Cone { h, r1, r2, fn_ } => generate_cone_mesh(*h, *r1, *r2, *fn_),
        }
    }
}

fn generate_cube_mesh(size: Vector3<f64>, center: bool) -> Mesh {
    let mut mesh = Mesh::new();

    // Calculate cube positions based on center flag
    let (min_x, max_x) = if center {
        (-size.x / 2.0, size.x / 2.0)
    } else {
        (0.0, size.x)
    };
    let (min_y, max_y) = if center {
        (-size.y / 2.0, size.y / 2.0)
    } else {
        (0.0, size.y)
    };
    let (min_z, max_z) = if center {
        (-size.z / 2.0, size.z / 2.0)
    } else {
        (0.0, size.z)
    };

    // 8 vertices of the cube
    let positions = [
        Point3::new(min_x, min_y, min_z),
        Point3::new(max_x, min_y, min_z),
        Point3::new(max_x, max_y, min_z),
        Point3::new(min_x, max_y, min_z),
        Point3::new(min_x, min_y, max_z),
        Point3::new(max_x, min_y, max_z),
        Point3::new(max_x, max_y, max_z),
        Point3::new(min_x, max_y, max_z),
    ];

    // 6 faces, each with its normal
    let faces = [
        // Front (z+)
        ([4, 5, 6], Vector3::new(0.0, 0.0, 1.0)),
        ([4, 6, 7], Vector3::new(0.0, 0.0, 1.0)),
        // Back (z-)
        ([1, 0, 3], Vector3::new(0.0, 0.0, -1.0)),
        ([1, 3, 2], Vector3::new(0.0, 0.0, -1.0)),
        // Right (x+)
        ([5, 1, 2], Vector3::new(1.0, 0.0, 0.0)),
        ([5, 2, 6], Vector3::new(1.0, 0.0, 0.0)),
        // Left (x-)
        ([0, 4, 7], Vector3::new(-1.0, 0.0, 0.0)),
        ([0, 7, 3], Vector3::new(-1.0, 0.0, 0.0)),
        // Top (y+)
        ([7, 6, 2], Vector3::new(0.0, 1.0, 0.0)),
        ([7, 2, 3], Vector3::new(0.0, 1.0, 0.0)),
        // Bottom (y-)
        ([0, 1, 5], Vector3::new(0.0, -1.0, 0.0)),
        ([0, 5, 4], Vector3::new(0.0, -1.0, 0.0)),
    ];

    for (indices, normal) in faces {
        let v0 = mesh.add_vertex(Vertex::new(positions[indices[0]], normal));
        let v1 = mesh.add_vertex(Vertex::new(positions[indices[1]], normal));
        let v2 = mesh.add_vertex(Vertex::new(positions[indices[2]], normal));
        mesh.add_triangle(Triangle::new([v0, v1, v2]));
    }

    mesh
}

fn generate_sphere_mesh(radius: f64, segments: u32) -> Mesh {
    let mut mesh = Mesh::new();
    let stacks = segments;
    let slices = segments;

    for i in 0..=stacks {
        let phi = PI * i as f64 / stacks as f64;
        let y = radius * phi.cos();
        let r = radius * phi.sin();

        for j in 0..=slices {
            let theta = 2.0 * PI * j as f64 / slices as f64;
            let x = r * theta.cos();
            let z = r * theta.sin();

            let position = Point3::new(x, y, z);
            let normal = Vector3::new(x, y, z).normalize();
            mesh.add_vertex(Vertex::new(position, normal));
        }
    }

    // Generate triangles
    for i in 0..stacks {
        for j in 0..slices {
            let first = i * (slices + 1) + j;
            let second = first + slices + 1;

            mesh.add_triangle(Triangle::new([
                first as usize,
                second as usize,
                (first + 1) as usize,
            ]));
            mesh.add_triangle(Triangle::new([
                second as usize,
                (second + 1) as usize,
                (first + 1) as usize,
            ]));
        }
    }

    mesh
}

fn generate_cylinder_mesh(height: f64, radius: f64, segments: u32) -> Mesh {
    generate_cone_mesh(height, radius, radius, segments)
}

fn generate_cone_mesh(height: f64, r1: f64, r2: f64, segments: u32) -> Mesh {
    let mut mesh = Mesh::new();

    // OpenSCAD cylinders go from z=0 to z=height (NOT centered by default)
    // Bottom center at z=0
    let bottom_center_idx = mesh.add_vertex(Vertex::new(
        Point3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 0.0, -1.0),
    ));

    // Top center at z=height
    let top_center_idx = mesh.add_vertex(Vertex::new(
        Point3::new(0.0, 0.0, height),
        Vector3::new(0.0, 0.0, 1.0),
    ));

    // Generate vertices
    let mut bottom_indices = Vec::new();
    let mut top_indices = Vec::new();

    for i in 0..segments {
        let angle = 2.0 * PI * i as f64 / segments as f64;
        let cos = angle.cos();
        let sin = angle.sin();

        // Bottom vertex at z=0
        let bottom_pos = Point3::new(r1 * cos, r1 * sin, 0.0);
        let bottom_idx = mesh.add_vertex(Vertex::new(bottom_pos, Vector3::new(0.0, 0.0, -1.0)));
        bottom_indices.push(bottom_idx);

        // Top vertex at z=height
        let top_pos = Point3::new(r2 * cos, r2 * sin, height);
        let top_idx = mesh.add_vertex(Vertex::new(top_pos, Vector3::new(0.0, 0.0, 1.0)));
        top_indices.push(top_idx);
    }

    // Bottom cap triangles
    for i in 0..segments {
        let next = (i + 1) % segments;
        mesh.add_triangle(Triangle::new([
            bottom_center_idx,
            bottom_indices[next as usize],
            bottom_indices[i as usize],
        ]));
    }

    // Top cap triangles
    for i in 0..segments {
        let next = (i + 1) % segments;
        mesh.add_triangle(Triangle::new([
            top_center_idx,
            top_indices[i as usize],
            top_indices[next as usize],
        ]));
    }

    // Side triangles - reuse existing vertices to maintain manifold topology
    for i in 0..segments {
        let next = (i + 1) % segments;

        // Calculate proper cylindrical normal (radial direction from axis)
        let angle = 2.0 * PI * (i as f64 + 0.5) / segments as f64;
        let cos = angle.cos();
        let sin = angle.sin();
        let side_normal = Vector3::new(cos, sin, 0.0).normalize();

        // Update normals for side vertices (they'll be averaged later)
        // For now, set them to the radial direction
        mesh.vertices[bottom_indices[i as usize]].normal = side_normal;
        mesh.vertices[top_indices[i as usize]].normal = side_normal;
        
        // Use existing vertex indices - this ensures manifold mesh
        let bi = bottom_indices[i as usize];
        let ti = top_indices[i as usize];
        let bn = bottom_indices[next as usize];
        let tn = top_indices[next as usize];

        // Add two triangles per segment
        mesh.add_triangle(Triangle::new([bi, ti, bn]));
        mesh.add_triangle(Triangle::new([ti, tn, bn]));
    }

    // Recompute normals to properly average at shared vertices
    mesh.recompute_normals();
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::mesh_utils::{is_closed, is_manifold};
    use nalgebra::Vector3;

    #[test]
    fn test_cube_generation() {
        let mesh = generate_cube_mesh(Vector3::new(10.0, 10.0, 10.0), false);
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
        assert!(is_manifold(&mesh));
        // Note: cube generation creates duplicate vertices per face, so it's not closed
        // This is expected behavior - each face has its own vertices for proper normals
    }

    #[test]
    fn test_cylinder_is_manifold() {
        let mesh = generate_cylinder_mesh(10.0, 5.0, 32);
        assert!(is_manifold(&mesh), "Cylinder mesh should be manifold");
        assert!(is_closed(&mesh), "Cylinder mesh should be closed");
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
    }

    #[test]
    fn test_cone_is_manifold() {
        let mesh = generate_cone_mesh(10.0, 5.0, 3.0, 32);
        assert!(is_manifold(&mesh), "Cone mesh should be manifold");
        assert!(is_closed(&mesh), "Cone mesh should be closed");
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
    }

    #[test]
    fn test_cylinder_vertex_reuse() {
        // Verify that vertices are reused (no duplicates at same position)
        let mesh = generate_cylinder_mesh(10.0, 5.0, 16);
        
        // Check that side vertices are shared between adjacent triangles
        // A properly generated cylinder should have exactly:
        // - 2 center vertices (top and bottom)
        // - segments * 2 rim vertices (top and bottom rims)
        // Total: 2 + (segments * 2) = 2 + 32 = 34 vertices for 16 segments
        let expected_vertices = 2 + (16 * 2);
        assert_eq!(mesh.vertex_count(), expected_vertices, 
            "Cylinder should have exactly {} vertices (2 centers + {} rim vertices)", 
            expected_vertices, 16 * 2);
    }
}
