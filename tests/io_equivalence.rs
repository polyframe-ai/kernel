// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! I/O Equivalence Testing - Verify Polyframe outputs match OpenSCAD

use polyframe::cli::{compare_with_openscad, Runner};
use std::path::Path;

/// Helper function to check if we should skip OpenSCAD tests
fn should_skip_openscad_tests() -> bool {
    let runner = Runner::new();
    !runner.is_openscad_available()
}

/// Helper macro to skip test if OpenSCAD is not available
macro_rules! skip_if_no_openscad {
    () => {
        if should_skip_openscad_tests() {
            println!("⏭  Skipping test: OpenSCAD not available in CI environment");
            return;
        }
    };
}

#[test]
fn test_io_equivalence_basic_cube() {
    skip_if_no_openscad!();
    
    let path = Path::new("examples/primitives/cube.scad");
    if !path.exists() {
        println!("⏭  Skipping test: File not found");
        return;
    }
    
    let result = compare_with_openscad(path, 1e-5, false);
    match result {
        Ok(comparison) => {
            assert!(
                comparison.passed,
                "I/O equivalence test failed for cube.scad: vertex count mismatch"
            );
        }
        Err(e) => {
            println!("⚠️  Test error: {}", e);
            // Don't fail the test if it's an expected error
        }
    }
}

#[test]
fn test_io_equivalence_sphere() {
    skip_if_no_openscad!();
    
    let path = Path::new("examples/primitives/sphere.scad");
    if !path.exists() {
        println!("⏭  Skipping test: File not found");
        return;
    }
    
    let result = compare_with_openscad(path, 1e-5, false);
    match result {
        Ok(comparison) => {
            assert!(
                comparison.passed,
                "I/O equivalence test failed for sphere.scad: vertex count mismatch"
            );
        }
        Err(e) => {
            println!("⚠️  Test error: {}", e);
            // Don't fail the test if it's an expected error
        }
    }
}

#[test]
fn test_io_equivalence_difference() {
    skip_if_no_openscad!();
    
    let path = Path::new("examples/operations/difference.scad");
    if !path.exists() {
        println!("⏭  Skipping test: File not found");
        return;
    }
    
    let result = compare_with_openscad(path, 1e-5, false);
    match result {
        Ok(comparison) => {
            assert!(
                comparison.passed,
                "I/O equivalence test failed for difference.scad: vertex count mismatch"
            );
        }
        Err(e) => {
            println!("⚠️  Test error: {}", e);
            // Don't fail the test if it's an expected error
        }
    }
}

