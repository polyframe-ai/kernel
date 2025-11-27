// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Geometry module - mesh representation and operations

pub mod analytics;
mod bbox;
mod boolean;
mod mesh;
mod mesh_utils;
mod parallel_boolean;
mod primitives;
mod csg;
mod robust_csg;
mod halfedge;
mod robust_predicates;
mod bvh;
mod triangle_intersection;
mod triangle_splitting;
mod classification;
mod mesh_reconstruction;

pub use analytics::{analyze, GeometryStats};
pub use bbox::BoundingBox;
pub use boolean::{BooleanOp, BooleanQuality};
pub use csg::{csg_difference, csg_intersection, csg_union};
pub use mesh::{Mesh, Triangle, Vertex};
pub use mesh_utils::{is_closed, is_manifold, validate_mesh, validate_winding_order, MeshValidation};
pub use parallel_boolean::{
    batch_process_meshes, ParallelBooleanExecutor, ThreadSafeMesh, ThreadSafeMeshOps,
};
pub use primitives::Primitive;
pub use robust_csg::{robust_difference, robust_intersection, robust_union};
