// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Unified validation report generator

use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::path::Path;

use super::types::{TestStatus, ValidationReport};

/// Unified validation reporter
pub struct ValidationReporter;

impl ValidationReporter {
    /// Write JSON report
    pub fn write_json(report: &ValidationReport, path: impl AsRef<Path>) -> Result<()> {
        let json = serde_json::to_string_pretty(report)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Write Markdown report
    pub fn write_markdown(report: &ValidationReport, path: impl AsRef<Path>) -> Result<()> {
        let mut md = String::new();

        // Header
        md.push_str("# Polyframe Kernel Validation Report\n\n");
        md.push_str(&format!("**Generated:** {}\n\n", report.timestamp));
        md.push_str("---\n\n");

        // Summary
        md.push_str("## Summary\n\n");
        md.push_str(&format!("- **Total Suites:** {}\n", report.total_suites));
        md.push_str(&format!("- **Total Tests:** {}\n", report.total_tests));
        md.push_str(&format!("- **Passed:** {} ({:.1}%)\n", report.total_passed, report.overall_pass_rate()));
        md.push_str(&format!("- **Failed:** {}\n", report.total_failed));
        md.push_str(&format!("- **Errors:** {}\n", report.total_errors));
        md.push_str(&format!("- **Skipped:** {}\n", report.total_skipped));
        md.push_str(&format!("- **Total Duration:** {:.2}s\n", report.total_duration.as_secs_f64()));
        md.push_str(&format!("- **Compatibility Score:** {:.1}%\n\n", report.compatibility_score));
        md.push_str("---\n\n");

        // Suite results
        md.push_str("## Test Suites\n\n");
        for suite_result in &report.suite_results {
            md.push_str(&format!("### {}\n\n", suite_result.suite.as_str().to_uppercase()));
            md.push_str(&format!("- **Total:** {}\n", suite_result.total));
            md.push_str(&format!("- **Passed:** {} ({:.1}%)\n", suite_result.passed, suite_result.pass_rate()));
            md.push_str(&format!("- **Failed:** {}\n", suite_result.failed));
            md.push_str(&format!("- **Errors:** {}\n", suite_result.errors));
            md.push_str(&format!("- **Duration:** {:.2}s\n\n", suite_result.duration.as_secs_f64()));

            // Failed tests
            let failed: Vec<_> = suite_result.results.iter()
                .filter(|r| matches!(r.status(), TestStatus::Failed | TestStatus::Error))
                .collect();

            if !failed.is_empty() {
                md.push_str("#### Failed Tests\n\n");
                for result in failed {
                    match result {
                        super::types::ValidationResult::Unit(r) => {
                            md.push_str(&format!("- `{}`\n", r.name));
                            if let Some(ref err) = r.error {
                                md.push_str(&format!("  - Error: {}\n", err));
                            }
                        }
                        super::types::ValidationResult::Integration(r) => {
                            md.push_str(&format!("- `{}`\n", r.name));
                            if let Some(ref err) = r.error {
                                md.push_str(&format!("  - Error: {}\n", err));
                            }
                        }
                        super::types::ValidationResult::Evaluation(r) => {
                            md.push_str(&format!("- `{}`\n", r.model));
                            if let Some(ref err) = r.error {
                                md.push_str(&format!("  - Error: {}\n", err));
                            }
                        }
                        super::types::ValidationResult::Comparison(r) => {
                            md.push_str(&format!("- `{}`\n", r.file.display()));
                            md.push_str(&format!("  - Vertex delta: {:.2}%\n", r.vertex_delta));
                            md.push_str(&format!("  - Triangle delta: {:.2}%\n", r.triangle_delta));
                            if let Some(delta) = r.visual_diff_delta {
                                md.push_str(&format!("  - Visual diff: {:.2}%\n", delta));
                            }
                            if let Some(ref preview) = r.polyframe_preview {
                                md.push_str(&format!("  - Polyframe preview: `{}`\n", preview.display()));
                            }
                            if let Some(ref preview) = r.openscad_preview {
                                md.push_str(&format!("  - OpenSCAD preview: `{}`\n", preview.display()));
                            }
                            if let Some(ref preview) = r.diff_preview {
                                md.push_str(&format!("  - Diff image: `{}`\n", preview.display()));
                            }
                            if let Some(ref err) = r.error {
                                md.push_str(&format!("  - Error: {}\n", err));
                            }
                        }
                        super::types::ValidationResult::Fuzz(r) => {
                            md.push_str(&format!("- Fuzz test failures: {}\n", r.errors.len()));
                            for err in &r.errors {
                                md.push_str(&format!("  - {}\n", err));
                            }
                        }
                        super::types::ValidationResult::Regression(r) => {
                            md.push_str(&format!("- `{}`\n", r.file.display()));
                            if let Some(ref err) = r.error {
                                md.push_str(&format!("  - Error: {}\n", err));
                            }
                        }
                    }
                }
                md.push_str("\n");
            }

            md.push_str("---\n\n");
        }

        fs::write(path, md)?;
        Ok(())
    }

    /// Print terminal summary
    pub fn print_summary(report: &ValidationReport) {
        Self::print_summary_with_verbose(report, false)
    }

    /// Print terminal summary with optional verbose mode
    pub fn print_summary_with_verbose(report: &ValidationReport, verbose: bool) {
        println!("\n{}", "═".repeat(80).white());
        println!("{}", "Validation Report".bold());
        println!("{}", "═".repeat(80).white());
        println!("  {} {}", "Timestamp:".white(), report.timestamp.cyan());
        println!("  {} {}", "Total Suites:".white(), report.total_suites.to_string().cyan());
        println!("  {} {}", "Total Tests:".white(), report.total_tests.to_string().cyan());
        println!(
            "  {} {} ({:.1}%)",
            "Passed:".white(),
            report.total_passed.to_string().green(),
            report.overall_pass_rate()
        );
        println!(
            "  {} {}",
            "Failed:".white(),
            if report.total_failed > 0 {
                report.total_failed.to_string().red()
            } else {
                report.total_failed.to_string().green()
            }
        );
        println!(
            "  {} {}",
            "Errors:".white(),
            if report.total_errors > 0 {
                report.total_errors.to_string().red()
            } else {
                report.total_errors.to_string().green()
            }
        );
        println!(
            "  {} {}",
            "Skipped:".white(),
            report.total_skipped.to_string().yellow()
        );
        println!(
            "  {} {:.2}s",
            "Duration:".white(),
            report.total_duration.as_secs_f64()
        );
        println!(
            "  {} {:.1}%",
            "Compatibility Score:".white(),
            if report.compatibility_score >= 90.0 {
                report.compatibility_score.to_string().green()
            } else if report.compatibility_score >= 70.0 {
                report.compatibility_score.to_string().yellow()
            } else {
                report.compatibility_score.to_string().red()
            }
        );

        println!("\n{}", "Suite Results".bold());
        println!("{}", "─".repeat(80).white());

        for suite_result in &report.suite_results {
            let status_icon = if suite_result.failed == 0 && suite_result.errors == 0 {
                "✓".green()
            } else {
                "✗".red()
            };

            println!(
                "  {} {}: {} passed, {} failed, {} errors ({:.1}%)",
                status_icon,
                suite_result.suite.as_str().to_uppercase().cyan(),
                suite_result.passed.to_string().green(),
                if suite_result.failed > 0 {
                    suite_result.failed.to_string().red()
                } else {
                    suite_result.failed.to_string().white()
                },
                if suite_result.errors > 0 {
                    suite_result.errors.to_string().red()
                } else {
                    suite_result.errors.to_string().white()
                },
                suite_result.pass_rate()
            );
        }

        if report.has_failures() {
            println!("\n{}", "Failed Tests".red().bold());
            println!("{}", "─".repeat(80).white());

            for suite_result in &report.suite_results {
                let failed_count = suite_result.results.iter()
                    .filter(|r| matches!(r.status(), TestStatus::Failed | TestStatus::Error))
                    .count();
                
                if failed_count > 0 {
                    println!("\n{} {} ({} failures)", 
                        "Suite:".bold(), 
                        suite_result.suite.as_str().to_uppercase().cyan(),
                        failed_count.to_string().red()
                    );
                    println!("{}", "─".repeat(80).white());
                }

                for result in &suite_result.results {
                    if matches!(result.status(), TestStatus::Failed | TestStatus::Error) {
                        Self::print_failure_details(result, verbose);
                    }
                }
            }
        }

        println!("{}", "═".repeat(80).white());
    }

    /// Print detailed failure information for a single test result
    fn print_failure_details(result: &super::types::ValidationResult, verbose: bool) {
        match result {
            super::types::ValidationResult::Unit(r) => {
                println!("\n  {} {}", "❌".red(), r.name.bold());
                println!("     {}: {:?}", "Status".white(), r.status);
                println!("     {}: {:.3}s", "Duration".white(), r.duration.as_secs_f64().to_string().white());
                if let Some(ref err) = r.error {
                    println!("     {}:", "Error".red().bold());
                    for line in err.lines() {
                        println!("       {}", line.white());
                    }
                } else if verbose {
                    println!("     {}: No error message available", "Note".yellow());
                }
            }
            super::types::ValidationResult::Integration(r) => {
                println!("\n  {} {}", "❌".red(), r.name.bold());
                println!("     {}: {:?}", "Status".white(), r.status);
                println!("     {}: {:.3}s", "Duration".white(), r.duration.as_secs_f64().to_string().white());
                if let Some(ref file) = r.file {
                    println!("     {}: {}", "File".white(), file.display());
                }
                if let Some(ref err) = r.error {
                    println!("     {}:", "Error".red().bold());
                    for line in err.lines() {
                        println!("       {}", line.white());
                    }
                } else if verbose {
                    println!("     {}: No error message available", "Note".yellow());
                }
            }
            super::types::ValidationResult::Evaluation(r) => {
                println!("\n  {} {}", "❌".red(), r.model.bold());
                println!("     {}: {:?}", "Status".white(), r.status);
                println!("     {}: {:.3}s", "Duration".white(), r.duration.as_secs_f64().to_string().white());
                println!("     {}: {}", "Comparison Passed".white(), 
                    if r.comparison_passed { "Yes".green() } else { "No".red() });
                
                if let Some(ref metrics) = r.metrics {
                    println!("     {}: {:.1}×", "Speedup".white(), metrics.speedup_ratio.to_string().white());
                    println!("     {}: {}ms (Polyframe), {}ms (OpenSCAD)", 
                        "Timing".white(),
                        metrics.polyframe_time_ms.to_string().white(),
                        metrics.openscad_time_ms.to_string().white()
                    );
                }
                
                if let Some(ref err) = r.error {
                    println!("     {}:", "Error".red().bold());
                    for line in err.lines() {
                        println!("       {}", line.white());
                    }
                } else if !r.comparison_passed && verbose {
                    println!("     {}: Comparison failed but no error message", "Note".yellow());
                }
            }
            super::types::ValidationResult::Comparison(r) => {
                println!("\n  {} {}", "❌".red(), r.file.display().to_string().bold());
                println!("     {}: {:?}", "Status".white(), r.status);
                println!("     {}: {:.3}s", "Duration".white(), r.duration.as_secs_f64().to_string().white());
                println!("     {}: {}", "Comparison Passed".white(), 
                    if r.comparison_passed { "Yes".green() } else { "No".red() });
                
                println!("     {}:", "Deltas".white());
                // vertex_delta is stored as ratio (0.23 = 23%), convert to percentage for display
                println!("       {}: {:.2}%", "Vertex Delta".white(), (r.vertex_delta * 100.0).to_string().white());
                println!("       {}: {:.2}%", "Triangle Delta".white(), (r.triangle_delta * 100.0).to_string().white());
                if let Some(delta) = r.visual_diff_delta {
                    println!("       {}: {:.2}%", "Visual Diff".white(), delta.to_string().white());
                }
                if let Some(ref preview) = r.polyframe_preview {
                    println!("       {}: {}", "Polyframe Preview".white(), preview.display());
                }
                if let Some(ref preview) = r.openscad_preview {
                    println!("       {}: {}", "OpenSCAD Preview".white(), preview.display());
                }
                if let Some(ref preview) = r.diff_preview {
                    println!("       {}: {}", "Diff Image".white(), preview.display());
                }
                
                // Show thresholds
                if verbose {
                    println!("     {}:", "Thresholds".white());
                    println!("       {}: 2.0%", "Vertex Threshold".white());
                    println!("       {}: 2.0%", "Triangle Threshold".white());
                    
                    // Note: vertex_delta is stored as ratio (0.02 = 2%), not percentage
                    if r.vertex_delta > 0.02 {
                        println!("       {}: Vertex delta exceeds threshold!", "⚠️".yellow());
                    }
                    if r.triangle_delta > 0.02 {
                        println!("       {}: Triangle delta exceeds threshold!", "⚠️".yellow());
                    }
                }
                
                if let Some(ref err) = r.error {
                    println!("     {}:", "Error".red().bold());
                    for line in err.lines() {
                        println!("       {}", line.white());
                    }
                } else if !r.comparison_passed && verbose {
                    println!("     {}: Comparison failed due to delta thresholds", "Note".yellow());
                }
            }
            super::types::ValidationResult::Fuzz(r) => {
                if !r.errors.is_empty() {
                    println!("\n  {} Fuzz Test Results", "❌".red());
                    println!("     {}: {}", "Generated".white(), r.generated_count.to_string().white());
                    println!("     {}: {} ({:.1}%)", "Parse Success".white(), 
                        r.parse_success_count.to_string().white(),
                        (r.parse_success_count as f32 / r.generated_count as f32) * 100.0
                    );
                    println!("     {}: {} ({:.1}%)", "Render Success".white(),
                        r.render_success_count.to_string().white(),
                        (r.render_success_count as f32 / r.generated_count as f32) * 100.0
                    );
                    println!("     {}: {:.3}s", "Duration".white(), r.duration.as_secs_f64());
                    println!("     {}: {}", "Failures".red().bold(), r.errors.len());
                    
                    let display_count = if verbose { r.errors.len() } else { 5.min(r.errors.len()) };
                    for err in r.errors.iter().take(display_count) {
                        println!("       {}", err.white());
                    }
                    if r.errors.len() > display_count {
                        println!("       {} more failures", format!("... and {}", r.errors.len() - display_count).white());
                    }
                }
            }
            super::types::ValidationResult::Regression(r) => {
                println!("\n  {} {}", "❌".red(), r.file.display().to_string().bold());
                println!("     {}: {:?}", "Status".white(), r.status);
                println!("     {}: {:.3}s", "Duration".white(), r.duration.as_secs_f64().to_string().white());
                println!("     {}: {}", "Fixed".white(), 
                    if r.fixed { "Yes".green() } else { "No".red() });
                
                if let Some(ref err) = r.error {
                    println!("     {}:", "Error".red().bold());
                    for line in err.lines() {
                        println!("       {}", line.white());
                    }
                } else if verbose {
                    println!("     {}: Regression test failed", "Note".yellow());
                }
            }
        }
    }
}

