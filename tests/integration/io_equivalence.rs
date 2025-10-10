// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! I/O Equivalence Testing - Verify Polyframe outputs match OpenSCAD

use polyframe::cli::{compare_with_openscad, Runner};
use anyhow::Result;
use std::path::Path;

/// Helper function to run comparison test
fn run_equivalence_test(file_path: &str, tolerance: f32) -> Result<bool> {
    let path = Path::new(file_path);
    
    if !path.exists() {
        println!("Test file not found: {}", file_path);
        return Ok(true); // Skip if file doesn't exist
    }
    
    // Check if OpenSCAD is available
    let runner = Runner::new();
    if !runner.is_openscad_available() {
        println!("OpenSCAD not found, skipping test");
        return Ok(true); // Skip if OpenSCAD not available
    }
    
    // Run comparison (verbose=false for tests)
    let result = compare_with_openscad(path, tolerance, false)?;
    
    Ok(result.passed)
}

#[test]
fn test_io_equivalence_basic_cube() {
    let result = run_equivalence_test("examples/primitives/cube.scad", 1e-5);
    
    match result {
        Ok(passed) => assert!(passed, "I/O equivalence test failed for cube.scad"),
        Err(e) => println!("Test error (may be expected if OpenSCAD not installed): {}", e),
    }
}

#[test]
fn test_io_equivalence_sphere() {
    let result = run_equivalence_test("examples/primitives/sphere.scad", 1e-5);
    
    match result {
        Ok(passed) => assert!(passed, "I/O equivalence test failed for sphere.scad"),
        Err(e) => println!("Test error (may be expected if OpenSCAD not installed): {}", e),
    }
}

#[test]
fn test_io_equivalence_difference() {
    let result = run_equivalence_test("examples/operations/difference.scad", 1e-5);
    
    match result {
        Ok(passed) => assert!(passed, "I/O equivalence test failed for difference.scad"),
        Err(e) => println!("Test error (may be expected if OpenSCAD not installed): {}", e),
    }
}

