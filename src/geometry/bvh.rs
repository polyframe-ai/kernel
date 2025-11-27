// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Bounding Volume Hierarchy (BVH) for spatial acceleration
//! Used to accelerate triangle-triangle intersection tests

use super::BoundingBox;
use nalgebra::Point3;

/// BVH node
#[derive(Debug, Clone)]
pub struct BVHNode {
    /// Bounding box of this node
    pub bbox: BoundingBox,
    /// Left child (None for leaf)
    pub left: Option<Box<BVHNode>>,
    /// Right child (None for leaf)
    pub right: Option<Box<BVHNode>>,
    /// Triangle indices (only for leaf nodes)
    pub triangle_indices: Vec<usize>,
}

impl BVHNode {
    /// Create a leaf node
    fn leaf(bbox: BoundingBox, triangle_indices: Vec<usize>) -> Self {
        Self {
            bbox,
            left: None,
            right: None,
            triangle_indices,
        }
    }

    /// Create an internal node
    fn internal(bbox: BoundingBox, left: Box<BVHNode>, right: Box<BVHNode>) -> Self {
        Self {
            bbox,
            left: Some(left),
            right: Some(right),
            triangle_indices: Vec::new(),
        }
    }

    /// Check if this is a leaf node
    pub fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }
}

/// Bounding Volume Hierarchy for triangle meshes
pub struct BVH {
    root: BVHNode,
}

impl BVH {
    /// Build BVH from triangles
    /// triangles: Vec of (triangle_index, bbox) pairs
    pub fn build(triangles: Vec<(usize, BoundingBox)>) -> Self {
        if triangles.is_empty() {
            // Return empty BVH
            let empty_bbox = BoundingBox::empty();
            return Self {
                root: BVHNode::leaf(empty_bbox, Vec::new()),
            };
        }

        let root = Self::build_recursive(triangles, 0);
        Self { root }
    }

    /// Recursively build BVH node
    fn build_recursive(mut triangles: Vec<(usize, BoundingBox)>, depth: usize) -> BVHNode {
        const MAX_DEPTH: usize = 32;
        const MIN_TRIANGLES: usize = 4;

        // Base case: create leaf if few triangles or max depth reached
        if triangles.len() <= MIN_TRIANGLES || depth >= MAX_DEPTH {
            let bbox = Self::compute_union_bbox(&triangles);
            let indices: Vec<usize> = triangles.iter().map(|(idx, _)| *idx).collect();
            return BVHNode::leaf(bbox, indices);
        }

        // Find best split axis
        let split_axis = Self::find_best_split_axis(&triangles);
        
        // Sort triangles along split axis
        triangles.sort_by(|(_, bbox_a), (_, bbox_b)| {
            let center_a = bbox_a.center();
            let center_b = bbox_b.center();
            match split_axis {
                0 => center_a.x.partial_cmp(&center_b.x).unwrap(),
                1 => center_a.y.partial_cmp(&center_b.y).unwrap(),
                2 => center_a.z.partial_cmp(&center_b.z).unwrap(),
                _ => unreachable!(),
            }
        });

        // Split at median
        let mid = triangles.len() / 2;
        let left_triangles = triangles[..mid].to_vec();
        let right_triangles = triangles[mid..].to_vec();

        // Recursively build children
        let left = Box::new(Self::build_recursive(left_triangles, depth + 1));
        let right = Box::new(Self::build_recursive(right_triangles, depth + 1));

        // Compute union bbox
        let bbox = Self::union_bbox(&left.bbox, &right.bbox);

        BVHNode::internal(bbox, left, right)
    }

    /// Find best split axis (longest axis)
    fn find_best_split_axis(triangles: &[(usize, BoundingBox)]) -> usize {
        let bbox = Self::compute_union_bbox(triangles);
        let size = bbox.size();
        
        if size.x >= size.y && size.x >= size.z {
            0 // X axis
        } else if size.y >= size.z {
            1 // Y axis
        } else {
            2 // Z axis
        }
    }

    /// Compute union bounding box of triangles
    fn compute_union_bbox(triangles: &[(usize, BoundingBox)]) -> BoundingBox {
        if triangles.is_empty() {
            return BoundingBox::empty();
        }

        let mut bbox = triangles[0].1;
        for (_, tri_bbox) in triangles.iter().skip(1) {
            bbox = Self::union_bbox(&bbox, tri_bbox);
        }
        bbox
    }

    /// Compute union of two bounding boxes
    fn union_bbox(a: &BoundingBox, b: &BoundingBox) -> BoundingBox {
        let min = Point3::new(
            a.min.x.min(b.min.x),
            a.min.y.min(b.min.y),
            a.min.z.min(b.min.z),
        );
        let max = Point3::new(
            a.max.x.max(b.max.x),
            a.max.y.max(b.max.y),
            a.max.z.max(b.max.z),
        );
        BoundingBox::new(min, max)
    }

    /// Query triangles that intersect with given bounding box
    pub fn query_triangles(&self, bbox: &BoundingBox) -> Vec<usize> {
        let mut result = Vec::new();
        Self::query_recursive(&self.root, bbox, &mut result);
        result
    }

    /// Recursively query BVH
    fn query_recursive(node: &BVHNode, bbox: &BoundingBox, result: &mut Vec<usize>) {
        // Check if node bbox intersects query bbox
        if !Self::bboxes_intersect(&node.bbox, bbox) {
            return;
        }

        if node.is_leaf() {
            // Add all triangle indices from leaf
            result.extend_from_slice(&node.triangle_indices);
        } else {
            // Recurse into children
            if let Some(ref left) = node.left {
                Self::query_recursive(left, bbox, result);
            }
            if let Some(ref right) = node.right {
                Self::query_recursive(right, bbox, result);
            }
        }
    }

    /// Check if two bounding boxes intersect
    fn bboxes_intersect(a: &BoundingBox, b: &BoundingBox) -> bool {
        a.min.x <= b.max.x
            && a.max.x >= b.min.x
            && a.min.y <= b.max.y
            && a.max.y >= b.min.y
            && a.min.z <= b.max.z
            && a.max.z >= b.min.z
    }

    /// Get root node (for testing)
    #[cfg(test)]
    pub fn root(&self) -> &BVHNode {
        &self.root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{Mesh, Primitive};

    #[test]
    fn test_bvh_build() {
        let mesh = Primitive::cube(nalgebra::Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
        
        // Build triangles with bboxes
        let triangles: Vec<(usize, BoundingBox)> = mesh
            .triangles
            .iter()
            .enumerate()
            .map(|(idx, tri)| {
                let v0 = &mesh.vertices[tri.indices[0]];
                let v1 = &mesh.vertices[tri.indices[1]];
                let v2 = &mesh.vertices[tri.indices[2]];
                
                let mut bbox = BoundingBox::empty();
                bbox.expand_to_include(&v0.position);
                bbox.expand_to_include(&v1.position);
                bbox.expand_to_include(&v2.position);
                
                (idx, bbox)
            })
            .collect();
        
        let bvh = BVH::build(triangles);
        assert!(!bvh.root().is_leaf() || bvh.root().triangle_indices.len() > 0);
    }

    #[test]
    fn test_bvh_query() {
        let mesh = Primitive::cube(nalgebra::Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
        
        let triangles: Vec<(usize, BoundingBox)> = mesh
            .triangles
            .iter()
            .enumerate()
            .map(|(idx, tri)| {
                let v0 = &mesh.vertices[tri.indices[0]];
                let v1 = &mesh.vertices[tri.indices[1]];
                let v2 = &mesh.vertices[tri.indices[2]];
                
                let mut bbox = BoundingBox::empty();
                bbox.expand_to_include(&v0.position);
                bbox.expand_to_include(&v1.position);
                bbox.expand_to_include(&v2.position);
                
                (idx, bbox)
            })
            .collect();
        
        let bvh = BVH::build(triangles);
        
        // Query with mesh bbox (should return all triangles)
        let mesh_bbox = mesh.bounding_box();
        let results = bvh.query_triangles(&mesh_bbox);
        assert!(results.len() > 0);
    }
}

