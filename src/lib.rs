// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Polyframe CAD Kernel
//!
//! A high-performance Rust-based CAD kernel with OpenSCAD compatibility.
//! Provides parametric modeling, boolean operations, and multiple export formats.

pub mod ast;
pub mod benchmark_metrics;
pub mod cli;
pub mod evaluation;
pub mod geometry;
pub mod io;
pub mod kernel;
pub mod utils;

#[cfg(feature = "wasm")]
pub mod ffi;

pub use ast::{
    CacheStats, IncrementalEvaluator, Node, NodeId, NodeKind, ParallelEvaluator, TransformOp,
};
pub use geometry::{analyze, GeometryStats, Mesh, Primitive};
pub use io::{export_3mf, export_gltf, export_step, export_stl, import_scad_file, parse_scad};
pub use kernel::Kernel;

use anyhow::Result;

/// Main entry point for rendering a SCAD script to a mesh
pub fn render(source: &str) -> Result<Mesh> {
    let ast = parse_scad(source)?;
    let evaluator = ast::Evaluator::new();
    evaluator.evaluate(&ast)
}

/// Render a SCAD file to a mesh
pub fn render_file(path: &str) -> Result<Mesh> {
    let ast = import_scad_file(path)?;
    let evaluator = ast::Evaluator::new();
    evaluator.evaluate(&ast)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_cube() {
        let result = render("cube([10, 10, 10]);");
        assert!(result.is_ok());
    }
}
