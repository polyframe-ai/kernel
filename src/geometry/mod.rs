// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Geometry module - mesh representation and operations

pub mod analytics;
mod bbox;
mod boolean;
mod mesh;
mod parallel_boolean;
mod primitives;

pub use analytics::{analyze, GeometryStats};
pub use bbox::BoundingBox;
pub use boolean::BooleanOp;
pub use mesh::{Mesh, Triangle, Vertex};
pub use parallel_boolean::{
    batch_process_meshes, ParallelBooleanExecutor, ThreadSafeMesh, ThreadSafeMeshOps,
};
pub use primitives::Primitive;
