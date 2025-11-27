// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Unified validation CLI for Polyframe Kernel

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use polyframe::validation::{TestSuite, ValidationConfig, ValidationCoordinator, ValidationReporter};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "polyframe-validate")]
#[command(about = "Unified validation system for OpenSCAD compatibility", long_about = None)]
struct Cli {
    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Output directory for reports
    #[arg(short, long, global = true, default_value = "tests/evaluation/outputs")]
    output: String,

    /// Generate preview renders and visual diffs for comparison tests
    #[arg(long, global = true)]
    preview_images: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all validation suites
    All {
        /// Test suites to run (comma-separated: unit,integration,evaluation,comparison,fuzz,regression)
        #[arg(long)]
        suites: Option<String>,
    },

    /// Run specific test suite
    Suite {
        /// Suite to run (unit, integration, evaluation, comparison, fuzz, regression)
        suite: String,

        /// Filter test names
        #[arg(short, long)]
        filter: Option<String>,

        /// Specific file to test
        #[arg(long)]
        file: Option<String>,
    },

    /// Generate report from existing results
    Report {
        /// Input JSON report file
        #[arg(short, long)]
        input: String,

        /// Output format (json, markdown, terminal)
        #[arg(short, long, default_value = "markdown")]
        format: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::All { suites } => {
            run_all_validation(
                suites.as_deref(),
                &cli.output,
                cli.verbose,
                cli.preview_images,
            )?;
        }
        Commands::Suite { suite, filter, file } => {
            run_suite(
                suite,
                filter.as_deref(),
                file.as_deref(),
                &cli.output,
                cli.verbose,
                cli.preview_images,
            )?;
        }
        Commands::Report { input, format } => {
            generate_report(input, format, &cli.output)?;
        }
    }

    Ok(())
}

fn run_all_validation(
    suites: Option<&str>,
    output_dir: &str,
    verbose: bool,
    preview_images: bool,
) -> Result<()> {
    let mut config = ValidationConfig::load().unwrap_or_default();
    config.verbose = verbose;
    config.output_dir = PathBuf::from(output_dir);
    if preview_images {
        config.generate_visual_diffs = true;
    }

    // Parse suites if provided
    if let Some(suites_str) = suites {
        let suite_names: Vec<&str> = suites_str.split(',').map(|s| s.trim()).collect();
        config.suites = suite_names
            .iter()
            .filter_map(|s| TestSuite::from_str(s))
            .collect();
    } else {
        // Default: run all suites
        config.suites = vec![
            TestSuite::Unit,
            TestSuite::Integration,
            TestSuite::Evaluation,
            TestSuite::Comparison,
        ];
    }

    if verbose {
        println!("{}", "Starting unified validation...".bold());
        println!("  Suites: {:?}", config.suites.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        println!("  Output: {}", config.output_dir.display());
    }

    let coordinator = ValidationCoordinator::new(config);
    let suite_results = coordinator.run_all()?;

    // Build validation report
    let mut report = polyframe::validation::ValidationReport::new();
    for suite_result in suite_results {
        report.add_suite_result(suite_result);
    }

    // Write reports
    let output_path = PathBuf::from(output_dir);
    std::fs::create_dir_all(&output_path)?;

    let json_path = output_path.join("validation_report.json");
    ValidationReporter::write_json(&report, &json_path)?;

    let md_path = output_path.join("validation_report.md");
    ValidationReporter::write_markdown(&report, &md_path)?;

    // Print summary
    ValidationReporter::print_summary_with_verbose(&report, verbose);

    if report.has_failures() {
        std::process::exit(1);
    }

    Ok(())
}

fn run_suite(
    suite_name: &str,
    filter: Option<&str>,
    file: Option<&str>,
    output_dir: &str,
    verbose: bool,
    preview_images: bool,
) -> Result<()> {
    let suite = TestSuite::from_str(suite_name)
        .ok_or_else(|| anyhow::anyhow!("Unknown test suite: {}", suite_name))?;

    let mut config = ValidationConfig::load().unwrap_or_default();
    config.verbose = verbose;
    config.output_dir = PathBuf::from(output_dir);
    config.suites = vec![suite];
    if preview_images {
        config.generate_visual_diffs = true;
    }

    if let Some(filter_str) = filter {
        config.filters.push(filter_str.to_string());
    }

    if let Some(file_path) = file {
        config.file_patterns.push(file_path.to_string());
    }

    if verbose {
        println!("{}", format!("Running {} suite...", suite.as_str()).bold());
    }

    let coordinator = ValidationCoordinator::new(config);
    let suite_result = coordinator.run_suite(suite)?;

    // Build validation report
    let mut report = polyframe::validation::ValidationReport::new();
    report.add_suite_result(suite_result);

    // Write reports
    let output_path = PathBuf::from(output_dir);
    std::fs::create_dir_all(&output_path)?;

    let json_path = output_path.join(format!("{}_report.json", suite.as_str()));
    ValidationReporter::write_json(&report, &json_path)?;

    let md_path = output_path.join(format!("{}_report.md", suite.as_str()));
    ValidationReporter::write_markdown(&report, &md_path)?;

    // Print summary
    ValidationReporter::print_summary_with_verbose(&report, verbose);

    if report.has_failures() {
        std::process::exit(1);
    }

    Ok(())
}

fn generate_report(input: &str, format: &str, output_dir: &str) -> Result<()> {
    let json_content = std::fs::read_to_string(input)?;
    let report: polyframe::validation::ValidationReport = serde_json::from_str(&json_content)?;

    let output_path = PathBuf::from(output_dir);
    std::fs::create_dir_all(&output_path)?;

    match format.to_lowercase().as_str() {
        "json" => {
            let json_path = output_path.join("report.json");
            ValidationReporter::write_json(&report, &json_path)?;
            println!("{} Generated JSON report: {}", "Success:".green(), json_path.display());
        }
        "markdown" | "md" => {
            let md_path = output_path.join("report.md");
            ValidationReporter::write_markdown(&report, &md_path)?;
            println!("{} Generated Markdown report: {}", "Success:".green(), md_path.display());
        }
        "terminal" | "term" => {
            ValidationReporter::print_summary(&report);
        }
        _ => {
            return Err(anyhow::anyhow!("Unknown format: {}. Use json, markdown, or terminal", format));
        }
    }

    Ok(())
}

