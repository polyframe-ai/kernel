// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! AST Evaluator - converts AST to geometry

use super::{Node, NodeKind};
use crate::geometry::{BooleanOp, Mesh, Primitive};
use anyhow::{Context, Result};
use dashmap::DashMap;
use nalgebra::Matrix4;
use std::sync::Arc;

/// AST evaluator with caching support
pub struct Evaluator {
    cache: Arc<DashMap<String, Mesh>>,
}

impl Evaluator {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
        }
    }

    /// Evaluate an AST node and return a mesh
    pub fn evaluate(&self, node: &Node) -> Result<Mesh> {
        // Check cache if node has an ID
        if let Some(id) = &node.id {
            if let Some(mesh) = self.cache.get(id) {
                return Ok(mesh.clone());
            }
        }

        let mesh = self.evaluate_node(&node.kind, &Matrix4::identity())?;

        // Store in cache if node has an ID
        if let Some(id) = &node.id {
            self.cache.insert(id.clone(), mesh.clone());
        }

        Ok(mesh)
    }

    fn evaluate_node(&self, kind: &NodeKind, transform: &Matrix4<f64>) -> Result<Mesh> {
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
                self.evaluate_boolean(children, transform, BooleanOp::Union)
            }

            NodeKind::Difference(children) => {
                self.evaluate_boolean(children, transform, BooleanOp::Difference)
            }

            NodeKind::Intersection(children) => {
                self.evaluate_boolean(children, transform, BooleanOp::Intersection)
            }

            NodeKind::Transform { op, children } => {
                let new_transform = transform * op.to_matrix();

                if children.len() == 1 {
                    self.evaluate_node(&children[0].kind, &new_transform)
                } else {
                    self.evaluate_boolean(children, &new_transform, BooleanOp::Union)
                }
            }

            NodeKind::Empty => Ok(Mesh::empty()),
        }
    }

    fn evaluate_boolean(
        &self,
        children: &[Node],
        transform: &Matrix4<f64>,
        op: BooleanOp,
    ) -> Result<Mesh> {
        if children.is_empty() {
            return Ok(Mesh::empty());
        }

        let mut result = self
            .evaluate_node(&children[0].kind, transform)
            .context("Failed to evaluate first child")?;

        for child in &children[1..] {
            let child_mesh = self
                .evaluate_node(&child.kind, transform)
                .context("Failed to evaluate child")?;

            result = result
                .boolean_operation(&child_mesh, op.clone())
                .context("Boolean operation failed")?;
        }

        Ok(result)
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::TransformOp;
    use nalgebra::Vector3;

    #[test]
    fn test_difference_with_transforms() {
        // Test difference operation with transformed children
        // This simulates: difference() { cube(20); translate([10,0,0]) sphere(5); }
        let evaluator = Evaluator::new();

        let cube = Node::new(NodeKind::Cube {
            size: Vector3::new(20.0, 20.0, 20.0),
            center: false,
        });

        let sphere = Node::new(NodeKind::Sphere { r: 5.0, fn_: 16 });
        let translate_op = TransformOp::Translate(Vector3::new(10.0, 0.0, 0.0));
        let transformed_sphere = Node::new(NodeKind::Transform {
            op: translate_op,
            children: vec![sphere],
        });

        let difference = Node::new(NodeKind::Difference(vec![cube, transformed_sphere]));

        let result = evaluator.evaluate(&difference);
        assert!(result.is_ok());
        let mesh = result.unwrap();
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
    }

    #[test]
    fn test_difference_multiple_children_with_transforms() {
        // Test difference with multiple transformed children
        // Simulates mechanical part scenario
        let evaluator = Evaluator::new();

        let base = Node::new(NodeKind::Cube {
            size: Vector3::new(50.0, 30.0, 5.0),
            center: false,
        });

        // First cylinder hole
        let cyl1 = Node::new(NodeKind::Cylinder {
            h: 32.0,
            r: 3.0,
            fn_: 16,
        });
        let trans1 = TransformOp::Translate(Vector3::new(15.0, 15.0, -1.0));
        let transformed_cyl1 = Node::new(NodeKind::Transform {
            op: trans1,
            children: vec![cyl1],
        });

        // Second cylinder hole
        let cyl2 = Node::new(NodeKind::Cylinder {
            h: 32.0,
            r: 3.0,
            fn_: 16,
        });
        let trans2 = TransformOp::Translate(Vector3::new(35.0, 15.0, -1.0));
        let transformed_cyl2 = Node::new(NodeKind::Transform {
            op: trans2,
            children: vec![cyl2],
        });

        let difference = Node::new(NodeKind::Difference(vec![
            base,
            transformed_cyl1,
            transformed_cyl2,
        ]));

        let result = evaluator.evaluate(&difference);
        assert!(result.is_ok());
        let mesh = result.unwrap();
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
    }
}
