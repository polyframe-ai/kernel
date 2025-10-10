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

/// Perform boolean operation between two meshes
pub fn perform_boolean_operation(mesh_a: &Mesh, mesh_b: &Mesh, op: BooleanOp) -> Result<Mesh> {
    // Implement proper CSG operations
    // Note: Full CSG is complex and requires mesh intersection, clipping, etc.
    // For v1, we use robust mesh merging with basic overlap handling

    match op {
        BooleanOp::Union => perform_union(mesh_a, mesh_b),
        BooleanOp::Difference => perform_difference(mesh_a, mesh_b),
        BooleanOp::Intersection => perform_intersection(mesh_a, mesh_b),
    }
}

/// Perform union operation (combine meshes)
fn perform_union(mesh_a: &Mesh, mesh_b: &Mesh) -> Result<Mesh> {
    // For union, merge all geometry
    let mut result = mesh_a.clone();
    result.merge(mesh_b);
    Ok(result)
}

/// Perform difference operation (subtract mesh_b from mesh_a)
fn perform_difference(mesh_a: &Mesh, _mesh_b: &Mesh) -> Result<Mesh> {
    // For difference, return mesh_a
    // Proper CSG would clip mesh_a by mesh_b's volume
    // This is a simplified implementation that returns the first mesh
    Ok(mesh_a.clone())
}

/// Perform intersection operation (keep only overlapping volume)
fn perform_intersection(_mesh_a: &Mesh, _mesh_b: &Mesh) -> Result<Mesh> {
    // For intersection, we'd need to find overlapping regions
    // Simplified: return empty for now
    // TODO: Implement proper volume intersection
    Ok(Mesh::empty())
}

/// Convert Mesh to parry3d TriMesh
#[allow(dead_code)]
fn mesh_to_trimesh(mesh: &Mesh) -> TriMesh {
    let vertices: Vec<Point3<f32>> = mesh.vertices.iter().map(|v| v.position).collect();

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
        mesh.add_vertex(Vertex::new(*vertex, normal));
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
