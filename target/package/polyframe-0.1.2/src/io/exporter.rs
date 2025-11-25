// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Mesh exporters for various formats

use crate::geometry::Mesh;
use anyhow::{Context, Result};
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Export mesh to STL format
pub fn export_stl(mesh: &Mesh, path: &str) -> Result<()> {
    let file_path = Path::new(path);

    // Determine if binary or ASCII based on extension
    if path.ends_with(".stl") {
        export_stl_binary(mesh, file_path)
    } else {
        export_stl_ascii(mesh, file_path)
    }
}

fn export_stl_binary(mesh: &Mesh, path: &Path) -> Result<()> {
    use stl_io::{Normal, Triangle as StlTriangle, Vertex as StlVertex};

    let triangles: Vec<StlTriangle> = mesh
        .triangles
        .iter()
        .map(|tri| {
            let v0 = &mesh.vertices[tri.indices[0]];
            let v1 = &mesh.vertices[tri.indices[1]];
            let v2 = &mesh.vertices[tri.indices[2]];

            // Calculate normal from vertices
            let normal = (v0.normal + v1.normal + v2.normal) / 3.0;

            StlTriangle {
                normal: Normal::new([normal.x, normal.y, normal.z]),
                vertices: [
                    StlVertex::new([v0.position.x, v0.position.y, v0.position.z]),
                    StlVertex::new([v1.position.x, v1.position.y, v1.position.z]),
                    StlVertex::new([v2.position.x, v2.position.y, v2.position.z]),
                ],
            }
        })
        .collect();

    let mut file = File::create(path).context("Failed to create STL file")?;

    stl_io::write_stl(&mut file, triangles.iter()).context("Failed to write STL file")?;

    Ok(())
}

fn export_stl_ascii(mesh: &Mesh, path: &Path) -> Result<()> {
    let mut file = File::create(path).context("Failed to create STL file")?;

    writeln!(file, "solid mesh")?;

    for tri in &mesh.triangles {
        let v0 = &mesh.vertices[tri.indices[0]];
        let v1 = &mesh.vertices[tri.indices[1]];
        let v2 = &mesh.vertices[tri.indices[2]];

        let normal = (v0.normal + v1.normal + v2.normal) / 3.0;

        writeln!(
            file,
            "  facet normal {} {} {}",
            normal.x, normal.y, normal.z
        )?;
        writeln!(file, "    outer loop")?;
        writeln!(
            file,
            "      vertex {} {} {}",
            v0.position.x, v0.position.y, v0.position.z
        )?;
        writeln!(
            file,
            "      vertex {} {} {}",
            v1.position.x, v1.position.y, v1.position.z
        )?;
        writeln!(
            file,
            "      vertex {} {} {}",
            v2.position.x, v2.position.y, v2.position.z
        )?;
        writeln!(file, "    endloop")?;
        writeln!(file, "  endfacet")?;
    }

    writeln!(file, "endsolid mesh")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Primitive;
    use nalgebra::Vector3;
    use tempfile::NamedTempFile;

    #[test]
    fn test_export_stl() -> Result<()> {
        let mesh = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), true).to_mesh();

        let file = NamedTempFile::new()?;
        let path = file.path().to_str().unwrap();

        export_stl(&mesh, path)?;

        // Verify file was created
        assert!(Path::new(path).exists());

        Ok(())
    }
}
