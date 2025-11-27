// Difference operation tests
union() {
    difference() {
        cube([20, 20, 20]);
        translate([10, 10, 10]) sphere(r=8);
    }
    
    translate([30, 0, 0]) difference() {
        cube([20, 20, 20], center=true);
        cylinder(h=25, r=8, center=true);
    }
    
    translate([60, 0, 0]) difference() {
        cube([30, 30, 30]);
        translate([15, 15, 15]) sphere(r=12);
        translate([15, 15, -1]) cylinder(h=32, r=4);
        translate([15, 15, 15]) rotate([90, 0, 0]) cylinder(h=32, r=4, center=true);
    }
}

