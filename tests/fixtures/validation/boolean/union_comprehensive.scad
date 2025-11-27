// Comprehensive union tests - ranging from simple to complex
// These tests help debug triangle splitting in robust union

// Test 1: Simple non-overlapping union (2 cubes side by side)
// Expected: Both cubes should appear
union() {
    cube([10, 10, 10]);
    translate([15, 0, 0]) cube([10, 10, 10]);
}

// Test 2: Overlapping cubes (partial overlap)
// Expected: Should merge into single shape with internal faces removed
translate([0, 20, 0]) union() {
    cube([10, 10, 10]);
    translate([5, 0, 0]) cube([10, 10, 10]);
}

// Test 3: Three overlapping cubes (L-shape)
// Expected: All three cubes should merge
translate([0, 40, 0]) union() {
    cube([10, 10, 10]);
    translate([10, 0, 0]) cube([10, 10, 10]);
    translate([0, 10, 0]) cube([10, 10, 10]);
}

// Test 4: Cube and sphere (curved surface)
// Expected: Cube with sphere on top
translate([0, 60, 0]) union() {
    cube([20, 20, 20]);
    translate([10, 10, 20]) sphere(r=10);
}

// Test 5: Multiple cylinders (touching)
// Expected: All cylinders should appear
translate([0, 90, 0]) union() {
    cylinder(h=10, r=5);
    translate([12, 0, 0]) cylinder(h=10, r=5);
    translate([24, 0, 0]) cylinder(h=10, r=5);
}

// Test 6: Overlapping cylinders
// Expected: Should merge smoothly
translate([0, 110, 0]) union() {
    cylinder(h=10, r=5);
    translate([6, 0, 0]) cylinder(h=10, r=5);
}

// Test 7: Complex multi-shape union (cubes + cylinders + spheres)
// Expected: All shapes should appear and merge correctly
translate([0, 130, 0]) union() {
    // Base cube
    cube([30, 30, 5]);
    // Vertical supports
    translate([5, 5, 5]) cylinder(h=20, r=3);
    translate([25, 5, 5]) cylinder(h=20, r=3);
    translate([5, 25, 5]) cylinder(h=20, r=3);
    translate([25, 25, 5]) cylinder(h=20, r=3);
    // Top plate
    translate([0, 0, 25]) cube([30, 30, 5]);
    // Spheres on corners
    translate([0, 0, 30]) sphere(r=5);
    translate([30, 0, 30]) sphere(r=5);
    translate([0, 30, 30]) sphere(r=5);
    translate([30, 30, 30]) sphere(r=5);
}

// Test 8: Nested unions (union of unions)
// Expected: Should flatten correctly
translate([50, 0, 0]) union() {
    union() {
        cube([8, 8, 8]);
        translate([6, 0, 0]) cube([8, 8, 8]);
    }
    union() {
        translate([0, 10, 0]) cube([8, 8, 8]);
        translate([6, 10, 0]) cube([8, 8, 8]);
    }
}

// Test 9: Coplanar faces (cubes sharing a face)
// Expected: Should merge without gaps
translate([50, 30, 0]) union() {
    cube([10, 10, 10]);
    translate([10, 0, 0]) cube([10, 10, 10]);
}

// Test 10: Edge-to-edge touching (no overlap)
// Expected: Should connect seamlessly
translate([50, 50, 0]) union() {
    cube([10, 10, 10]);
    translate([10, 0, 0]) cube([10, 10, 10]);
}

// Test 11: Corner touching
// Expected: Should connect at corner
translate([50, 70, 0]) union() {
    cube([10, 10, 10]);
    translate([10, 10, 0]) cube([10, 10, 10]);
}

// Test 12: One shape completely inside another
// Expected: Should result in outer shape only
translate([50, 90, 0]) union() {
    cube([20, 20, 20]);
    translate([5, 5, 5]) cube([10, 10, 10]);
}

// Test 13: Multiple overlapping layers
// Expected: Should stack correctly
translate([50, 120, 0]) union() {
    cube([15, 15, 5]);
    translate([0, 0, 5]) cube([15, 15, 5]);
    translate([0, 0, 10]) cube([15, 15, 5]);
}

// Test 14: Complex mechanical part (like the failing test case)
// Expected: All parts should appear
translate([100, 0, 0]) union() {
    // Base plate
    cube([50, 30, 5]);
    // Vertical support
    translate([10, 0, 5]) cube([10, 30, 20]);
    // Top plate
    translate([0, 0, 25]) cube([50, 30, 5]);
}

// Test 15: Many small overlapping shapes
// Expected: Should merge into complex shape
translate([100, 40, 0]) union() {
    for (i = [0:4]) {
        for (j = [0:4]) {
            translate([i*8, j*8, 0]) cube([10, 10, 10]);
        }
    }
}

// Test 16: Curved surfaces union (spheres)
// Expected: Should merge smoothly
translate([100, 90, 0]) union() {
    sphere(r=8);
    translate([10, 0, 0]) sphere(r=8);
}

// Test 17: Mixed primitives (cube, cylinder, sphere)
// Expected: All should merge
translate([100, 120, 0]) union() {
    cube([15, 15, 15]);
    translate([7.5, 7.5, 15]) cylinder(h=10, r=5);
    translate([7.5, 7.5, 25]) sphere(r=6);
}

// Test 18: Thin overlapping shapes (stress test for splitting)
// Expected: Should handle thin intersections
translate([150, 0, 0]) union() {
    cube([20, 1, 20]);
    translate([0, 0.5, 0]) cube([20, 1, 20]);
}

// Test 19: Large union with many intersections
// Expected: Should handle many triangle splits
translate([150, 30, 0]) union() {
    cube([30, 30, 10]);
    for (i = [0:2]) {
        for (j = [0:2]) {
            translate([i*10+5, j*10+5, 10]) cylinder(h=10, r=4);
        }
    }
}

// Test 20: Extreme overlap (almost identical shapes)
// Expected: Should result in single shape
translate([150, 70, 0]) union() {
    cube([10, 10, 10]);
    translate([0.1, 0.1, 0.1]) cube([10, 10, 10]);
}

