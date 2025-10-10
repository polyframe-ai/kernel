// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Subprocess execution runner for OpenSCAD and Polyframe

use crate::geometry::Mesh;
use crate::io;
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

/// Result of a render operation
pub struct RenderResult {
    pub mesh: Mesh,
    pub duration: Duration,
}

/// Runner for executing rendering operations
pub struct Runner {
    timeout: Option<Duration>,
}

impl Runner {
    pub fn new() -> Self {
        Self { timeout: None }
    }

    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            timeout: Some(timeout),
        }
    }

    /// Run OpenSCAD to generate STL
    pub fn run_openscad(&self, input: &Path, output: &Path) -> Result<Duration> {
        if !self.is_openscad_available() {
            bail!("OpenSCAD is not installed or not in PATH");
        }

        let start = Instant::now();

        let status = Command::new("openscad")
            .arg("-o")
            .arg(output)
            .arg(input)
            .arg("--quiet")
            .status()
            .context("Failed to execute OpenSCAD")?;

        if !status.success() {
            bail!("OpenSCAD exited with status: {}", status);
        }

        Ok(start.elapsed())
    }

    /// Run Polyframe kernel to generate STL
    pub fn run_polyframe(&self, input: &Path, output: &Path) -> Result<Duration> {
        let start = Instant::now();

        // Render the mesh
        let mesh = crate::render_file(input.to_str().unwrap())
            .context("Failed to render with Polyframe")?;

        // Export to STL
        io::export_stl(&mesh, output.to_str().unwrap()).context("Failed to export STL")?;

        Ok(start.elapsed())
    }

    /// Run Polyframe and return mesh with timing
    pub fn run_polyframe_with_mesh(&self, input: &Path) -> Result<RenderResult> {
        let start = Instant::now();

        let mesh = crate::render_file(input.to_str().unwrap())
            .context("Failed to render with Polyframe")?;

        let duration = start.elapsed();

        Ok(RenderResult { mesh, duration })
    }

    /// Load STL file into mesh
    pub fn load_stl(&self, path: &Path) -> Result<Mesh> {
        use crate::geometry::{Triangle, Vertex};
        use nalgebra::{Point3, Vector3};
        use std::fs::File;
        use stl_io::read_stl;

        let mut file = File::open(path).context(format!("Failed to open STL file: {:?}", path))?;

        let stl = read_stl(&mut file).context("Failed to read STL file")?;

        let mut mesh = Mesh::new();

        // stl_io returns an IndexedMesh with vertices and faces
        for face in &stl.faces {
            let vertices = &stl.vertices;

            // Get the three vertices of this face
            let v0_pos = &vertices[face.vertices[0]];
            let v1_pos = &vertices[face.vertices[1]];
            let v2_pos = &vertices[face.vertices[2]];

            let normal = Vector3::new(face.normal[0], face.normal[1], face.normal[2]);

            let v0 = mesh.add_vertex(Vertex::new(
                Point3::new(v0_pos[0], v0_pos[1], v0_pos[2]),
                normal,
            ));

            let v1 = mesh.add_vertex(Vertex::new(
                Point3::new(v1_pos[0], v1_pos[1], v1_pos[2]),
                normal,
            ));

            let v2 = mesh.add_vertex(Vertex::new(
                Point3::new(v2_pos[0], v2_pos[1], v2_pos[2]),
                normal,
            ));

            mesh.add_triangle(Triangle::new([v0, v1, v2]));
        }

        Ok(mesh)
    }

    /// Check if OpenSCAD is available
    pub fn is_openscad_available(&self) -> bool {
        Command::new("openscad").arg("--version").output().is_ok()
    }
}

impl Default for Runner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runner_creation() {
        let runner = Runner::new();
        assert!(runner.timeout.is_none());
    }

    #[test]
    fn test_openscad_check() {
        let runner = Runner::new();
        // This may pass or fail depending on whether OpenSCAD is installed
        let _ = runner.is_openscad_available();
    }
}
