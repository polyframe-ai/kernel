// Demo: Complex shape combining multiple operations
// Note: Sphere subtraction from union+cylinder can cause CSG edge cases
// This version uses cylinders only for maximum robustness
difference() {
    // Main body - union of cube and cylinder
    union() {
        cube([30, 30, 20]);
        translate([15, 15, 0])
            cylinder(h=30, r=12);
    }
    
    // Subtract large cylinder from center (instead of sphere)
    translate([15, 15, 5])
        cylinder(h=15, r=8);
    
    // Subtract small cylinders as holes
    translate([7, 7, -1])
        cylinder(h=25, r=2);
    translate([23, 7, -1])
        cylinder(h=25, r=2);
    translate([7, 23, -1])
        cylinder(h=25, r=2);
    translate([23, 23, -1])
        cylinder(h=25, r=2);
}

