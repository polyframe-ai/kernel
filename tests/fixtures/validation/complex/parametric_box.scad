// Parametric box with lid (simplified - no variables)
union() {
    // Box
    difference() {
        cube([40, 20, 30]);
        translate([2, 2, 2]) 
            cube([36, 16, 28]);
    }
    
    // Lid
    translate([0, 25, 0]) {
        difference() {
            cube([40, 20, 2]);
            translate([2, 2, -1]) 
                cube([36, 16, 4]);
        }
    }
}

