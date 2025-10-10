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

    fn evaluate_node(&self, kind: &NodeKind, transform: &Matrix4<f32>) -> Result<Mesh> {
        match kind {
            NodeKind::Cube(size) => {
                let mut mesh = Primitive::cube(*size).to_mesh();
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
                let new_transform = op.to_matrix() * transform;

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
        transform: &Matrix4<f32>,
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
