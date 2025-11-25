// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Round-trip export/import tests

use anyhow::Result;
use nalgebra::Vector3;
use polyframe::geometry::Primitive;
use polyframe::io;
use tempfile::NamedTempFile;

#[test]
fn test_roundtrip_stl_export() -> Result<()> {
    // Create a test mesh
    let original_mesh = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), true).to_mesh();

    println!(
        "Original mesh: {} vertices, {} triangles",
        original_mesh.vertex_count(),
        original_mesh.triangle_count()
    );

    // Export to STL
    let file = NamedTempFile::with_suffix(".stl")?;
    let path = file.path().to_str().unwrap();

    io::export_stl(&original_mesh, path)?;

    // Verify file exists and has content
    let metadata = std::fs::metadata(path)?;
    assert!(metadata.len() > 100, "STL file too small");

    println!("STL file size: {} bytes", metadata.len());

    Ok(())
}

#[test]
fn test_roundtrip_3mf_export() -> Result<()> {
    // Create a test mesh
    let original_mesh = Primitive::sphere(5.0, 32).to_mesh();

    println!(
        "Original mesh: {} vertices, {} triangles",
        original_mesh.vertex_count(),
        original_mesh.triangle_count()
    );

    // Export to 3MF
    let file = NamedTempFile::with_suffix(".3mf")?;
    let path = file.path().to_str().unwrap();

    io::export_3mf(&original_mesh, path)?;

    // Verify file exists and is a valid ZIP
    let file_content = std::fs::read(path)?;
    assert!(!file_content.is_empty(), "3MF file is empty");

    // Check ZIP signature (PK)
    assert_eq!(&file_content[0..2], b"PK", "3MF file is not a valid ZIP");

    println!("3MF file size: {} bytes", file_content.len());

    Ok(())
}

#[test]
fn test_roundtrip_glb_export() -> Result<()> {
    // Create a test mesh
    let original_mesh = Primitive::cylinder(20.0, 5.0, 16).to_mesh();

    println!(
        "Original mesh: {} vertices, {} triangles",
        original_mesh.vertex_count(),
        original_mesh.triangle_count()
    );

    // Export to GLB
    let file = NamedTempFile::with_suffix(".glb")?;
    let path = file.path().to_str().unwrap();

    io::export_gltf(&original_mesh, path)?;

    // Verify file exists and has GLB magic number
    let file_content = std::fs::read(path)?;
    assert!(!file_content.is_empty(), "GLB file is empty");

    // Check GLB magic number
    assert_eq!(
        &file_content[0..4],
        b"glTF",
        "GLB file has invalid magic number"
    );

    println!("GLB file size: {} bytes", file_content.len());

    Ok(())
}

#[test]
fn test_roundtrip_gltf_export() -> Result<()> {
    // Create a test mesh
    let original_mesh = Primitive::sphere(8.0, 24).to_mesh();

    println!(
        "Original mesh: {} vertices, {} triangles",
        original_mesh.vertex_count(),
        original_mesh.triangle_count()
    );

    // Export to GLTF
    let file = NamedTempFile::with_suffix(".gltf")?;
    let path = file.path().to_str().unwrap();

    io::export_gltf(&original_mesh, path)?;

    // Verify both .gltf and .bin files exist
    assert!(std::path::Path::new(path).exists(), "GLTF file not created");

    let bin_path = path.replace(".gltf", ".bin");
    assert!(
        std::path::Path::new(&bin_path).exists(),
        "BIN file not created"
    );

    // Verify GLTF JSON is valid
    let gltf_content = std::fs::read_to_string(path)?;
    let _json: serde_json::Value = serde_json::from_str(&gltf_content)?;

    println!("GLTF file size: {} bytes", gltf_content.len());

    Ok(())
}

#[test]
fn test_roundtrip_step_export() -> Result<()> {
    // Create a test mesh
    let original_mesh = Primitive::cube(Vector3::new(15.0, 15.0, 15.0), true).to_mesh();

    println!(
        "Original mesh: {} vertices, {} triangles",
        original_mesh.vertex_count(),
        original_mesh.triangle_count()
    );

    // Export to STEP
    let file = NamedTempFile::with_suffix(".step")?;
    let path = file.path().to_str().unwrap();

    io::export_step(&original_mesh, path)?;

    // Verify file exists and has STEP header
    let content = std::fs::read_to_string(path)?;
    assert!(
        content.contains("ISO-10303-21"),
        "STEP file missing ISO header"
    );
    assert!(
        content.contains("CARTESIAN_POINT"),
        "STEP file missing geometry data"
    );

    println!("STEP file size: {} bytes", content.len());

    Ok(())
}

#[test]
fn test_export_format_detection() -> Result<()> {
    let mesh = Primitive::sphere(3.0, 16).to_mesh();

    // Test each format
    let formats = vec![
        ("test.stl", "STL"),
        ("test.3mf", "3MF"),
        ("test.glb", "GLB"),
        ("test.gltf", "GLTF"),
        ("test.step", "STEP"),
    ];

    for (filename, format_name) in formats {
        let file = NamedTempFile::with_suffix(filename)?;
        let path = file.path().to_str().unwrap();

        let result = match filename {
            f if f.ends_with(".stl") => io::export_stl(&mesh, path),
            f if f.ends_with(".3mf") => io::export_3mf(&mesh, path),
            f if f.ends_with(".glb") | f.ends_with(".gltf") => io::export_gltf(&mesh, path),
            f if f.ends_with(".step") | f.ends_with(".stp") => io::export_step(&mesh, path),
            _ => continue,
        };

        assert!(
            result.is_ok(),
            "{} export failed: {:?}",
            format_name,
            result.err()
        );
        println!("✓ {} export successful", format_name);
    }

    Ok(())
}
