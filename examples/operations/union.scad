// Union of multiple shapes
union() {
    cube([10, 10, 10]);
    translate([8, 0, 0])
        cube([10, 10, 10]);
    translate([4, 8, 0])
        cylinder(h=10, r=3);
}

