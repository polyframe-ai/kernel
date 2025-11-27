// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Polyframe Kernel CLI

use anyhow::Result;
use clap::{Parser, Subcommand};
use polyframe::io;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "polyframe-kernel")]
#[command(about = "Polyframe CAD Kernel - OpenSCAD-compatible geometry engine", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Input SCAD file
    #[arg(value_name = "FILE")]
    input: Option<String>,

    /// Output file
    #[arg(short, long, value_name = "FILE")]
    output: Option<String>,

    /// Output format (stl, 3mf, gltf)
    #[arg(short, long, default_value = "stl")]
    format: String,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Render a SCAD file to mesh
    Render {
        /// Input SCAD file
        input: String,

        /// Output file
        #[arg(short, long)]
        output: String,

        /// Output format
        #[arg(short, long, default_value = "stl")]
        format: String,

        /// Lazy rendering mode (defer rendering until explicitly requested)
        #[arg(long)]
        lazy: bool,

        /// Use parallel evaluation
        #[arg(long)]
        parallel: bool,

        /// Use incremental evaluation
        #[arg(long)]
        incremental: bool,
    },

    /// Compare Polyframe output with OpenSCAD
    Compare {
        /// Input SCAD file(s)
        #[arg(required = true)]
        inputs: Vec<String>,

        /// Comparison tolerance
        #[arg(short, long, default_value = "0.00001")]
        tolerance: f32,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,

        /// Directory to store preview renders and visual diffs
        #[arg(long)]
        preview_dir: Option<String>,
    },

    /// Run evaluation harness on dataset
    Eval {
        /// Dataset directory or file(s)
        #[arg(required = true)]
        dataset: Vec<String>,

        /// Output directory for results
        #[arg(short, long, default_value = "tests/evaluation/results")]
        out: String,
    },

    /// Parse SCAD file and output AST as JSON
    Parse {
        /// Input SCAD file
        input: String,

        /// Output JSON file
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Analyze geometry and print statistics
    Analyze {
        /// Input file (STL, 3MF, or SCAD)
        input: String,

        /// Output format (json, text)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Show version information
    Version,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Render {
            input,
            output,
            format,
            lazy,
            parallel,
            incremental,
        }) => {
            render_command(
                input,
                output,
                format,
                *lazy,
                *parallel,
                *incremental,
                cli.verbose,
            )?;
        }
        Some(Commands::Compare {
            inputs,
            tolerance,
            verbose,
            preview_dir,
        }) => {
            compare_command(inputs, *tolerance, *verbose, preview_dir.as_deref())?;
        }
        Some(Commands::Eval { dataset, out }) => {
            eval_command(dataset, out, cli.verbose)?;
        }
        Some(Commands::Parse { input, output }) => {
            parse_command(input, output.as_deref(), cli.verbose)?;
        }
        Some(Commands::Analyze { input, format }) => {
            analyze_command(input, format, cli.verbose)?;
        }
        Some(Commands::Version) => {
            println!("Polyframe Kernel v{}", env!("CARGO_PKG_VERSION"));
        }
        None => {
            // Default behavior: render input to output
            if let (Some(input), Some(output)) = (&cli.input, &cli.output) {
                render_command(input, output, &cli.format, false, false, false, cli.verbose)?;
            } else {
                eprintln!("Error: Input and output files required");
                eprintln!("Usage: polyframe-kernel <INPUT> --output <OUTPUT>");
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

fn render_command(
    input: &str,
    output: &str,
    format: &str,
    lazy: bool,
    parallel: bool,
    incremental: bool,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("Rendering: {}", input);
        if lazy {
            println!("  Mode: Lazy (deferred rendering)");
        }
        if parallel {
            println!("  Mode: Parallel evaluation");
        }
        if incremental {
            println!("  Mode: Incremental evaluation");
        }
    }

    // Check if input file exists
    if !Path::new(input).exists() {
        eprintln!("Error: Input file not found: {}", input);
        std::process::exit(1);
    }

    // Parse AST
    let start = std::time::Instant::now();
    let ast = io::import_scad_file(input)?;
    let parse_time = start.elapsed();

    if verbose {
        println!("Parsed in {:.2?}", parse_time);
    }

    // Choose evaluator based on flags
    let render_start = std::time::Instant::now();
    let mesh = if incremental {
        // Use incremental evaluator
        use polyframe::IncrementalEvaluator;
        let evaluator = IncrementalEvaluator::from_ast(&ast);
        let result = evaluator.evaluate(&ast)?;

        if verbose {
            let stats = evaluator.cache_stats();
            println!(
                "Cache stats: {}/{} nodes cached ({:.1}% hit rate)",
                stats.cached_nodes,
                stats.total_nodes,
                stats.hit_rate()
            );
        }

        result
    } else if parallel {
        // Use parallel evaluator
        use polyframe::ast::ParallelEvaluator;
        ParallelEvaluator::evaluate(&ast)?
    } else {
        // Use standard evaluator
        let evaluator = polyframe::ast::Evaluator::new();
        evaluator.evaluate(&ast)?
    };

    let render_time = render_start.elapsed();

    if verbose {
        println!("Rendered in {:.2?}", render_time);
        println!("Vertices: {}", mesh.vertex_count());
        println!("Triangles: {}", mesh.triangle_count());
    }

    // Lazy mode: skip export if flag is set
    if lazy {
        if verbose {
            println!("Lazy mode: Skipping export (mesh kept in memory)");
        }
        return Ok(());
    }

    // Export based on format
    let export_start = std::time::Instant::now();
    match format.to_lowercase().as_str() {
        "stl" => io::export_stl(&mesh, output)?,
        "3mf" => io::export_3mf(&mesh, output)?,
        "gltf" | "glb" => io::export_gltf(&mesh, output)?,
        "step" | "stp" => io::export_step(&mesh, output)?,
        _ => {
            eprintln!("Error: Unsupported format: {}", format);
            eprintln!("Supported formats: stl, 3mf, gltf, glb, step");
            std::process::exit(1);
        }
    }
    let export_time = export_start.elapsed();

    if verbose {
        println!("Exported in {:.2?}", export_time);
        println!("Output: {}", output);
    } else {
        println!("Successfully rendered {} -> {}", input, output);
    }

    Ok(())
}

fn compare_command(
    inputs: &[String],
    tolerance: f32,
    verbose: bool,
    preview_dir: Option<&str>,
) -> Result<()> {
    use polyframe::cli::{batch_compare, compare_with_openscad, PreviewConfig, Reporter};

    let preview_base = preview_dir.map(PathBuf::from);

    if inputs.len() == 1 {
        // Single file comparison
        let input = Path::new(&inputs[0]);

        if !input.exists() {
            Reporter::report_error(&format!("Input file not found: {}", inputs[0]));
            std::process::exit(1);
        }

        let preview = preview_base
            .as_ref()
            .map(|root| PreviewConfig::for_input(root, input));

        let result = compare_with_openscad(input, tolerance, verbose, preview)?;

        if !result.passed {
            std::process::exit(1);
        }
    } else {
        // Batch comparison
        let paths: Vec<&Path> = inputs.iter().map(|s| Path::new(s.as_str())).collect();
        let preview_root = preview_base.as_deref();
        let results = batch_compare(&paths, tolerance, verbose, preview_root)?;

        let failed = results.iter().filter(|(_, r)| !r.passed).count();
        if failed > 0 {
            std::process::exit(1);
        }
    }

    Ok(())
}

fn eval_command(dataset: &[String], out: &str, verbose: bool) -> Result<()> {
    use colored::Colorize;
    use indicatif::{ProgressBar, ProgressStyle};
    use polyframe::evaluation;

    if verbose {
        println!("{}", "Starting evaluation harness...".bold());
    }

    // Load tasks from all dataset sources
    let mut all_tasks = Vec::new();

    for dataset_path in dataset {
        let path = std::path::PathBuf::from(dataset_path);

        if !path.exists() {
            eprintln!("{} Path not found: {}", "Error:".red(), dataset_path);
            continue;
        }

        // Detect source type (JSON or folder)
        let source = evaluation::detect_source(&path);

        if verbose {
            match &source {
                evaluation::DatasetSource::JsonFile(_) => {
                    println!(
                        "{} Loading JSON exercises from {}",
                        "ℹ".bright_blue(),
                        dataset_path
                    );
                }
                evaluation::DatasetSource::Folder(_) => {
                    println!(
                        "{} Discovering .scad files in {}",
                        "ℹ".bright_blue(),
                        dataset_path
                    );
                }
            }
        }

        match evaluation::load_dataset(source) {
            Ok(tasks) => {
                if verbose {
                    println!("  Found {} tasks", tasks.len());
                }
                all_tasks.extend(tasks);
            }
            Err(e) => {
                eprintln!(
                    "{} Failed to load dataset {}: {}",
                    "Error:".red(),
                    dataset_path,
                    e
                );
            }
        }
    }

    if all_tasks.is_empty() {
        eprintln!("{}", "No tasks found in dataset(s)".red());
        std::process::exit(1);
    }

    if verbose {
        println!("Total tasks to evaluate: {}", all_tasks.len());
    }

    // Create progress bar
    let progress = if verbose {
        let pb = ProgressBar::new(all_tasks.len() as u64);
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

    // Run evaluation
    let output_dir = std::path::PathBuf::from(out);
    let mut report = evaluation::reporter::EvaluationReport::new();

    for task in &all_tasks {
        if let Some(ref pb) = progress {
            pb.set_message(format!("Evaluating {}", task.name()));
        }

        match evaluation::run_model_task(task) {
            Ok(result) => {
                report.add_result(result);
            }
            Err(e) => {
                if verbose {
                    eprintln!("{} {}: {}", "Error".red(), task.name(), e);
                }
                // Add a failed result to the report
                report.add_error(task.name(), e.to_string());
            }
        }

        if let Some(ref pb) = progress {
            pb.inc(1);
        }
    }

    if let Some(pb) = progress {
        pb.finish_with_message("Evaluation complete");
    }

    // Write reports
    std::fs::create_dir_all(&output_dir)?;
    evaluation::Reporter::write_json(&report, &output_dir.join("latest.json"))?;
    evaluation::Reporter::write_markdown(&report, &output_dir.join("report.md"))?;

    // Print summary
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

    if report.errors > 0 && verbose {
        println!("\n  {}", "Errors:".red().bold());
        for err in &report.error_details {
            println!("    {} {}", "❌".red(), err.model);
            println!("       {}", err.error.bright_black());
        }
    }

    println!(
        "\n  {} {}",
        "JSON Report:".bright_black(),
        output_dir.join("latest.json").display().to_string().cyan()
    );
    println!(
        "  {} {}",
        "Markdown Report:".bright_black(),
        output_dir.join("report.md").display().to_string().cyan()
    );
    println!("{}", "═".repeat(80).bright_black());

    if report.failed > 0 || report.errors > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn parse_command(input: &str, output: Option<&str>, verbose: bool) -> Result<()> {
    if verbose {
        println!("Parsing: {}", input);
    }

    // Check if input file exists
    if !Path::new(input).exists() {
        eprintln!("Error: Input file not found: {}", input);
        std::process::exit(1);
    }

    // Parse the file
    let ast = io::import_scad_file(input)?;
    let json = serde_json::to_string_pretty(&ast)?;

    // Output to file or stdout
    if let Some(output_path) = output {
        std::fs::write(output_path, json)?;
        if verbose {
            println!("AST written to: {}", output_path);
        }
    } else {
        println!("{}", json);
    }

    Ok(())
}

fn analyze_command(input: &str, format: &str, verbose: bool) -> Result<()> {
    use polyframe::geometry::analyze;

    if verbose {
        println!("Analyzing: {}", input);
    }

    // Check if input file exists
    if !Path::new(input).exists() {
        eprintln!("Error: Input file not found: {}", input);
        std::process::exit(1);
    }

    // Render the mesh first
    let mesh = if input.ends_with(".scad") {
        polyframe::render_file(input)?
    } else if input.ends_with(".stl") {
        // For STL files, we'd need an STL importer
        // For now, try to render as SCAD
        eprintln!("Note: STL import not yet implemented, trying as SCAD");
        polyframe::render_file(input)?
    } else {
        polyframe::render_file(input)?
    };

    // Analyze the mesh
    let stats = analyze(&mesh);

    // Output based on format
    match format.to_lowercase().as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&stats)?;
            println!("{}", json);
        }
        "text" => {
            stats.print();
        }
        _ => {
            stats.print();
        }
    }

    Ok(())
}
