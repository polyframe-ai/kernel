// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Polyframe Evaluation Harness CLI
//! Comprehensive test suite for OpenSCAD compatibility

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use polyframe::evaluation::{
    load_corpus, run_and_compare, run_model_task, Comparison, EvaluationReport, Reporter,
    RegressionSuite, Fuzzer, FuzzerConfig,
};
use rayon::prelude::*;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "polyframe-eval")]
#[command(about = "Polyframe Kernel Compatibility Test Harness", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Run evaluation on dataset or file
    Eval {
        /// Dataset directory path
        #[arg(long)]
        dataset: Option<String>,

        /// Single SCAD file to evaluate
        #[arg(long)]
        file: Option<String>,

        /// Output directory for results
        #[arg(short, long, default_value = "tests/evaluation/outputs")]
        out: String,
    },

    /// Run fuzz testing
    Fuzz {
        /// Number of fuzz tests to generate
        #[arg(short, long, default_value = "500")]
        count: usize,

        /// Output directory for fuzz results
        #[arg(short, long, default_value = "tests/evaluation/outputs/fuzz")]
        out: String,
    },

    /// Generate markdown report from JSON
    GenerateReport {
        /// Input JSON report file
        #[arg(short, long, default_value = "tests/evaluation/outputs/latest.json")]
        input: String,

        /// Output markdown report file
        #[arg(short, long, default_value = "tests/evaluation/report.md")]
        output: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Eval { dataset, file, out } => {
            eval_command(dataset.as_deref(), file.as_deref(), out, cli.verbose)?;
        }
        Commands::Fuzz { count, out } => {
            fuzz_command(*count, out, cli.verbose)?;
        }
        Commands::GenerateReport { input, output } => {
            generate_report_command(input, output)?;
        }
    }

    Ok(())
}

fn eval_command(
    dataset: Option<&str>,
    file: Option<&str>,
    out: &str,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("{}", "Starting evaluation harness...".bold());
    }

    let output_dir = PathBuf::from(out);
    std::fs::create_dir_all(&output_dir)?;

    let mut report = EvaluationReport::new();
    let regression_suite = RegressionSuite::new("tests/evaluation/datasets/regressions");
    regression_suite.initialize()?;

    if let Some(file_path) = file {
        // Single file evaluation
        let path = PathBuf::from(file_path);
        if !path.exists() {
            eprintln!("{} File not found: {}", "Error:".red(), file_path);
            std::process::exit(1);
        }

        if verbose {
            println!("Evaluating single file: {}", file_path);
        }

        match run_and_compare(&path) {
            Ok(result) => {
                report.add_result(result);
            }
            Err(e) => {
                report.add_error(
                    path.display().to_string(),
                    format!("Execution error: {}", e),
                );
                // Add to regression suite
                let _ = regression_suite.add_regression(
                    &path,
                    Some(&e.to_string()),
                    "Should render successfully",
                    None,
                    None,
                );
            }
        }
    } else if let Some(dataset_path) = dataset {
        // Dataset evaluation
        if verbose {
            println!("Loading dataset from: {}", dataset_path);
        }

        let corpus = load_corpus(dataset_path)?;
        let total = corpus.len();

        if verbose {
            println!("Found {} files to evaluate", total);
        }

        // Create progress bar
        let progress = if verbose {
            let pb = ProgressBar::new(total as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template(
                        "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
                    )
                    .unwrap()
                    .progress_chars("#>-"),
            );
            Some(pb)
        } else {
            None
        };

        // Process files in parallel with Rayon
        let results: Vec<_> = corpus
            .par_iter()
            .map(|entry| {
                if let Some(ref pb) = progress {
                    pb.set_message(format!("Evaluating {}", entry.path.display()));
                }

                let result = match run_and_compare(&entry.path) {
                    Ok(r) => Ok(r),
                    Err(e) => {
                        // Add to regression suite
                        let _ = regression_suite.add_regression(
                            &entry.path,
                            Some(&e.to_string()),
                            "Should render successfully",
                            None,
                            None,
                        );
                        Err(e)
                    }
                };

                if let Some(ref pb) = progress {
                    pb.inc(1);
                }

                result
            })
            .collect();

        // Process results
        for result in results {
            match result {
                Ok(eval_result) => {
                    report.add_result(eval_result);
                }
                Err(e) => {
                    // Error already added to regression suite
                    if verbose {
                        eprintln!("{} Error: {}", "Warning:".yellow(), e);
                    }
                }
            }
        }

        if let Some(pb) = progress {
            pb.finish_with_message("Evaluation complete");
        }
    } else {
        eprintln!("{} Either --dataset or --file must be specified", "Error:".red());
        std::process::exit(1);
    }

    // Write reports
    let json_path = output_dir.join("latest.json");
    Reporter::write_json(&report, &json_path)?;
    
    let md_path = output_dir.join("report.md");
    Reporter::write_markdown(&report, &md_path)?;

    // Print summary
    if verbose {
        print_summary(&report);
    } else {
        println!(
            "{} {} ({:.1}%)",
            "Passed:".green(),
            report.passed,
            report.pass_rate()
        );
        println!("{} {}", "Failed:".red(), report.failed);
        println!("{} {}", "Errors:".yellow(), report.errors);
    }

    if report.failed > 0 || report.errors > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn fuzz_command(count: usize, out: &str, verbose: bool) -> Result<()> {
    if verbose {
        println!("{}", "Starting fuzz testing...".bold());
        println!("Generating {} random SCAD files", count);
    }

    let output_dir = PathBuf::from(out);
    std::fs::create_dir_all(&output_dir)?;

    let config = FuzzerConfig {
        count,
        max_depth: 5,
        max_primitives: 10,
    };
    let mut fuzzer = Fuzzer::new(config);

    let results = fuzzer.run();

    if verbose {
        println!("Generated {} fuzz test cases", results.len());
        println!("Writing to: {}", output_dir.display());
    }

    // Write fuzz test files
    for (name, scad_code) in &results {
        let file_path = output_dir.join(format!("{}.scad", name));
        std::fs::write(&file_path, scad_code)?;
    }

    // Test parse parity
    if verbose {
        println!("Testing parse parity...");
    }

    let mut parse_failures = 0;
    for (name, scad_code) in &results {
        match polyframe::parse_scad(scad_code) {
            Ok(_) => {
                if verbose {
                    println!("  ✓ {}: Parsed successfully", name);
                }
            }
            Err(e) => {
                parse_failures += 1;
                if verbose {
                    println!("  ✗ {}: Parse failed - {}", name, e);
                }
            }
        }
    }

    println!(
        "\n{} Fuzz test complete: {} generated, {} parse failures",
        "Summary:".bold(),
        results.len(),
        parse_failures
    );

    Ok(())
}

fn generate_report_command(input: &str, output: &str) -> Result<()> {
    let input_path = PathBuf::from(input);
    let output_path = PathBuf::from(output);

    if !input_path.exists() {
        eprintln!("{} Input file not found: {}", "Error:".red(), input);
        std::process::exit(1);
    }

    Reporter::generate_report(&input_path, &output_path)?;

    println!("{} Generated report: {}", "Success:".green(), output);

    Ok(())
}

fn print_summary(report: &EvaluationReport) {
    println!("\n{}", "═".repeat(80).bright_black());
    println!("{}", "Evaluation Summary".bold());
    println!("{}", "═".repeat(80).bright_black());
    println!(
        "  {} {}",
        "Total Models:".bright_black(),
        report.total_models.to_string().cyan()
    );
    println!(
        "  {} {} ({:.1}%)",
        "Passed:".bright_black(),
        report.passed.to_string().green(),
        report.pass_rate()
    );
    println!(
        "  {} {}",
        "Failed:".bright_black(),
        if report.failed > 0 {
            report.failed.to_string().red()
        } else {
            report.failed.to_string().green()
        }
    );
    println!(
        "  {} {}",
        "Errors:".bright_black(),
        if report.errors > 0 {
            report.errors.to_string().red()
        } else {
            report.errors.to_string().green()
        }
    );
    println!(
        "  {} {} ({:.1}% success)",
        "Success Rate:".bright_black(),
        (report.total_models - report.errors).to_string().cyan(),
        report.success_rate()
    );
    println!(
        "  {} {}",
        "Avg Speedup:".bright_black(),
        format!("{:.1}×", report.avg_speedup).yellow()
    );

    if report.errors > 0 {
        println!("\n  {}", "Errors:".red().bold());
        for err in &report.error_details {
            println!("    {} {}", "❌".red(), err.model);
            println!("       {}", err.error.bright_black());
        }
    }

    println!(
        "\n  {} {}",
        "JSON Report:".bright_black(),
        "tests/evaluation/outputs/latest.json".cyan()
    );
    println!(
        "  {} {}",
        "Markdown Report:".bright_black(),
        "tests/evaluation/outputs/report.md".cyan()
    );
    println!("{}", "═".repeat(80).bright_black());
}

