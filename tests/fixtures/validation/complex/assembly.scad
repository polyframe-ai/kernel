// Assembly of multiple parts
// Part 1: Base
cube([40, 40, 5]);

// Part 2: Pillar
translate([20, 20, 5]) {
    difference() {
        cylinder(h=25, r=8);
        cylinder(h=27, r=5, center=true);
    }
}

// Part 3: Top plate
translate([0, 0, 30]) {
    difference() {
        cube([40, 40, 5]);
        translate([20, 20, -1]) cylinder(h=7, r=6);
    }
}

// Part 4: Decorative spheres
translate([10, 10, 30]) sphere(r=3);
translate([30, 10, 30]) sphere(r=3);
translate([10, 30, 30]) sphere(r=3);
translate([30, 30, 30]) sphere(r=3);

