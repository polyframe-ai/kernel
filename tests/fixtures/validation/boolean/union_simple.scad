// Simple union tests for debugging
// Each test case is clearly separated

// Test 1: Two non-overlapping cubes
translate([0, 0, 0]) cube([10, 10, 10]);
translate([20, 0, 0]) cube([10, 10, 10]);

// Test 2: Two overlapping cubes (should merge)
translate([40, 0, 0]) union() {
    cube([10, 10, 10]);
    translate([5, 0, 0]) cube([10, 10, 10]);
}

// Test 3: Two touching cubes (edge to edge)
translate([70, 0, 0]) union() {
    cube([10, 10, 10]);
    translate([10, 0, 0]) cube([10, 10, 10]);
}

// Test 4: One cube inside another (should result in outer cube only)
translate([100, 0, 0]) union() {
    cube([20, 20, 20]);
    translate([5, 5, 5]) cube([10, 10, 10]);
}

// Test 5: Three cubes in a row
translate([130, 0, 0]) union() {
    cube([10, 10, 10]);
    translate([10, 0, 0]) cube([10, 10, 10]);
    translate([20, 0, 0]) cube([10, 10, 10]);
}

