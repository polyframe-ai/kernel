// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Parallel AST evaluator using rayon

use super::{Node, NodeKind};
use crate::geometry::{BooleanOp, Mesh, ParallelBooleanExecutor, Primitive};
use anyhow::Result;
use nalgebra::Matrix4;
use rayon::prelude::*;

/// Parallel evaluator for AST
pub struct ParallelEvaluator;

impl ParallelEvaluator {
    /// Evaluate AST with parallel processing
    pub fn evaluate(node: &Node) -> Result<Mesh> {
        Self::evaluate_node(&node.kind, &Matrix4::identity())
    }

    fn evaluate_node(kind: &NodeKind, transform: &Matrix4<f64>) -> Result<Mesh> {
        match kind {
            NodeKind::Cube { size, center } => {
                let mut mesh = Primitive::cube(*size, *center).to_mesh();
                mesh.transform(transform);
                Ok(mesh)
            }

            NodeKind::Sphere { r, fn_ } => {
                let mut mesh = Primitive::sphere(*r, *fn_).to_mesh();
                mesh.transform(transform);
                Ok(mesh)
            }

            NodeKind::Cylinder { h, r, fn_ } => {
                let mut mesh = Primitive::cylinder(*h, *r, *fn_).to_mesh();
                mesh.transform(transform);
                Ok(mesh)
            }

            NodeKind::Cone { h, r1, r2, fn_ } => {
                let mut mesh = Primitive::cone(*h, *r1, *r2, *fn_).to_mesh();
                mesh.transform(transform);
                Ok(mesh)
            }

            NodeKind::Union(children) => {
                Self::evaluate_parallel_boolean(children, transform, BooleanOp::Union)
            }

            NodeKind::Difference(children) => {
                Self::evaluate_parallel_boolean(children, transform, BooleanOp::Difference)
            }

            NodeKind::Intersection(children) => {
                Self::evaluate_parallel_boolean(children, transform, BooleanOp::Intersection)
            }

            NodeKind::Transform { op, children } => {
                let new_transform = transform * op.to_matrix();

                if children.len() == 1 {
                    Self::evaluate_node(&children[0].kind, &new_transform)
                } else {
                    Self::evaluate_parallel_boolean(children, &new_transform, BooleanOp::Union)
                }
            }

            NodeKind::Empty => Ok(Mesh::empty()),
        }
    }

    fn evaluate_parallel_boolean(
        children: &[Node],
        transform: &Matrix4<f64>,
        op: BooleanOp,
    ) -> Result<Mesh> {
        if children.is_empty() {
            return Ok(Mesh::empty());
        }

        // Evaluate children in parallel
        let meshes: Result<Vec<Mesh>> = children
            .par_iter()
            .map(|child| Self::evaluate_node(&child.kind, transform))
            .collect();

        let meshes = meshes?;

        // Combine results based on operation
        match op {
            BooleanOp::Union => ParallelBooleanExecutor::union_parallel(meshes),
            BooleanOp::Difference => ParallelBooleanExecutor::difference_parallel(meshes),
            BooleanOp::Intersection => ParallelBooleanExecutor::intersection_parallel(meshes),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Vec3;

    #[test]
    fn test_parallel_evaluate_union() {
        let child1 = Node::new(NodeKind::Cube {
            size: Vec3::new(10.0, 10.0, 10.0),
            center: true,
        });
        let child2 = Node::new(NodeKind::Sphere { r: 5.0, fn_: 32 });
        let root = Node::new(NodeKind::Union(vec![child1, child2]));

        let mesh = ParallelEvaluator::evaluate(&root).unwrap();
        assert!(mesh.vertex_count() > 0);
    }

    #[test]
    fn test_parallel_evaluate_transform() {
        let cube = Node::new(NodeKind::Cube {
            size: Vec3::new(10.0, 10.0, 10.0),
            center: true,
        });
        let root = Node::new(NodeKind::Transform {
            op: crate::ast::TransformOp::Translate(Vec3::new(5.0, 0.0, 0.0)),
            children: vec![cube],
        });

        let mesh = ParallelEvaluator::evaluate(&root).unwrap();
        assert!(mesh.vertex_count() > 0);
    }
}
