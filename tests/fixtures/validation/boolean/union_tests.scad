// Union operation tests
union() {
    union() {
        cube([10, 10, 10]);
        translate([8, 0, 0]) cube([10, 10, 10]);
    }
    
    translate([30, 0, 0]) union() {
        cube([15, 15, 15]);
        translate([7.5, 7.5, 15]) sphere(r=8);
    }
    
    translate([60, 0, 0]) union() {
        cube([20, 20, 20]);
        translate([10, 10, 20]) sphere(r=8);
        translate([10, 10, -8]) cylinder(h=8, r=5);
    }
}

