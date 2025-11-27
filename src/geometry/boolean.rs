// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Boolean operations using parry3d

use super::{Mesh, Triangle, Vertex};
use anyhow::Result;
use nalgebra::Point3;
use parry3d::shape::TriMesh;

#[derive(Debug, Clone)]
pub enum BooleanOp {
    Union,
    Difference,
    Intersection,
}

/// Quality level for boolean operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanQuality {
    /// Fast implementation (uses simple point-in-mesh tests)
    Fast,
    /// Robust implementation (uses intersection splitting and robust predicates)
    Robust,
}

impl Default for BooleanQuality {
    fn default() -> Self {
        BooleanQuality::Fast
    }
}

/// Perform boolean operation between two meshes
/// Defaults to Robust quality for better results
pub fn perform_boolean_operation(mesh_a: &Mesh, mesh_b: &Mesh, op: BooleanOp) -> Result<Mesh> {
    perform_boolean_operation_with_quality(mesh_a, mesh_b, op, BooleanQuality::Robust)
}

/// Perform boolean operation with specified quality
pub fn perform_boolean_operation_with_quality(
    mesh_a: &Mesh,
    mesh_b: &Mesh,
    op: BooleanOp,
    quality: BooleanQuality,
) -> Result<Mesh> {
    use super::csg;

    match op {
        BooleanOp::Union => csg::csg_union_with_quality(mesh_a, mesh_b, quality),
        BooleanOp::Difference => csg::csg_difference(mesh_a, mesh_b),
        BooleanOp::Intersection => csg::csg_intersection(mesh_a, mesh_b),
    }
}

/// Convert Mesh to parry3d TriMesh
#[allow(dead_code)]
fn mesh_to_trimesh(mesh: &Mesh) -> TriMesh {
    let vertices: Vec<parry3d::math::Point<f32>> = mesh.vertices.iter().map(|v| {
        parry3d::math::Point::new(
            v.position.x as f32,
            v.position.y as f32,
            v.position.z as f32,
        )
    }).collect();

    let indices: Vec<[u32; 3]> = mesh
        .triangles
        .iter()
        .map(|t| {
            [
                t.indices[0] as u32,
                t.indices[1] as u32,
                t.indices[2] as u32,
            ]
        })
        .collect();

    TriMesh::new(vertices, indices)
}

/// Convert parry3d TriMesh to Mesh
#[allow(dead_code)]
fn trimesh_to_mesh(trimesh: &TriMesh) -> Mesh {
    let mut mesh = Mesh::new();

    for vertex in trimesh.vertices() {
        // Calculate normal (placeholder, should be computed properly)
        let normal = nalgebra::Vector3::new(0.0, 1.0, 0.0);
        let position = Point3::new(vertex.x as f64, vertex.y as f64, vertex.z as f64);
        mesh.add_vertex(Vertex::new(position, normal));
    }

    for triangle in trimesh.indices() {
        mesh.add_triangle(Triangle::new([
            triangle[0] as usize,
            triangle[1] as usize,
            triangle[2] as usize,
        ]));
    }

    mesh
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Primitive;

    #[test]
    fn test_union() {
        let mesh_a = Primitive::cube(nalgebra::Vector3::new(10.0, 10.0, 10.0), true).to_mesh();
        let mesh_b = Primitive::sphere(5.0, 16).to_mesh();

        let result = perform_boolean_operation(&mesh_a, &mesh_b, BooleanOp::Union);
        assert!(result.is_ok());
    }
}
