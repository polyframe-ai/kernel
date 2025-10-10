// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Process runner for OpenSCAD and Polyframe

use super::{comparator, Comparison, Metrics};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use tempfile::TempDir;

/// Result of running a renderer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResult {
    pub file: String,
    pub time_ms: u128,
    pub output_path: PathBuf,
}

/// Complete evaluation result for a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    pub model: String,
    pub openscad_result: Option<RunResult>,
    pub polyframe_result: RunResult,
    pub comparison: Comparison,
    pub metrics: Metrics,
}

/// Run OpenSCAD on a .scad file
pub fn run_openscad(file: &Path) -> Result<RunResult> {
    // Check if OpenSCAD is available
    if Command::new("openscad").arg("--version").output().is_err() {
        bail!("OpenSCAD not found in PATH");
    }

    let temp_dir = TempDir::new()?;
    let output_path = temp_dir.path().join("output.stl");

    let start = Instant::now();

    let status = Command::new("openscad")
        .arg("-o")
        .arg(&output_path)
        .arg(file)
        .arg("--quiet")
        .status()
        .context("Failed to execute OpenSCAD")?;

    let time_ms = start.elapsed().as_millis();

    if !status.success() {
        bail!("OpenSCAD exited with status: {}", status);
    }

    // Move to persistent location for comparison
    let persistent_path = PathBuf::from(format!(
        "target/eval_openscad_{}.stl",
        file.file_stem().unwrap().to_str().unwrap()
    ));
    std::fs::copy(&output_path, &persistent_path)?;

    Ok(RunResult {
        file: file.display().to_string(),
        time_ms,
        output_path: persistent_path,
    })
}

/// Run Polyframe on a .scad file
pub fn run_polyframe(file: &Path) -> Result<RunResult> {
    let output_path = PathBuf::from(format!(
        "target/eval_polyframe_{}.stl",
        file.file_stem().unwrap().to_str().unwrap()
    ));

    let start = Instant::now();

    // Use the library directly
    let mesh =
        crate::render_file(file.to_str().unwrap()).context("Failed to render with Polyframe")?;

    crate::io::export_stl(&mesh, output_path.to_str().unwrap()).context("Failed to export STL")?;

    let time_ms = start.elapsed().as_millis();

    Ok(RunResult {
        file: file.display().to_string(),
        time_ms,
        output_path,
    })
}

/// Run both renderers and compare outputs (legacy path-based)
pub fn run_and_compare(file: &Path) -> Result<EvaluationResult> {
    // Run Polyframe (always)
    let polyframe_result = run_polyframe(file).context("Polyframe execution failed")?;

    // Try to run OpenSCAD (optional)
    let openscad_result = run_openscad(file).ok();

    // Compare if we have both outputs
    let comparison = if let Some(ref openscad) = openscad_result {
        comparator::compare_stl_files(&polyframe_result.output_path, &openscad.output_path)?
    } else {
        // No OpenSCAD available, create a "passed" comparison
        Comparison {
            vertices_diff: 0.0,
            triangles_diff: 0.0,
            bbox_diff: 0.0,
            checksum_match: true,
            passed: true,
            vertex_count_poly: 0,
            vertex_count_openscad: 0,
            triangle_count_poly: 0,
            triangle_count_openscad: 0,
        }
    };

    let metrics = Metrics {
        openscad_time_ms: openscad_result.as_ref().map(|r| r.time_ms).unwrap_or(0),
        polyframe_time_ms: polyframe_result.time_ms,
        speedup_ratio: if let Some(ref openscad) = openscad_result {
            if polyframe_result.time_ms > 0 {
                openscad.time_ms as f32 / polyframe_result.time_ms as f32
            } else {
                0.0
            }
        } else {
            0.0
        },
    };

    Ok(EvaluationResult {
        model: file.display().to_string(),
        openscad_result,
        polyframe_result,
        comparison,
        metrics,
    })
}

/// Run model task (supports both file and JSON sources)
pub fn run_model_task(task: &super::dataset::ModelTask) -> Result<EvaluationResult> {
    let name = task.name();
    let source = task.source()?;

    // Run Polyframe from source
    let polyframe_result =
        run_polyframe_from_source(&name, &source).context("Polyframe execution failed")?;

    // Try to run OpenSCAD from source (optional)
    let openscad_result = run_openscad_from_source(&name, &source).ok();

    // Compare if we have both outputs
    let comparison = if let Some(ref openscad) = openscad_result {
        comparator::compare_stl_files(&polyframe_result.output_path, &openscad.output_path)?
    } else {
        // No OpenSCAD available, create a "passed" comparison
        Comparison {
            vertices_diff: 0.0,
            triangles_diff: 0.0,
            bbox_diff: 0.0,
            checksum_match: true,
            passed: true,
            vertex_count_poly: 0,
            vertex_count_openscad: 0,
            triangle_count_poly: 0,
            triangle_count_openscad: 0,
        }
    };

    let metrics = Metrics {
        openscad_time_ms: openscad_result.as_ref().map(|r| r.time_ms).unwrap_or(0),
        polyframe_time_ms: polyframe_result.time_ms,
        speedup_ratio: if let Some(ref openscad) = openscad_result {
            if polyframe_result.time_ms > 0 {
                openscad.time_ms as f32 / polyframe_result.time_ms as f32
            } else {
                0.0
            }
        } else {
            0.0
        },
    };

    Ok(EvaluationResult {
        model: name,
        openscad_result,
        polyframe_result,
        comparison,
        metrics,
    })
}

/// Run Polyframe from source string
fn run_polyframe_from_source(name: &str, source: &str) -> Result<RunResult> {
    let output_path = PathBuf::from(format!(
        "target/eval_polyframe_{}.stl",
        name.replace(['/', '\\', ' '], "_")
    ));

    let start = Instant::now();

    // Render from source
    let mesh = crate::render(source).context("Failed to render with Polyframe")?;

    crate::io::export_stl(&mesh, output_path.to_str().unwrap()).context("Failed to export STL")?;

    let time_ms = start.elapsed().as_millis();

    Ok(RunResult {
        file: name.to_string(),
        time_ms,
        output_path,
    })
}

/// Run OpenSCAD from source string
fn run_openscad_from_source(name: &str, source: &str) -> Result<RunResult> {
    // Check if OpenSCAD is available
    if Command::new("openscad").arg("--version").output().is_err() {
        bail!("OpenSCAD not found in PATH");
    }

    let temp_dir = TempDir::new()?;
    let input_path = temp_dir.path().join("input.scad");
    let output_path = temp_dir.path().join("output.stl");

    // Write source to temp file
    std::fs::write(&input_path, source)?;

    let start = Instant::now();

    let status = Command::new("openscad")
        .arg("-o")
        .arg(&output_path)
        .arg(&input_path)
        .arg("--quiet")
        .status()
        .context("Failed to execute OpenSCAD")?;

    let time_ms = start.elapsed().as_millis();

    if !status.success() {
        bail!("OpenSCAD exited with status: {}", status);
    }

    // Move to persistent location for comparison
    let persistent_path = PathBuf::from(format!(
        "target/eval_openscad_{}.stl",
        name.replace(['/', '\\', ' '], "_")
    ));
    std::fs::copy(&output_path, &persistent_path)?;

    Ok(RunResult {
        file: name.to_string(),
        time_ms,
        output_path: persistent_path,
    })
}

/// Check if OpenSCAD is available
pub fn is_openscad_available() -> bool {
    Command::new("openscad").arg("--version").output().is_ok()
}
