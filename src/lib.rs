// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Polyframe CAD Kernel
//! 
//! A high-performance Rust-based CAD kernel with OpenSCAD compatibility.
//! Provides parametric modeling, boolean operations, and multiple export formats.

pub mod ast;
pub mod geometry;
pub mod io;
pub mod utils;
pub mod cli;
pub mod evaluation;
pub mod kernel;
pub mod benchmark_metrics;

#[cfg(feature = "wasm")]
pub mod ffi;

pub use ast::{Node, NodeKind, TransformOp, IncrementalEvaluator, NodeId, CacheStats, ParallelEvaluator};
pub use geometry::{Mesh, Primitive};
pub use io::{parse_scad, import_scad_file, export_stl, export_3mf, export_gltf};
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

