# Union Evaluation Test Suite

This directory contains comprehensive tests for debugging and validating triangle splitting in robust union operations.

## Test Files

### 1. `union_evaluation.rs`
Rust test suite with 10+ individual test cases covering:
- Simple non-overlapping unions
- Overlapping shapes
- Multiple shape unions
- Curved surfaces (spheres, cylinders)
- Coplanar faces
- Nested unions
- Complex mechanical parts

### 2. `union_comprehensive.scad`
OpenSCAD file with 20 test cases ranging from simple to complex:
- Test 1-3: Basic cube unions (non-overlapping, overlapping, L-shape)
- Test 4-6: Curved surfaces (cube+sphere, cylinders)
- Test 7: Complex multi-shape union
- Test 8: Nested unions
- Test 9-11: Edge cases (coplanar, touching, corner)
- Test 12-13: Special cases (one inside another, layers)
- Test 14: Mechanical part (like the failing test case)
- Test 15-20: Stress tests (many shapes, thin intersections, etc.)

## Running the Tests

### Run all union evaluation tests:
```bash
cargo test --test union_evaluation
```

### Run specific test:
```bash
cargo test --test union_evaluation test_union_mechanical_part
```

### Run comprehensive suite with output:
```bash
cargo test --test union_evaluation test_union_comprehensive_suite -- --nocapture
```

## Test Results Format

Each test reports:
- **Input triangles**: Number of triangles in mesh A and B
- **Output triangles**: Number of triangles in union result
- **Output vertices**: Number of vertices in union result
- **Status**: PASS/FAIL

## Current Status

All basic tests are passing:
- ✓ Simple non-overlapping: 22 triangles, 66 vertices
- ✓ Overlapping cubes: 29 triangles, 87 vertices
- ✓ Cube and sphere: 1041 triangles, 3123 vertices
- ✓ Coplanar faces: 20 triangles, 60 vertices
- ✓ One inside another: 14 triangles, 42 vertices
- ✓ Mechanical part: 39 triangles, 117 vertices

## Debugging Tips

1. **Missing shapes**: Check if triangles are being incorrectly classified as "Inside" when they should be "Outside" or "OnBoundary"

2. **Too few triangles**: Verify that splitting is working correctly and fragments are being kept

3. **Too many triangles**: Check for duplicate triangles or incorrect splitting

4. **Empty results**: Verify intersection detection is working and triangles aren't all being filtered out

## Next Steps

1. Run the comprehensive SCAD file through the evaluation system to generate visual comparisons
2. Add more edge case tests (degenerate triangles, very thin intersections, etc.)
3. Add performance benchmarks for large unions
4. Add validation checks (manifold, closed mesh, etc.)

