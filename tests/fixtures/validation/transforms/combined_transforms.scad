// Combined transformation tests
union() {
    translate([10, 0, 0]) rotate([45, 0, 0]) scale([2, 1, 1]) cube([10, 10, 10]);
    translate([40, 0, 0]) rotate([0, 45, 0]) translate([10, 0, 0]) scale([1, 2, 1]) cube([10, 10, 10]);
    translate([70, 0, 0]) scale([1.5, 1.5, 1.5]) translate([-5, -5, -5]) rotate([0, 0, 45]) cube([10, 10, 10]);
}

