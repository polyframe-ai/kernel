// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Incremental evaluator with cache invalidation

use super::{
    dependency_graph::{DependencyGraph, NodeId},
    Node, NodeKind,
};
use crate::geometry::{BooleanOp, Mesh, Primitive};
use anyhow::{Context, Result};
use dashmap::DashMap;
use nalgebra::Matrix4;
use std::sync::{Arc, RwLock};

/// Thread-safe mesh cache
pub type MeshCache = Arc<DashMap<NodeId, Arc<RwLock<Mesh>>>>;

/// Incremental evaluator with dependency tracking
pub struct IncrementalEvaluator {
    cache: MeshCache,
    dep_graph: DependencyGraph,
}

impl IncrementalEvaluator {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            dep_graph: DependencyGraph::new(),
        }
    }

    /// Initialize with AST and build dependency graph
    pub fn from_ast(root: &Node) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            dep_graph: DependencyGraph::from_ast(root),
        }
    }

    /// Full evaluation of the AST
    pub fn evaluate(&self, node: &Node) -> Result<Mesh> {
        self.evaluate_node(&node.kind, &Matrix4::identity(), &node.id)
    }

    /// Update a specific subtree and re-evaluate affected nodes
    pub fn update_subtree(&mut self, node_id: &NodeId, updated_node: &Node) -> Result<()> {
        // Get all affected nodes (this node + ancestors)
        let affected = self.dep_graph.get_affected_nodes(node_id);

        // Invalidate cache for affected nodes
        for id in &affected {
            self.cache.remove(id);
        }

        // Rebuild dependency graph for the updated subtree
        self.dep_graph = DependencyGraph::from_ast(updated_node);

        Ok(())
    }

    /// Invalidate cache for specific node and its ancestors
    pub fn invalidate(&mut self, node_id: &NodeId) {
        let affected = self.dep_graph.get_affected_nodes(node_id);
        for id in affected {
            self.cache.remove(&id);
        }
    }

    /// Get cached mesh if available
    pub fn get_cached(&self, node_id: &NodeId) -> Option<Mesh> {
        self.cache
            .get(node_id)
            .map(|entry| entry.read().unwrap().clone())
    }

    /// Evaluate node with caching
    fn evaluate_node(
        &self,
        kind: &NodeKind,
        transform: &Matrix4<f64>,
        node_id: &Option<String>,
    ) -> Result<Mesh> {
        // Check cache if node has an ID
        if let Some(id) = node_id {
            if let Some(cached) = self.get_cached(id) {
                return Ok(cached);
            }
        }

        let mesh = self.evaluate_node_uncached(kind, transform)?;

        // Store in cache if node has an ID
        if let Some(id) = node_id {
            self.cache
                .insert(id.clone(), Arc::new(RwLock::new(mesh.clone())));
        }

        Ok(mesh)
    }

    fn evaluate_node_uncached(&self, kind: &NodeKind, transform: &Matrix4<f64>) -> Result<Mesh> {
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
                    self.evaluate_node(&children[0].kind, &new_transform, &children[0].id)
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
            .evaluate_node(&children[0].kind, transform, &children[0].id)
            .context("Failed to evaluate first child")?;

        for child in &children[1..] {
            let child_mesh = self
                .evaluate_node(&child.kind, transform, &child.id)
                .context("Failed to evaluate child")?;

            result = result
                .boolean_operation(&child_mesh, op.clone())
                .context("Boolean operation failed")?;
        }

        Ok(result)
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            cached_nodes: self.cache.len(),
            total_nodes: self.dep_graph.all_nodes().len(),
        }
    }
}

impl Default for IncrementalEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub cached_nodes: usize,
    pub total_nodes: usize,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f32 {
        if self.total_nodes == 0 {
            0.0
        } else {
            (self.cached_nodes as f32 / self.total_nodes as f32) * 100.0
        }
    }
}
