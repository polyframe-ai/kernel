// Intersection operation tests
union() {
    intersection() {
        cube([20, 20, 20]);
        sphere(r=15);
    }
    
    translate([30, 0, 0]) intersection() {
        cube([20, 20, 20], center=true);
        cylinder(h=30, r=10, center=true);
    }
    
    translate([60, 0, 0]) intersection() {
        cube([30, 30, 30], center=true);
        sphere(r=20);
        cylinder(h=40, r=15, center=true);
    }
}

