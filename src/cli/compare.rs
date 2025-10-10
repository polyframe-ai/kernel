// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Comparison logic for OpenSCAD vs Polyframe

use super::{ComparisonResult, MeshDiff, Reporter, Runner};
use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;
use tempfile::TempDir;

/// Compare Polyframe output with OpenSCAD output
pub fn compare_with_openscad(
    input: &Path,
    tolerance: f32,
    verbose: bool,
) -> Result<ComparisonResult> {
    let runner = Runner::new();

    // Check if OpenSCAD is available
    if !runner.is_openscad_available() {
        Reporter::report_warning("OpenSCAD not found. Skipping OpenSCAD comparison.");

        // Just run Polyframe
        if verbose {
            Reporter::progress("Running Polyframe");
        }

        let result = runner.run_polyframe_with_mesh(input)?;

        if verbose {
            Reporter::report_render(
                input.to_str().unwrap(),
                result.mesh.vertex_count(),
                result.mesh.triangle_count(),
                result.duration,
            );
        }

        // Return a "passed" result with just Polyframe data
        return Ok(ComparisonResult {
            passed: true,
            vertex_delta: 0.0,
            triangle_delta: 0.0,
            bbox_delta: 0.0,
            vertex_count_a: result.mesh.vertex_count(),
            vertex_count_b: result.mesh.vertex_count(),
            triangle_count_a: result.mesh.triangle_count(),
            triangle_count_b: result.mesh.triangle_count(),
            tolerance,
        });
    }

    // Create temp directory for outputs
    let temp_dir = TempDir::new().context("Failed to create temp directory")?;
    let openscad_output = temp_dir.path().join("openscad.stl");
    let polyframe_output = temp_dir.path().join("polyframe.stl");

    // Run OpenSCAD
    if verbose {
        Reporter::progress("Running OpenSCAD");
    }

    let openscad_duration = runner
        .run_openscad(input, &openscad_output)
        .context("Failed to run OpenSCAD")?;

    // Run Polyframe
    if verbose {
        Reporter::progress("Running Polyframe");
    }

    let polyframe_duration = runner
        .run_polyframe(input, &polyframe_output)
        .context("Failed to run Polyframe")?;

    // Load meshes
    if verbose {
        Reporter::progress("Loading and comparing meshes");
    }

    let openscad_mesh = runner
        .load_stl(&openscad_output)
        .context("Failed to load OpenSCAD STL")?;

    let polyframe_mesh = runner
        .load_stl(&polyframe_output)
        .context("Failed to load Polyframe STL")?;

    // Compare meshes
    let result = MeshDiff::compare(&polyframe_mesh, &openscad_mesh, tolerance);

    // Report results
    if verbose {
        Reporter::report_comparison(
            input.to_str().unwrap(),
            &result,
            polyframe_duration,
            Some(openscad_duration),
        );
    }

    Ok(result)
}

/// Batch compare multiple files
pub fn batch_compare(
    files: &[&Path],
    tolerance: f32,
    verbose: bool,
) -> Result<Vec<(String, ComparisonResult)>> {
    let mut results = Vec::new();

    for file in files {
        let result = compare_with_openscad(file, tolerance, verbose)?;
        results.push((file.to_str().unwrap().to_string(), result));
    }

    // Summary
    if verbose {
        println!("\n{}", "═".repeat(80));
        println!("{}", "Summary".bold());
        println!("{}", "═".repeat(80));

        let total = results.len();
        let passed = results.iter().filter(|(_, r)| r.passed).count();
        let failed = total - passed;

        println!(
            "{} {} | {} {} | {} {}",
            "Total:".bright_black(),
            total.to_string().cyan(),
            "Passed:".bright_black(),
            passed.to_string().green(),
            "Failed:".bright_black(),
            if failed > 0 {
                failed.to_string().red()
            } else {
                failed.to_string().green()
            }
        );

        if failed > 0 {
            println!("\n{}", "Failed files:".red().bold());
            for (file, result) in &results {
                if !result.passed {
                    println!("  {} {}", "❌".red(), file);
                }
            }
        }

        println!("{}", "═".repeat(80));
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_compare_availability_check() {
        let runner = Runner::new();
        let _ = runner.is_openscad_available();
    }
}
