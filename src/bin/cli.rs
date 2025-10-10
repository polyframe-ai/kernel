// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Polyframe Kernel CLI

use clap::{Parser, Subcommand};
use polyframe::{render_file, io};
use anyhow::Result;
use std::path::Path;

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
    },
    
    /// Compare Polyframe output with OpenSCAD
    Compare {
        /// Input SCAD file(s)
        #[arg(required = true)]
        inputs: Vec<String>,
        
        /// Comparison tolerance
        #[arg(short, long, default_value = "0.00001")]
        tolerance: f32,
    },
    
    /// Run evaluation harness on dataset
    Eval {
        /// Dataset directory or file(s)
        #[arg(required = true)]
        dataset: Vec<String>,
        
        /// Output directory for results
        #[arg(short, long, default_value = "evaluation/results")]
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
    
    /// Show version information
    Version,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Render { input, output, format }) => {
            render_command(input, output, format, cli.verbose)?;
        }
        Some(Commands::Compare { inputs, tolerance }) => {
            compare_command(inputs, *tolerance, cli.verbose)?;
        }
        Some(Commands::Eval { dataset, out }) => {
            eval_command(dataset, out, cli.verbose)?;
        }
        Some(Commands::Parse { input, output }) => {
            parse_command(input, output.as_deref(), cli.verbose)?;
        }
        Some(Commands::Version) => {
            println!("Polyframe Kernel v{}", env!("CARGO_PKG_VERSION"));
        }
        None => {
            // Default behavior: render input to output
            if let (Some(input), Some(output)) = (&cli.input, &cli.output) {
                render_command(input, output, &cli.format, cli.verbose)?;
            } else {
                eprintln!("Error: Input and output files required");
                eprintln!("Usage: polyframe-kernel <INPUT> --output <OUTPUT>");
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

fn render_command(input: &str, output: &str, format: &str, verbose: bool) -> Result<()> {
    if verbose {
        println!("Rendering: {}", input);
    }

    // Check if input file exists
    if !Path::new(input).exists() {
        eprintln!("Error: Input file not found: {}", input);
        std::process::exit(1);
    }

    // Render the mesh
    let start = std::time::Instant::now();
    let mesh = render_file(input)?;
    let render_time = start.elapsed();

    if verbose {
        println!("Rendered in {:.2?}", render_time);
        println!("Vertices: {}", mesh.vertex_count());
        println!("Triangles: {}", mesh.triangle_count());
    }

    // Export based on format
    let export_start = std::time::Instant::now();
    match format.to_lowercase().as_str() {
        "stl" => io::export_stl(&mesh, output)?,
        "3mf" => io::export_3mf(&mesh, output)?,
        "gltf" | "glb" => io::export_gltf(&mesh, output)?,
        _ => {
            eprintln!("Error: Unsupported format: {}", format);
            eprintln!("Supported formats: stl, 3mf, gltf");
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

fn compare_command(inputs: &[String], tolerance: f32, verbose: bool) -> Result<()> {
    use polyframe::cli::{compare_with_openscad, batch_compare, Reporter};
    
    if inputs.len() == 1 {
        // Single file comparison
        let input = Path::new(&inputs[0]);
        
        if !input.exists() {
            Reporter::report_error(&format!("Input file not found: {}", inputs[0]));
            std::process::exit(1);
        }
        
        let result = compare_with_openscad(input, tolerance, verbose)?;
        
        if !result.passed {
            std::process::exit(1);
        }
    } else {
        // Batch comparison
        let paths: Vec<&Path> = inputs.iter()
            .map(|s| Path::new(s.as_str()))
            .collect();
        
        let results = batch_compare(&paths, tolerance, verbose)?;
        
        let failed = results.iter().filter(|(_, r)| !r.passed).count();
        if failed > 0 {
            std::process::exit(1);
        }
    }
    
    Ok(())
}

fn eval_command(dataset: &[String], out: &str, verbose: bool) -> Result<()> {
    use polyframe::evaluation;
    use colored::Colorize;
    use indicatif::{ProgressBar, ProgressStyle};
    
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
                    println!("{} Loading JSON exercises from {}", "ℹ".bright_blue(), dataset_path);
                }
                evaluation::DatasetSource::Folder(_) => {
                    println!("{} Discovering .scad files in {}", "ℹ".bright_blue(), dataset_path);
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
                eprintln!("{} Failed to load dataset {}: {}", "Error:".red(), dataset_path, e);
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
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-"));
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
    println!("  {} {}", "Total Models:".bright_black(), report.total_models.to_string().cyan());
    println!("  {} {} ({:.1}%)", 
        "Passed:".bright_black(), 
        report.passed.to_string().green(),
        report.pass_rate()
    );
    println!("  {} {}", 
        "Failed:".bright_black(), 
        if report.failed > 0 { report.failed.to_string().red() } else { report.failed.to_string().green() }
    );
    println!("  {} {}", 
        "Errors:".bright_black(), 
        if report.errors > 0 { report.errors.to_string().red() } else { report.errors.to_string().green() }
    );
    println!("  {} {} ({:.1}% success)", 
        "Success Rate:".bright_black(),
        (report.total_models - report.errors).to_string().cyan(),
        report.success_rate()
    );
    println!("  {} {}", "Avg Speedup:".bright_black(), format!("{:.1}×", report.avg_speedup).yellow());
    
    if report.errors > 0 && verbose {
        println!("\n  {}", "Errors:".red().bold());
        for err in &report.error_details {
            println!("    {} {}", "❌".red(), err.model);
            println!("       {}", err.error.bright_black());
        }
    }
    
    println!("\n  {} {}", "JSON Report:".bright_black(), output_dir.join("latest.json").display().to_string().cyan());
    println!("  {} {}", "Markdown Report:".bright_black(), output_dir.join("report.md").display().to_string().cyan());
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

