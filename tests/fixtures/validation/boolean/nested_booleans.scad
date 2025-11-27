// Nested boolean operations
union() {
    difference() {
        union() {
            cube([20, 20, 20]);
            translate([10, 10, 20]) sphere(r=8);
        }
        translate([10, 10, 10]) sphere(r=6);
    }
    
    translate([40, 0, 0]) union() {
        difference() {
            cube([20, 20, 20]);
            translate([10, 10, 10]) sphere(r=8);
        }
        translate([10, 10, 20]) cylinder(h=5, r=5);
    }
    
    translate([80, 0, 0]) intersection() {
        difference() {
            cube([25, 25, 25], center=true);
            sphere(r=12);
        }
        cylinder(h=30, r=10, center=true);
    }
}

