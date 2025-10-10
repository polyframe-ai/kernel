// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Dependency graph for incremental evaluation

use super::Node;
use ahash::AHashMap;
use std::collections::HashSet;

/// Unique identifier for AST nodes
pub type NodeId = String;

/// Dependency graph tracking parent-child relationships
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Maps node ID to its direct children
    children: AHashMap<NodeId, Vec<NodeId>>,
    /// Maps node ID to its direct parents
    parents: AHashMap<NodeId, Vec<NodeId>>,
    /// Set of all node IDs in the graph
    nodes: HashSet<NodeId>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            children: AHashMap::new(),
            parents: AHashMap::new(),
            nodes: HashSet::new(),
        }
    }

    /// Build dependency graph from AST
    pub fn from_ast(root: &Node) -> Self {
        let mut graph = Self::new();
        graph.build_from_node(root, None);
        graph
    }

    /// Recursively build graph from node
    fn build_from_node(&mut self, node: &Node, parent_id: Option<&NodeId>) {
        if let Some(node_id) = &node.id {
            self.nodes.insert(node_id.clone());

            // Link to parent if exists
            if let Some(pid) = parent_id {
                self.children
                    .entry(pid.clone())
                    .or_insert_with(Vec::new)
                    .push(node_id.clone());

                self.parents
                    .entry(node_id.clone())
                    .or_insert_with(Vec::new)
                    .push(pid.clone());
            }

            // Process children based on node kind
            let children = node.kind.get_children();
            for child in children {
                self.build_from_node(child, Some(node_id));
            }
        } else {
            // For nodes without ID, still process children
            let children = node.kind.get_children();
            for child in children {
                self.build_from_node(child, parent_id);
            }
        }
    }

    /// Get all descendants of a node (depth-first)
    pub fn get_descendants(&self, node_id: &NodeId) -> Vec<NodeId> {
        let mut descendants = Vec::new();
        let mut visited = HashSet::new();
        self.collect_descendants(node_id, &mut descendants, &mut visited);
        descendants
    }

    fn collect_descendants(
        &self,
        node_id: &NodeId,
        descendants: &mut Vec<NodeId>,
        visited: &mut HashSet<NodeId>,
    ) {
        if visited.contains(node_id) {
            return;
        }
        visited.insert(node_id.clone());

        if let Some(children) = self.children.get(node_id) {
            for child_id in children {
                descendants.push(child_id.clone());
                self.collect_descendants(child_id, descendants, visited);
            }
        }
    }

    /// Get all ancestors of a node (nodes that depend on this node)
    pub fn get_ancestors(&self, node_id: &NodeId) -> Vec<NodeId> {
        let mut ancestors = Vec::new();
        let mut visited = HashSet::new();
        self.collect_ancestors(node_id, &mut ancestors, &mut visited);
        ancestors
    }

    fn collect_ancestors(
        &self,
        node_id: &NodeId,
        ancestors: &mut Vec<NodeId>,
        visited: &mut HashSet<NodeId>,
    ) {
        if visited.contains(node_id) {
            return;
        }
        visited.insert(node_id.clone());

        if let Some(parents) = self.parents.get(node_id) {
            for parent_id in parents {
                ancestors.push(parent_id.clone());
                self.collect_ancestors(parent_id, ancestors, visited);
            }
        }
    }

    /// Get affected nodes when a node changes (node + ancestors)
    pub fn get_affected_nodes(&self, node_id: &NodeId) -> Vec<NodeId> {
        let mut affected = vec![node_id.clone()];
        affected.extend(self.get_ancestors(node_id));
        affected
    }

    /// Check if a node exists in the graph
    pub fn contains(&self, node_id: &NodeId) -> bool {
        self.nodes.contains(node_id)
    }

    /// Get all node IDs
    pub fn all_nodes(&self) -> Vec<NodeId> {
        self.nodes.iter().cloned().collect()
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{NodeKind, Vec3};

    #[test]
    fn test_dependency_graph() {
        let child1 = Node::with_id(NodeKind::Cube(Vec3::new(10.0, 10.0, 10.0)), "child1".into());
        let child2 = Node::with_id(NodeKind::Sphere { r: 5.0, fn_: 32 }, "child2".into());
        let root = Node::with_id(NodeKind::Union(vec![child1, child2]), "root".into());

        let graph = DependencyGraph::from_ast(&root);

        assert!(graph.contains(&"root".to_string()));
        assert!(graph.contains(&"child1".to_string()));
        assert!(graph.contains(&"child2".to_string()));

        let descendants = graph.get_descendants(&"root".to_string());
        assert_eq!(descendants.len(), 2);
    }
}
