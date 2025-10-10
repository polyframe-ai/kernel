// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Abstract Syntax Tree module
//!
//! Defines the AST structure for OpenSCAD-compatible operations

mod dependency_graph;
mod evaluator;
mod incremental_evaluator;
mod node;
mod parallel_evaluator;

pub use dependency_graph::{DependencyGraph, NodeId};
pub use evaluator::Evaluator;
pub use incremental_evaluator::{CacheStats, IncrementalEvaluator, MeshCache};
pub use node::{Node, NodeKind, TransformOp, Vec3};
pub use parallel_evaluator::ParallelEvaluator;
