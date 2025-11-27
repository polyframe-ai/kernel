// Cylinder variant tests
union() {
    cylinder(h=20, r=5);
    translate([15, 0, 0]) cylinder(h=20, r=5);
    translate([30, 0, 0]) cylinder(h=20, r1=10, r2=5);
    translate([45, 0, 0]) cylinder(h=20, r1=10, r2=5);
    translate([0, 15, 0]) cylinder(h=20, r=5, center=true);
    translate([15, 15, 0]) cylinder(h=20, r=5);
    translate([30, 15, 0]) cylinder(h=20, r=5);
}

