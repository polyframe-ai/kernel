// Varied size primitives test
union() {
    cube([1, 1, 1]);
    translate([5, 0, 0]) cube([10, 10, 10]);
    translate([20, 0, 0]) cube([100, 100, 100]);
    translate([0, 15, 0]) sphere(r=1);
    translate([0, 20, 0]) sphere(r=10);
    translate([0, 35, 0]) sphere(r=50);
    translate([0, 0, 5]) cylinder(h=1, r=5);
    translate([0, 0, 10]) cylinder(h=10, r=5);
    translate([0, 0, 25]) cylinder(h=100, r=5);
}

