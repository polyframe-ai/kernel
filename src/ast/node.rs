// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! AST Node definitions

use serde::{Deserialize, Serialize};

/// 3D Vector type alias
pub type Vec3 = nalgebra::Vector3<f32>;

/// AST Node representing a single operation or primitive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub kind: NodeKind,
    pub id: Option<String>,
}

impl Node {
    pub fn new(kind: NodeKind) -> Self {
        Self { kind, id: None }
    }

    pub fn with_id(kind: NodeKind, id: String) -> Self {
        Self { kind, id: Some(id) }
    }
}

/// Types of AST nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeKind {
    // Primitives
    Cube(Vec3),
    Sphere {
        r: f32,
        fn_: u32,
    },
    Cylinder {
        h: f32,
        r: f32,
        fn_: u32,
    },
    Cone {
        h: f32,
        r1: f32,
        r2: f32,
        fn_: u32,
    },

    // Boolean operations
    Union(Vec<Node>),
    Difference(Vec<Node>),
    Intersection(Vec<Node>),

    // Transformations
    Transform {
        op: TransformOp,
        children: Vec<Node>,
    },

    // Empty node
    Empty,
}

impl NodeKind {
    /// Get child nodes for dependency tracking
    pub fn get_children(&self) -> Vec<&Node> {
        match self {
            NodeKind::Union(children) => children.iter().collect(),
            NodeKind::Difference(children) => children.iter().collect(),
            NodeKind::Intersection(children) => children.iter().collect(),
            NodeKind::Transform { children, .. } => children.iter().collect(),
            _ => Vec::new(),
        }
    }
}

/// Transformation operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransformOp {
    Translate(Vec3),
    Rotate(Vec3),
    Scale(Vec3),
    Mirror(Vec3),
    Multmatrix(nalgebra::Matrix4<f32>),
}

impl TransformOp {
    /// Convert transformation to a 4x4 matrix
    pub fn to_matrix(&self) -> nalgebra::Matrix4<f32> {
        use nalgebra::{Matrix4, UnitQuaternion, Vector3};

        match self {
            TransformOp::Translate(v) => Matrix4::new_translation(v),
            TransformOp::Rotate(angles) => {
                let rx = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), angles.x.to_radians());
                let ry = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), angles.y.to_radians());
                let rz = UnitQuaternion::from_axis_angle(&Vector3::z_axis(), angles.z.to_radians());
                (rz * ry * rx).to_homogeneous()
            }
            TransformOp::Scale(s) => Matrix4::new_nonuniform_scaling(s),
            TransformOp::Mirror(axis) => {
                let mut m = Matrix4::identity();
                if axis.x != 0.0 {
                    m[(0, 0)] = -1.0;
                }
                if axis.y != 0.0 {
                    m[(1, 1)] = -1.0;
                }
                if axis.z != 0.0 {
                    m[(2, 2)] = -1.0;
                }
                m
            }
            TransformOp::Multmatrix(m) => *m,
        }
    }
}
