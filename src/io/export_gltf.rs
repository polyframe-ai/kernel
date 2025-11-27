// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! GLTF/GLB exporter

use crate::geometry::Mesh;
use anyhow::Result;
use serde_json::json;
use std::fs::File;
use std::io::Write;

/// Export mesh to GLTF or GLB format
pub fn export(mesh: &Mesh, path: &str) -> Result<()> {
    if path.ends_with(".glb") {
        export_glb(mesh, path)
    } else {
        export_gltf_separate(mesh, path)
    }
}

/// Export mesh to GLB (binary GLTF)
fn export_glb(mesh: &Mesh, path: &str) -> Result<()> {
    let (gltf_json_val, buffer_data) = create_gltf_json(mesh)?;

    let json_string = serde_json::to_string(&gltf_json_val)?;
    let mut json_offset = json_string.len();
    align_to_multiple_of_four(&mut json_offset);
    let json_padding = json_offset - json_string.len();

    let mut buffer_offset = buffer_data.len();
    align_to_multiple_of_four(&mut buffer_offset);
    let buffer_padding = buffer_offset - buffer_data.len();

    let total_length = 12 + 8 + json_offset + 8 + buffer_offset;

    let mut file = File::create(path)?;

    // GLB header
    file.write_all(&0x46546C67u32.to_le_bytes())?; // magic: "glTF"
    file.write_all(&2u32.to_le_bytes())?; // version
    file.write_all(&(total_length as u32).to_le_bytes())?;

    // JSON chunk
    file.write_all(&(json_offset as u32).to_le_bytes())?;
    file.write_all(&0x4E4F534Au32.to_le_bytes())?; // type: "JSON"
    file.write_all(json_string.as_bytes())?;
    for _ in 0..json_padding {
        file.write_all(b" ")?;
    }

    // BIN chunk
    file.write_all(&(buffer_offset as u32).to_le_bytes())?;
    file.write_all(&0x004E4942u32.to_le_bytes())?; // type: "BIN\0"
    file.write_all(&buffer_data)?;
    for _ in 0..buffer_padding {
        file.write_all(&[0])?;
    }

    Ok(())
}

/// Export mesh to GLTF with separate .bin file
fn export_gltf_separate(mesh: &Mesh, path: &str) -> Result<()> {
    let (gltf_json_val, buffer_data) = create_gltf_json(mesh)?;

    // Write .gltf JSON file
    let json_string = serde_json::to_string_pretty(&gltf_json_val)?;
    std::fs::write(path, json_string)?;

    // Write .bin file
    let bin_path = path.replace(".gltf", ".bin");
    std::fs::write(bin_path, buffer_data)?;

    Ok(())
}

fn create_gltf_json(mesh: &Mesh) -> Result<(serde_json::Value, Vec<u8>)> {
    let mut buffer_data = Vec::new();

    // Write positions
    let position_offset = buffer_data.len();
    let (min_pos, max_pos) = calculate_bounds(mesh);
    for vertex in &mesh.vertices {
        buffer_data.extend_from_slice(&vertex.position.x.to_le_bytes());
        buffer_data.extend_from_slice(&vertex.position.y.to_le_bytes());
        buffer_data.extend_from_slice(&vertex.position.z.to_le_bytes());
    }
    let position_length = buffer_data.len() - position_offset;

    // Write normals
    let normal_offset = buffer_data.len();
    for vertex in &mesh.vertices {
        buffer_data.extend_from_slice(&vertex.normal.x.to_le_bytes());
        buffer_data.extend_from_slice(&vertex.normal.y.to_le_bytes());
        buffer_data.extend_from_slice(&vertex.normal.z.to_le_bytes());
    }
    let normal_length = buffer_data.len() - normal_offset;

    // Write indices
    let indices_offset = buffer_data.len();
    for triangle in &mesh.triangles {
        buffer_data.extend_from_slice(&(triangle.indices[0] as u32).to_le_bytes());
        buffer_data.extend_from_slice(&(triangle.indices[1] as u32).to_le_bytes());
        buffer_data.extend_from_slice(&(triangle.indices[2] as u32).to_le_bytes());
    }
    let indices_length = buffer_data.len() - indices_offset;

    // Build GLTF JSON
    let gltf = json!({
        "asset": {
            "generator": "Polyframe Kernel",
            "version": "2.0"
        },
        "scene": 0,
        "scenes": [
            {
                "nodes": [0]
            }
        ],
        "nodes": [
            {
                "mesh": 0
            }
        ],
        "meshes": [
            {
                "primitives": [
                    {
                        "attributes": {
                            "POSITION": 0,
                            "NORMAL": 1
                        },
                        "indices": 2,
                        "mode": 4
                    }
                ]
            }
        ],
        "accessors": [
            {
                "bufferView": 0,
                "byteOffset": 0,
                "componentType": 5126,
                "count": mesh.vertices.len(),
                "type": "VEC3",
                "min": [min_pos[0], min_pos[1], min_pos[2]],
                "max": [max_pos[0], max_pos[1], max_pos[2]]
            },
            {
                "bufferView": 1,
                "byteOffset": 0,
                "componentType": 5126,
                "count": mesh.vertices.len(),
                "type": "VEC3"
            },
            {
                "bufferView": 2,
                "byteOffset": 0,
                "componentType": 5125,
                "count": mesh.triangles.len() * 3,
                "type": "SCALAR"
            }
        ],
        "bufferViews": [
            {
                "buffer": 0,
                "byteOffset": position_offset,
                "byteLength": position_length,
                "target": 34962
            },
            {
                "buffer": 0,
                "byteOffset": normal_offset,
                "byteLength": normal_length,
                "target": 34962
            },
            {
                "buffer": 0,
                "byteOffset": indices_offset,
                "byteLength": indices_length,
                "target": 34963
            }
        ],
        "buffers": [
            {
                "byteLength": buffer_data.len(),
                "uri": "data.bin"
            }
        ]
    });

    Ok((gltf, buffer_data))
}

fn calculate_bounds(mesh: &Mesh) -> ([f32; 3], [f32; 3]) {
    let mut min = [f32::MAX, f32::MAX, f32::MAX];
    let mut max = [f32::MIN, f32::MIN, f32::MIN];

    for vertex in &mesh.vertices {
        min[0] = min[0].min(vertex.position.x as f32);
        min[1] = min[1].min(vertex.position.y as f32);
        min[2] = min[2].min(vertex.position.z as f32);
        max[0] = max[0].max(vertex.position.x as f32);
        max[1] = max[1].max(vertex.position.y as f32);
        max[2] = max[2].max(vertex.position.z as f32);
    }

    (min, max)
}

fn align_to_multiple_of_four(n: &mut usize) {
    *n = (*n + 3) & !3;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Primitive;
    use nalgebra::Vector3;
    use tempfile::NamedTempFile;

    #[test]
    fn test_export_glb() -> Result<()> {
        let mesh = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), true).to_mesh();

        let file = NamedTempFile::with_suffix(".glb")?;
        let path = file.path().to_str().unwrap();

        export(&mesh, path)?;

        // Verify file was created
        let metadata = std::fs::metadata(path)?;
        assert!(metadata.len() > 0);

        // Verify GLB header
        let file_content = std::fs::read(path)?;
        assert_eq!(&file_content[0..4], b"glTF");

        Ok(())
    }

    #[test]
    fn test_export_gltf() -> Result<()> {
        let mesh = Primitive::sphere(5.0, 16).to_mesh();

        let file = NamedTempFile::with_suffix(".gltf")?;
        let path = file.path().to_str().unwrap();

        export(&mesh, path)?;

        // Verify files were created
        assert!(std::path::Path::new(path).exists());
        let bin_path = path.replace(".gltf", ".bin");
        assert!(std::path::Path::new(&bin_path).exists());

        Ok(())
    }
}
