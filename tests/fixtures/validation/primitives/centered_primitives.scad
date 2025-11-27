// Centered primitives test
union() {
    cube([10, 10, 10], center=true);
    translate([15, 0, 0]) sphere(r=10);
    translate([30, 0, 0]) cylinder(h=20, r=5, center=true);
    translate([0, 15, 0]) cube([20, 20, 20], center=true);
    translate([0, 40, 0]) sphere(r=15);
    translate([0, 60, 0]) cylinder(h=30, r=8, center=true);
}

