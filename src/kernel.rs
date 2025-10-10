// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Kernel API for incremental rendering

use crate::ast::{CacheStats, IncrementalEvaluator, Node, NodeId};
use crate::geometry::Mesh;
use anyhow::Result;

/// Main kernel for incremental rendering
pub struct Kernel {
    evaluator: IncrementalEvaluator,
    root: Option<Node>,
}

impl Kernel {
    /// Create a new kernel
    pub fn new() -> Self {
        Self {
            evaluator: IncrementalEvaluator::new(),
            root: None,
        }
    }

    /// Initialize kernel with AST
    pub fn with_ast(ast: Node) -> Self {
        let evaluator = IncrementalEvaluator::from_ast(&ast);
        Self {
            evaluator,
            root: Some(ast),
        }
    }

    /// Full render of the current AST
    pub fn render(&self) -> Result<Mesh> {
        if let Some(ref root) = self.root {
            self.evaluator.evaluate(root)
        } else {
            Ok(Mesh::empty())
        }
    }

    /// Update a specific subtree and trigger incremental re-evaluation
    pub fn update_subtree(&mut self, node_id: &NodeId, updated_node: Node) -> Result<Mesh> {
        // Update the evaluator's cache and dependency graph
        self.evaluator.update_subtree(node_id, &updated_node)?;

        // Update the root AST if necessary
        if let Some(root) = &mut self.root {
            Self::update_node_in_ast_static(root, node_id, &updated_node);
        } else {
            // If no root exists, set the updated node as root
            self.root = Some(updated_node.clone());
        }

        // Re-render the entire tree (only affected nodes will be recalculated)
        self.render()
    }

    /// Recursively update a node in the AST (static method)
    fn update_node_in_ast_static(node: &mut Node, target_id: &NodeId, updated_node: &Node) -> bool {
        if let Some(ref id) = node.id {
            if id == target_id {
                *node = updated_node.clone();
                return true;
            }
        }

        // Search in children
        match &mut node.kind {
            crate::ast::NodeKind::Union(children)
            | crate::ast::NodeKind::Difference(children)
            | crate::ast::NodeKind::Intersection(children) => {
                for child in children.iter_mut() {
                    if Self::update_node_in_ast_static(child, target_id, updated_node) {
                        return true;
                    }
                }
            }
            crate::ast::NodeKind::Transform { children, .. } => {
                for child in children.iter_mut() {
                    if Self::update_node_in_ast_static(child, target_id, updated_node) {
                        return true;
                    }
                }
            }
            _ => {}
        }

        false
    }

    /// Invalidate cache for a specific node
    pub fn invalidate(&mut self, node_id: &NodeId) {
        self.evaluator.invalidate(node_id);
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        self.evaluator.cache_stats()
    }

    /// Set the root AST
    pub fn set_ast(&mut self, ast: Node) {
        self.evaluator = IncrementalEvaluator::from_ast(&ast);
        self.root = Some(ast);
    }

    /// Get a reference to the root AST
    pub fn get_ast(&self) -> Option<&Node> {
        self.root.as_ref()
    }
}

impl Default for Kernel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{NodeKind, Vec3};

    #[test]
    fn test_kernel_basic_render() {
        let ast = Node::with_id(NodeKind::Cube(Vec3::new(10.0, 10.0, 10.0)), "cube1".into());

        let kernel = Kernel::with_ast(ast);
        let mesh = kernel.render().unwrap();
        assert!(mesh.vertex_count() > 0);
    }

    #[test]
    fn test_kernel_update_subtree() {
        let child = Node::with_id(NodeKind::Cube(Vec3::new(10.0, 10.0, 10.0)), "child1".into());
        let root = Node::with_id(NodeKind::Union(vec![child]), "root".into());

        let mut kernel = Kernel::with_ast(root);

        // First render
        let mesh1 = kernel.render().unwrap();

        // Update subtree
        let updated_child = Node::with_id(NodeKind::Sphere { r: 15.0, fn_: 32 }, "child1".into());

        let mesh2 = kernel
            .update_subtree(&"child1".to_string(), updated_child)
            .unwrap();

        // Meshes should be different
        assert_ne!(mesh1.vertex_count(), mesh2.vertex_count());
    }
}
