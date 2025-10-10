// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Geometric primitives generator

use super::{Mesh, Vertex, Triangle};
use nalgebra::{Point3, Vector3};
use std::f32::consts::PI;

/// Geometric primitives
pub enum Primitive {
    Cube(Vector3<f32>),
    Sphere { r: f32, fn_: u32 },
    Cylinder { h: f32, r: f32, fn_: u32 },
    Cone { h: f32, r1: f32, r2: f32, fn_: u32 },
}

impl Primitive {
    pub fn cube(size: Vector3<f32>) -> Self {
        Self::Cube(size)
    }

    pub fn sphere(r: f32, fn_: u32) -> Self {
        let segments = if fn_ > 0 { fn_ } else { 32 };
        Self::Sphere { r, fn_: segments }
    }

    pub fn cylinder(h: f32, r: f32, fn_: u32) -> Self {
        let segments = if fn_ > 0 { fn_ } else { 32 };
        Self::Cylinder { h, r, fn_: segments }
    }

    pub fn cone(h: f32, r1: f32, r2: f32, fn_: u32) -> Self {
        let segments = if fn_ > 0 { fn_ } else { 32 };
        Self::Cone { h, r1, r2, fn_: segments }
    }

    pub fn to_mesh(&self) -> Mesh {
        match self {
            Self::Cube(size) => generate_cube_mesh(*size),
            Self::Sphere { r, fn_ } => generate_sphere_mesh(*r, *fn_),
            Self::Cylinder { h, r, fn_ } => generate_cylinder_mesh(*h, *r, *fn_),
            Self::Cone { h, r1, r2, fn_ } => generate_cone_mesh(*h, *r1, *r2, *fn_),
        }
    }
}

fn generate_cube_mesh(size: Vector3<f32>) -> Mesh {
    let mut mesh = Mesh::new();
    let hx = size.x / 2.0;
    let hy = size.y / 2.0;
    let hz = size.z / 2.0;

    // 8 vertices of the cube
    let positions = [
        Point3::new(-hx, -hy, -hz), Point3::new(hx, -hy, -hz),
        Point3::new(hx, hy, -hz),   Point3::new(-hx, hy, -hz),
        Point3::new(-hx, -hy, hz),  Point3::new(hx, -hy, hz),
        Point3::new(hx, hy, hz),    Point3::new(-hx, hy, hz),
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

fn generate_sphere_mesh(radius: f32, segments: u32) -> Mesh {
    let mut mesh = Mesh::new();
    let stacks = segments;
    let slices = segments;

    for i in 0..=stacks {
        let phi = PI * i as f32 / stacks as f32;
        let y = radius * phi.cos();
        let r = radius * phi.sin();

        for j in 0..=slices {
            let theta = 2.0 * PI * j as f32 / slices as f32;
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

fn generate_cylinder_mesh(height: f32, radius: f32, segments: u32) -> Mesh {
    generate_cone_mesh(height, radius, radius, segments)
}

fn generate_cone_mesh(height: f32, r1: f32, r2: f32, segments: u32) -> Mesh {
    let mut mesh = Mesh::new();
    let half_h = height / 2.0;

    // Bottom center
    let bottom_center_idx = mesh.add_vertex(Vertex::new(
        Point3::new(0.0, -half_h, 0.0),
        Vector3::new(0.0, -1.0, 0.0),
    ));

    // Top center
    let top_center_idx = mesh.add_vertex(Vertex::new(
        Point3::new(0.0, half_h, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
    ));

    // Generate vertices
    let mut bottom_indices = Vec::new();
    let mut top_indices = Vec::new();

    for i in 0..segments {
        let angle = 2.0 * PI * i as f32 / segments as f32;
        let cos = angle.cos();
        let sin = angle.sin();

        // Bottom vertex
        let bottom_pos = Point3::new(r1 * cos, -half_h, r1 * sin);
        let bottom_idx = mesh.add_vertex(Vertex::new(
            bottom_pos,
            Vector3::new(0.0, -1.0, 0.0),
        ));
        bottom_indices.push(bottom_idx);

        // Top vertex
        let top_pos = Point3::new(r2 * cos, half_h, r2 * sin);
        let top_idx = mesh.add_vertex(Vertex::new(
            top_pos,
            Vector3::new(0.0, 1.0, 0.0),
        ));
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

    // Side triangles
    for i in 0..segments {
        let next = (i + 1) % segments;
        
        // Calculate side normal
        let p1 = mesh.vertices[bottom_indices[i as usize]].position;
        let p2 = mesh.vertices[top_indices[i as usize]].position;
        let p3 = mesh.vertices[bottom_indices[next as usize]].position;
        
        let v1 = p2 - p1;
        let v2 = p3 - p1;
        let normal = v1.cross(&v2).normalize();

        // Add side vertices with proper normals
        let bi = mesh.add_vertex(Vertex::new(p1, normal));
        let ti = mesh.add_vertex(Vertex::new(p2, normal));
        let bn = mesh.add_vertex(Vertex::new(
            mesh.vertices[bottom_indices[next as usize]].position,
            normal,
        ));
        let tn = mesh.add_vertex(Vertex::new(
            mesh.vertices[top_indices[next as usize]].position,
            normal,
        ));

        mesh.add_triangle(Triangle::new([bi, ti, bn]));
        mesh.add_triangle(Triangle::new([ti, tn, bn]));
    }

    mesh
}

