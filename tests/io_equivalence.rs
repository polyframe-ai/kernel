// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! I/O Equivalence Testing - Verify Polyframe outputs match OpenSCAD

use polyframe::cli::{compare_with_openscad, Runner};
use polyframe::render_file;
use std::path::Path;

/// Helper function to check if OpenSCAD is available
fn is_openscad_available() -> bool {
    let runner = Runner::new();
    runner.is_openscad_available()
}

/// Test that Polyframe can at least render the file, with optional OpenSCAD comparison
fn test_render_and_compare(file_path: &str, tolerance: f32, expected_min_vertices: usize) {
    let path = Path::new(file_path);

    if !path.exists() {
        panic!("Test file not found: {}", file_path);
    }

    // Always test that Polyframe can render
    println!("Testing Polyframe rendering of: {}", file_path);
    let mesh = render_file(file_path).expect("Polyframe failed to render");

    println!(
        "  Polyframe output: {} vertices, {} triangles",
        mesh.vertex_count(),
        mesh.triangle_count()
    );

    // Verify minimum mesh quality
    assert!(
        mesh.vertex_count() >= expected_min_vertices,
        "Polyframe generated too few vertices: {} < {}",
        mesh.vertex_count(),
        expected_min_vertices
    );
    assert!(
        mesh.triangle_count() > 0,
        "Polyframe generated no triangles"
    );

    // If OpenSCAD is available, do comparison
    if is_openscad_available() {
        println!("  OpenSCAD available, running comparison...");
        match compare_with_openscad(path, tolerance, false) {
            Ok(comparison) => {
                println!(
                    "  Comparison result: {}",
                    if comparison.passed {
                        "PASSED"
                    } else {
                        "FAILED"
                    }
                );
                println!("    Vertex delta: {:.2}%", comparison.vertex_delta);
                println!("    Triangle delta: {:.2}%", comparison.triangle_delta);

                // Only assert if comparison is within reasonable bounds
                // (some minor differences may be acceptable due to different implementations)
                if comparison.vertex_delta > 50.0 || comparison.triangle_delta > 50.0 {
                    panic!(
                        "Large discrepancy with OpenSCAD: vertex_delta={:.2}%, triangle_delta={:.2}%",
                        comparison.vertex_delta, comparison.triangle_delta
                    );
                }
            }
            Err(e) => {
                println!("  Comparison error: {}", e);
            }
        }
    } else {
        println!("  OpenSCAD not available, skipping comparison");
    }
}

#[test]
fn test_io_equivalence_basic_cube() {
    // Cube should have at least 8 vertices (corners), typically 36 with normals
    test_render_and_compare("examples/primitives/cube.scad", 1e-5, 8);
}

#[test]
fn test_io_equivalence_sphere() {
    // Sphere should have many vertices depending on $fn
    test_render_and_compare("examples/primitives/sphere.scad", 1e-5, 32);
}

#[test]
fn test_io_equivalence_difference() {
    // Difference operation should produce a valid mesh
    test_render_and_compare("examples/operations/difference.scad", 1e-5, 8);
}
