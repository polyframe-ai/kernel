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
    pub position: Point3<f32>,
    pub normal: Vector3<f32>,
}

impl Vertex {
    pub fn new(position: Point3<f32>, normal: Vector3<f32>) -> Self {
        Self { position, normal }
    }

    pub fn transform(&mut self, matrix: &Matrix4<f32>) {
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
    pub fn transform(&mut self, matrix: &Matrix4<f32>) {
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
    pub fn boolean_operation(&self, other: &Mesh, op: BooleanOp) -> Result<Mesh> {
        super::boolean::perform_boolean_operation(self, other, op)
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
}

impl Default for Mesh {
    fn default() -> Self {
        Self::new()
    }
}
