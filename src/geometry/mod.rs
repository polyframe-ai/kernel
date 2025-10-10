// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Geometry module - mesh representation and operations

mod primitives;
mod mesh;
mod boolean;
mod bbox;
mod parallel_boolean;

pub use primitives::Primitive;
pub use mesh::{Mesh, Vertex, Triangle};
pub use boolean::BooleanOp;
pub use bbox::BoundingBox;
pub use parallel_boolean::{
    ParallelBooleanExecutor, 
    ThreadSafeMesh, 
    ThreadSafeMeshOps,
    batch_process_meshes,
};

