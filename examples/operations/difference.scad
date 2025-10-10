// Difference operation - cube with sphere subtracted
difference() {
    cube([20, 20, 20]);
    translate([10, 10, 10])
        sphere(r=12);
}

