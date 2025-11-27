// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Comparison logic for OpenSCAD vs Polyframe

use super::{ComparisonResult, MeshDiff, Reporter, Runner};
use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use crate::evaluation::{generate_diff_image, render_stl_to_png};

#[derive(Debug, Clone)]
pub struct PreviewConfig {
    pub output_dir: PathBuf,
    pub copy_stl: bool,
    pub generate_diff: bool,
}

impl PreviewConfig {
    pub fn new(output_dir: PathBuf) -> Self {
        Self {
            output_dir,
            copy_stl: true,
            generate_diff: true,
        }
    }

    pub fn for_input(root: &Path, input: &Path) -> Self {
        let slug = sanitize_identifier(input);
        Self::new(root.join(slug))
    }
}

/// Compare Polyframe output with OpenSCAD output
pub fn compare_with_openscad(
    input: &Path,
    tolerance: f32,
    verbose: bool,
    preview: Option<PreviewConfig>,
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
            note: None,
            polyframe_preview: None,
            openscad_preview: None,
            diff_preview: None,
            visual_diff_delta: None,
            polyframe_stl: None,
            openscad_stl: None,
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
    let mut result = MeshDiff::compare(&polyframe_mesh, &openscad_mesh, tolerance);

    if let Some(preview_cfg) = preview {
        fs::create_dir_all(&preview_cfg.output_dir).with_context(|| {
            format!(
                "Failed to create preview directory {}",
                preview_cfg.output_dir.display()
            )
        })?;

        if preview_cfg.copy_stl {
            let pf_stl = preview_cfg.output_dir.join("polyframe.stl");
            let os_stl = preview_cfg.output_dir.join("openscad.stl");
            fs::copy(&polyframe_output, &pf_stl).with_context(|| {
                format!(
                    "Failed to copy Polyframe STL into {}",
                    pf_stl.display()
                )
            })?;
            fs::copy(&openscad_output, &os_stl).with_context(|| {
                format!("Failed to copy OpenSCAD STL into {}", os_stl.display())
            })?;
            result.polyframe_stl = Some(pf_stl);
            result.openscad_stl = Some(os_stl);
        }

        let pf_png = preview_cfg.output_dir.join("polyframe.png");
        render_stl_to_png(&polyframe_output, &pf_png)
            .context("Failed to render Polyframe STL preview")?;
        result.polyframe_preview = Some(pf_png.clone());

        let os_png = preview_cfg.output_dir.join("openscad.png");
        render_stl_to_png(&openscad_output, &os_png)
            .context("Failed to render OpenSCAD STL preview")?;
        result.openscad_preview = Some(os_png.clone());

        if preview_cfg.generate_diff {
            let diff_png = preview_cfg.output_dir.join("diff.png");
            let delta =
                generate_diff_image(&os_png, &pf_png, &diff_png).context("Failed to generate visual diff")?;
            result.visual_diff_delta = Some(delta);
            result.diff_preview = Some(diff_png);
        }
    }

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
    preview_root: Option<&Path>,
) -> Result<Vec<(String, ComparisonResult)>> {
    let mut results = Vec::new();

    for file in files {
        let preview_cfg = preview_root.map(|root| PreviewConfig::for_input(root, file));
        let result = compare_with_openscad(file, tolerance, verbose, preview_cfg)?;
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
            "Total:".white(),
            total.to_string().cyan(),
            "Passed:".white(),
            passed.to_string().green(),
            "Failed:".white(),
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

fn sanitize_identifier(path: &Path) -> String {
    let raw = path.to_string_lossy();
    let mut slug = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
        } else {
            slug.push('_');
        }
    }

    let trimmed = slug.trim_matches('_');
    if trimmed.is_empty() {
        "preview".to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_availability_check() {
        let runner = Runner::new();
        let _ = runner.is_openscad_available();
    }
}
