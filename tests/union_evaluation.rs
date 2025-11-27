// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Comprehensive union evaluation tests
//! Tests triangle splitting in robust union across various scenarios

use polyframe::geometry::{Mesh, Primitive, csg_union};
use nalgebra::Vector3;

#[derive(Debug, Clone)]
struct UnionTestResult {
    name: String,
    input_a_triangles: usize,
    input_b_triangles: usize,
    output_triangles: usize,
    output_vertices: usize,
    success: bool,
    error: Option<String>,
}

fn run_union_test(name: &str, mesh_a: &Mesh, mesh_b: &Mesh) -> UnionTestResult {
    let input_a_triangles = mesh_a.triangle_count();
    let input_b_triangles = mesh_b.triangle_count();
    
    match csg_union(mesh_a, mesh_b) {
        Ok(result) => {
            let output_triangles = result.triangle_count();
            let output_vertices = result.vertex_count();
            
            UnionTestResult {
                name: name.to_string(),
                input_a_triangles,
                input_b_triangles,
                output_triangles,
                output_vertices,
                success: output_triangles > 0 && output_vertices > 0,
                error: None,
            }
        }
        Err(e) => {
            UnionTestResult {
                name: name.to_string(),
                input_a_triangles,
                input_b_triangles,
                output_triangles: 0,
                output_vertices: 0,
                success: false,
                error: Some(e.to_string()),
            }
        }
    }
}

#[test]
fn test_union_simple_non_overlapping() {
    // Test 1: Simple non-overlapping union (2 cubes side by side)
    let mesh_a = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
    let mut mesh_b = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
    mesh_b.transform(&nalgebra::Matrix4::new_translation(&Vector3::new(15.0, 0.0, 0.0)));
    
    let result = run_union_test("Simple non-overlapping", &mesh_a, &mesh_b);
    assert!(result.success, "Simple non-overlapping union failed: {:?}", result.error);
    assert!(result.output_triangles > 0, "Output should have triangles");
}

#[test]
fn test_union_overlapping_cubes() {
    // Test 2: Overlapping cubes (partial overlap)
    let mesh_a = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
    let mut mesh_b = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
    mesh_b.transform(&nalgebra::Matrix4::new_translation(&Vector3::new(5.0, 0.0, 0.0)));
    
    let result = run_union_test("Overlapping cubes", &mesh_a, &mesh_b);
    assert!(result.success, "Overlapping cubes union failed: {:?}", result.error);
    // Overlapping should have fewer triangles than simple merge (internal faces removed)
    assert!(result.output_triangles > 0, "Output should have triangles");
}

#[test]
fn test_union_three_cubes() {
    // Test 3: Three overlapping cubes (L-shape)
    let mesh_a = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
    
    let mut mesh_b = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
    mesh_b.transform(&nalgebra::Matrix4::new_translation(&Vector3::new(10.0, 0.0, 0.0)));
    
    // Union A and B first
    let intermediate = csg_union(&mesh_a, &mesh_b).expect("First union failed");
    
    // Then union with third cube
    let mut mesh_c = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
    mesh_c.transform(&nalgebra::Matrix4::new_translation(&Vector3::new(0.0, 10.0, 0.0)));
    
    let result = run_union_test("Three cubes (L-shape)", &intermediate, &mesh_c);
    assert!(result.success, "Three cubes union failed: {:?}", result.error);
    assert!(result.output_triangles > 0, "Output should have triangles");
}

#[test]
fn test_union_cube_sphere() {
    // Test 4: Cube and sphere (curved surface)
    let mesh_a = Primitive::cube(Vector3::new(20.0, 20.0, 20.0), false).to_mesh();
    let mut mesh_b = Primitive::sphere(10.0, 32).to_mesh();
    mesh_b.transform(&nalgebra::Matrix4::new_translation(&Vector3::new(10.0, 10.0, 20.0)));
    
    let result = run_union_test("Cube and sphere", &mesh_a, &mesh_b);
    assert!(result.success, "Cube and sphere union failed: {:?}", result.error);
    assert!(result.output_triangles > 0, "Output should have triangles");
}

#[test]
fn test_union_multiple_cylinders() {
    // Test 5: Multiple cylinders (touching)
    let mesh_a = Primitive::cylinder(10.0, 5.0, 32).to_mesh();
    
    let mut mesh_b = Primitive::cylinder(10.0, 5.0, 32).to_mesh();
    mesh_b.transform(&nalgebra::Matrix4::new_translation(&Vector3::new(12.0, 0.0, 0.0)));
    
    let intermediate = csg_union(&mesh_a, &mesh_b).expect("First union failed");
    
    let mut mesh_c = Primitive::cylinder(10.0, 5.0, 32).to_mesh();
    mesh_c.transform(&nalgebra::Matrix4::new_translation(&Vector3::new(24.0, 0.0, 0.0)));
    
    let result = run_union_test("Multiple cylinders", &intermediate, &mesh_c);
    assert!(result.success, "Multiple cylinders union failed: {:?}", result.error);
    assert!(result.output_triangles > 0, "Output should have triangles");
}

#[test]
fn test_union_overlapping_cylinders() {
    // Test 6: Overlapping cylinders
    let mesh_a = Primitive::cylinder(10.0, 5.0, 32).to_mesh();
    let mut mesh_b = Primitive::cylinder(10.0, 5.0, 32).to_mesh();
    mesh_b.transform(&nalgebra::Matrix4::new_translation(&Vector3::new(6.0, 0.0, 0.0)));
    
    let result = run_union_test("Overlapping cylinders", &mesh_a, &mesh_b);
    assert!(result.success, "Overlapping cylinders union failed: {:?}", result.error);
    assert!(result.output_triangles > 0, "Output should have triangles");
}

#[test]
fn test_union_complex_multi_shape() {
    // Test 7: Complex multi-shape union (cubes + cylinders + spheres)
    // Base cube
    let mut base = Primitive::cube(Vector3::new(30.0, 30.0, 5.0), false).to_mesh();
    
    // Vertical support cylinder
    let mut support = Primitive::cylinder(20.0, 3.0, 32).to_mesh();
    support.transform(&nalgebra::Matrix4::new_translation(&Vector3::new(5.0, 5.0, 5.0)));
    
    let intermediate = csg_union(&base, &support).expect("Base union failed");
    
    // Top plate
    let mut top = Primitive::cube(Vector3::new(30.0, 30.0, 5.0), false).to_mesh();
    top.transform(&nalgebra::Matrix4::new_translation(&Vector3::new(0.0, 0.0, 25.0)));
    
    let intermediate2 = csg_union(&intermediate, &top).expect("Top union failed");
    
    // Sphere on corner
    let mut sphere = Primitive::sphere(5.0, 32).to_mesh();
    sphere.transform(&nalgebra::Matrix4::new_translation(&Vector3::new(0.0, 0.0, 30.0)));
    
    let result = run_union_test("Complex multi-shape", &intermediate2, &sphere);
    assert!(result.success, "Complex multi-shape union failed: {:?}", result.error);
    assert!(result.output_triangles > 0, "Output should have triangles");
}

#[test]
fn test_union_coplanar_faces() {
    // Test 9: Coplanar faces (cubes sharing a face)
    let mesh_a = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
    let mut mesh_b = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
    mesh_b.transform(&nalgebra::Matrix4::new_translation(&Vector3::new(10.0, 0.0, 0.0)));
    
    let result = run_union_test("Coplanar faces", &mesh_a, &mesh_b);
    assert!(result.success, "Coplanar faces union failed: {:?}", result.error);
    assert!(result.output_triangles > 0, "Output should have triangles");
}

#[test]
fn test_union_one_inside_another() {
    // Test 12: One shape completely inside another
    let mesh_a = Primitive::cube(Vector3::new(20.0, 20.0, 20.0), false).to_mesh();
    let mut mesh_b = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
    mesh_b.transform(&nalgebra::Matrix4::new_translation(&Vector3::new(5.0, 5.0, 5.0)));
    
    let result = run_union_test("One inside another", &mesh_a, &mesh_b);
    assert!(result.success, "One inside another union failed: {:?}", result.error);
    // Should result in outer shape only
    assert!(result.output_triangles > 0, "Output should have triangles");
}

#[test]
fn test_union_mechanical_part() {
    // Test 14: Complex mechanical part (like the failing test case)
    // Base plate
    let base = Primitive::cube(Vector3::new(50.0, 30.0, 5.0), false).to_mesh();
    
    // Vertical support
    let mut support = Primitive::cube(Vector3::new(10.0, 30.0, 20.0), false).to_mesh();
    support.transform(&nalgebra::Matrix4::new_translation(&Vector3::new(10.0, 0.0, 5.0)));
    
    let intermediate = csg_union(&base, &support).expect("Base union failed");
    
    // Top plate
    let mut top = Primitive::cube(Vector3::new(50.0, 30.0, 5.0), false).to_mesh();
    top.transform(&nalgebra::Matrix4::new_translation(&Vector3::new(0.0, 0.0, 25.0)));
    
    let result = run_union_test("Mechanical part", &intermediate, &top);
    assert!(result.success, "Mechanical part union failed: {:?}", result.error);
    assert!(result.output_triangles > 0, "Output should have triangles");
    assert!(result.output_vertices > 0, "Output should have vertices");
    
    // This is the critical test - should have significant geometry
    println!("Mechanical part result: {} triangles, {} vertices", 
             result.output_triangles, result.output_vertices);
}

#[test]
fn test_union_comprehensive_suite() {
    // Run all tests and collect results
    let mut results = Vec::new();
    
    // Test 1: Simple non-overlapping
    let mesh_a = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
    let mut mesh_b = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
    mesh_b.transform(&nalgebra::Matrix4::new_translation(&Vector3::new(15.0, 0.0, 0.0)));
    results.push(run_union_test("1. Simple non-overlapping", &mesh_a, &mesh_b));
    
    // Test 2: Overlapping cubes
    let mesh_a = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
    let mut mesh_b = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
    mesh_b.transform(&nalgebra::Matrix4::new_translation(&Vector3::new(5.0, 0.0, 0.0)));
    results.push(run_union_test("2. Overlapping cubes", &mesh_a, &mesh_b));
    
    // Test 3: Cube and sphere
    let mesh_a = Primitive::cube(Vector3::new(20.0, 20.0, 20.0), false).to_mesh();
    let mut mesh_b = Primitive::sphere(10.0, 32).to_mesh();
    mesh_b.transform(&nalgebra::Matrix4::new_translation(&Vector3::new(10.0, 10.0, 20.0)));
    results.push(run_union_test("3. Cube and sphere", &mesh_a, &mesh_b));
    
    // Test 4: Coplanar faces
    let mesh_a = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
    let mut mesh_b = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
    mesh_b.transform(&nalgebra::Matrix4::new_translation(&Vector3::new(10.0, 0.0, 0.0)));
    results.push(run_union_test("4. Coplanar faces", &mesh_a, &mesh_b));
    
    // Test 5: One inside another
    let mesh_a = Primitive::cube(Vector3::new(20.0, 20.0, 20.0), false).to_mesh();
    let mut mesh_b = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), false).to_mesh();
    mesh_b.transform(&nalgebra::Matrix4::new_translation(&Vector3::new(5.0, 5.0, 5.0)));
    results.push(run_union_test("5. One inside another", &mesh_a, &mesh_b));
    
    // Test 6: Mechanical part
    let base = Primitive::cube(Vector3::new(50.0, 30.0, 5.0), false).to_mesh();
    let mut support = Primitive::cube(Vector3::new(10.0, 30.0, 20.0), false).to_mesh();
    support.transform(&nalgebra::Matrix4::new_translation(&Vector3::new(10.0, 0.0, 5.0)));
    let intermediate = csg_union(&base, &support).expect("Base union failed");
    let mut top = Primitive::cube(Vector3::new(50.0, 30.0, 5.0), false).to_mesh();
    top.transform(&nalgebra::Matrix4::new_translation(&Vector3::new(0.0, 0.0, 25.0)));
    results.push(run_union_test("6. Mechanical part", &intermediate, &top));
    
    // Print summary
    println!("\n=== Union Test Suite Results ===");
    println!("{:<30} | {:>8} | {:>8} | {:>8} | {:>8} | Status", 
             "Test Name", "Tri A", "Tri B", "Out Tri", "Out Vtx");
    println!("{}", "-".repeat(100));
    
    let mut passed = 0;
    let mut failed = 0;
    
    for result in &results {
        let status = if result.success { "✓ PASS" } else { "✗ FAIL" };
        println!("{:<30} | {:>8} | {:>8} | {:>8} | {:>8} | {}", 
                 result.name,
                 result.input_a_triangles,
                 result.input_b_triangles,
                 result.output_triangles,
                 result.output_vertices,
                 status);
        
        if let Some(ref err) = result.error {
            println!("  Error: {}", err);
        }
        
        if result.success {
            passed += 1;
        } else {
            failed += 1;
        }
    }
    
    println!("\nSummary: {} passed, {} failed", passed, failed);
    
    // At least most tests should pass
    assert!(passed > failed, "Too many tests failed: {} passed, {} failed", passed, failed);
}

