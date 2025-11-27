// Mechanical part - bracket-like structure
difference() {
    union() {
        // Base plate
        cube([50, 30, 5]);
        // Vertical support
        translate([10, 0, 5]) cube([10, 30, 20]);
        // Top plate
        translate([0, 0, 25]) cube([50, 30, 5]);
    }
    // Holes
    translate([15, 15, -1]) cylinder(h=32, r=3);
    translate([35, 15, -1]) cylinder(h=32, r=3);
    // Corner cutouts
    translate([-1, -1, 5]) cube([12, 12, 20]);
    translate([39, -1, 5]) cube([12, 12, 20]);
}

