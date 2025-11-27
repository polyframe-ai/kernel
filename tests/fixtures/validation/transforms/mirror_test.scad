// Mirror transformation tests
union() {
    mirror([1, 0, 0]) cube([10, 10, 10]);
    translate([15, 0, 0]) mirror([0, 1, 0]) cube([10, 10, 10]);
    translate([30, 0, 0]) mirror([0, 0, 1]) cube([10, 10, 10]);
    translate([45, 0, 0]) mirror([1, 1, 0]) cube([10, 10, 10]);
}

