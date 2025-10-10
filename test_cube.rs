use polyframe::{parse_scad, ast::Evaluator};

fn main() {
    // Test non-centered cube (OpenSCAD default)
    let ast = parse_scad("cube([10, 10, 10]);").unwrap();
    let evaluator = Evaluator::new();
    let mesh = evaluator.evaluate(&ast).unwrap();
    let bbox = mesh.bounding_box();
    
    println!("Non-centered cube([10, 10, 10]):");
    println!("  BBox min: ({}, {}, {})", bbox.min.x, bbox.min.y, bbox.min.z);
    println!("  BBox max: ({}, {}, {})", bbox.max.x, bbox.max.y, bbox.max.z);
    println!();
    
    // Test centered cube
    let ast2 = parse_scad("cube([10, 10, 10], center=true);").unwrap();
    let mesh2 = evaluator.evaluate(&ast2).unwrap();
    let bbox2 = mesh2.bounding_box();
    
    println!("Centered cube([10, 10, 10], center=true):");
    println!("  BBox min: ({}, {}, {})", bbox2.min.x, bbox2.min.y, bbox2.min.z);
    println!("  BBox max: ({}, {}, {})", bbox2.max.x, bbox2.max.y, bbox2.max.z);
}
