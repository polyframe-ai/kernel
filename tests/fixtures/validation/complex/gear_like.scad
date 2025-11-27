// Gear-like structure (simplified - no for loop)
difference() {
    union() {
        // Central hub
        cylinder(h=10, r=10);
        // Teeth (manually created)
        translate([15, 0, 0]) cube([5, 3, 10], center=true);
        rotate([0, 0, 45]) translate([15, 0, 0]) cube([5, 3, 10], center=true);
        rotate([0, 0, 90]) translate([15, 0, 0]) cube([5, 3, 10], center=true);
        rotate([0, 0, 135]) translate([15, 0, 0]) cube([5, 3, 10], center=true);
        rotate([0, 0, 180]) translate([15, 0, 0]) cube([5, 3, 10], center=true);
        rotate([0, 0, 225]) translate([15, 0, 0]) cube([5, 3, 10], center=true);
        rotate([0, 0, 270]) translate([15, 0, 0]) cube([5, 3, 10], center=true);
        rotate([0, 0, 315]) translate([15, 0, 0]) cube([5, 3, 10], center=true);
    }
    // Center hole
    cylinder(h=12, r=5, center=true);
}

