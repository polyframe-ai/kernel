// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Abstract Syntax Tree module
//! 
//! Defines the AST structure for OpenSCAD-compatible operations

mod node;
mod evaluator;
mod dependency_graph;
mod incremental_evaluator;
mod parallel_evaluator;

pub use node::{Node, NodeKind, TransformOp, Vec3};
pub use evaluator::Evaluator;
pub use dependency_graph::{DependencyGraph, NodeId};
pub use incremental_evaluator::{IncrementalEvaluator, CacheStats, MeshCache};
pub use parallel_evaluator::ParallelEvaluator;

