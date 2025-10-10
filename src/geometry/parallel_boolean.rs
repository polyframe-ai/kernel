// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Parallel boolean operations using rayon

use super::{BooleanOp, Mesh};
use anyhow::Result;
use rayon::prelude::*;
use std::sync::{Arc, RwLock};

/// Thread-safe mesh wrapper
pub type ThreadSafeMesh = Arc<RwLock<Mesh>>;

/// Parallel boolean operation executor
pub struct ParallelBooleanExecutor;

impl ParallelBooleanExecutor {
    /// Perform parallel boolean operation on multiple meshes
    pub fn execute_parallel(meshes: Vec<Mesh>, op: BooleanOp) -> Result<Mesh> {
        if meshes.is_empty() {
            return Ok(Mesh::empty());
        }

        if meshes.len() == 1 {
            return Ok(meshes.into_iter().next().unwrap());
        }

        // Parallel reduce using rayon
        let result = meshes.into_par_iter().reduce(Mesh::empty, |acc, mesh| {
            if acc.vertex_count() == 0 {
                mesh
            } else {
                acc.boolean_operation(&mesh, op.clone()).unwrap_or(acc)
            }
        });

        Ok(result)
    }

    /// Parallel union of meshes
    pub fn union_parallel(meshes: Vec<Mesh>) -> Result<Mesh> {
        Self::execute_parallel(meshes, BooleanOp::Union)
    }

    /// Parallel difference (first mesh minus all others)
    pub fn difference_parallel(meshes: Vec<Mesh>) -> Result<Mesh> {
        if meshes.is_empty() {
            return Ok(Mesh::empty());
        }

        let mut iter = meshes.into_iter();
        let mut result = iter.next().unwrap();

        // Apply difference sequentially (must be ordered)
        for mesh in iter {
            result = result.boolean_operation(&mesh, BooleanOp::Difference)?;
        }

        Ok(result)
    }

    /// Parallel intersection of meshes
    pub fn intersection_parallel(meshes: Vec<Mesh>) -> Result<Mesh> {
        Self::execute_parallel(meshes, BooleanOp::Intersection)
    }

    /// Transform multiple meshes in parallel
    pub fn transform_parallel(meshes: Vec<Mesh>, transform: nalgebra::Matrix4<f32>) -> Vec<Mesh> {
        meshes
            .into_par_iter()
            .map(|mut mesh| {
                mesh.transform(&transform);
                mesh
            })
            .collect()
    }
}

/// Thread-safe mesh operations
pub trait ThreadSafeMeshOps {
    fn clone_mesh(&self) -> Mesh;
    fn transform_safe(&self, matrix: &nalgebra::Matrix4<f32>);
    fn merge_safe(&self, other: &Mesh);
}

impl ThreadSafeMeshOps for ThreadSafeMesh {
    fn clone_mesh(&self) -> Mesh {
        self.read().unwrap().clone()
    }

    fn transform_safe(&self, matrix: &nalgebra::Matrix4<f32>) {
        self.write().unwrap().transform(matrix);
    }

    fn merge_safe(&self, other: &Mesh) {
        self.write().unwrap().merge(other);
    }
}

/// Batch process meshes in parallel
pub fn batch_process_meshes<F>(meshes: Vec<Mesh>, processor: F) -> Vec<Mesh>
where
    F: Fn(Mesh) -> Mesh + Sync + Send,
{
    meshes.into_par_iter().map(processor).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Primitive;
    use nalgebra::Vector3;

    #[test]
    fn test_parallel_union() {
        let meshes = vec![
            Primitive::cube(Vector3::new(10.0, 10.0, 10.0), true).to_mesh(),
            Primitive::sphere(5.0, 16).to_mesh(),
            Primitive::cylinder(10.0, 3.0, 16).to_mesh(),
        ];

        let result = ParallelBooleanExecutor::union_parallel(meshes);
        assert!(result.is_ok());
        assert!(result.unwrap().vertex_count() > 0);
    }

    #[test]
    fn test_parallel_transform() {
        let meshes = vec![
            Primitive::cube(Vector3::new(10.0, 10.0, 10.0), true).to_mesh(),
            Primitive::sphere(5.0, 16).to_mesh(),
        ];

        let matrix = nalgebra::Matrix4::new_translation(&Vector3::new(5.0, 0.0, 0.0));
        let results = ParallelBooleanExecutor::transform_parallel(meshes, matrix);

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_thread_safe_mesh() {
        let mesh = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), true).to_mesh();
        let safe_mesh: ThreadSafeMesh = Arc::new(RwLock::new(mesh));

        let cloned = safe_mesh.clone_mesh();
        assert_eq!(
            safe_mesh.read().unwrap().vertex_count(),
            cloned.vertex_count()
        );
    }
}
