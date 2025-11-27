// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Validation coordinator - orchestrates all test execution

use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use super::config::ValidationConfig;
use super::discovery::TestDiscovery;
use super::types::{
    ComparisonTestResult, EvaluationTestResult, FuzzTestResult, IntegrationTestResult,
    RegressionTestResult, SuiteResult, TestStatus, TestSuite, UnitTestResult, ValidationResult,
};

/// Validation coordinator
pub struct ValidationCoordinator {
    config: ValidationConfig,
}

impl ValidationCoordinator {
    /// Create a new coordinator
    pub fn new(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Run all configured test suites
    pub fn run_all(&self) -> Result<Vec<SuiteResult>> {
        let mut suite_results = Vec::new();

        for suite in &self.config.suites {
            if self.config.should_run_suite(*suite) {
                let result = self.run_suite(*suite)?;
                suite_results.push(result);
            }
        }

        Ok(suite_results)
    }

    /// Run a specific test suite
    pub fn run_suite(&self, suite: TestSuite) -> Result<SuiteResult> {
        let start = Instant::now();
        let mut suite_result = SuiteResult::new(suite);

        match suite {
            TestSuite::Unit => {
                suite_result = self.run_unit_tests()?;
            }
            TestSuite::Integration => {
                suite_result = self.run_integration_tests()?;
            }
            TestSuite::Evaluation => {
                suite_result = self.run_evaluation_suite()?;
            }
            TestSuite::Comparison => {
                suite_result = self.run_comparison_tests()?;
            }
            TestSuite::Fuzz => {
                suite_result = self.run_fuzz_tests()?;
            }
            TestSuite::Regression => {
                suite_result = self.run_regression_tests()?;
            }
        }

        suite_result.duration = start.elapsed();
        Ok(suite_result)
    }

    /// Run unit tests via cargo test
    fn run_unit_tests(&self) -> Result<SuiteResult> {
        let mut suite_result = SuiteResult::new(TestSuite::Unit);

        if self.config.verbose {
            println!("{}", "Running unit tests...".bold().cyan());
        }

        // Run cargo test and capture output
        let output = Command::new("cargo")
            .args(&["test", "--lib", "--", "--test-threads=1"])
            .output()
            .context("Failed to run cargo test")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Parse test results from cargo output
        let lines: Vec<&str> = stdout.lines().collect();
        let mut current_test: Option<String> = None;
        let mut test_duration = Duration::ZERO;

        for line in lines {
            if line.contains("test ") && line.contains(" ... ") {
                // Extract test name
                if let Some(start) = line.find("test ") {
                    if let Some(end) = line[start + 5..].find(" ... ") {
                        let test_name = line[start + 5..start + 5 + end].trim().to_string();
                        current_test = Some(test_name);
                    }
                }
            } else if line.contains("test result:") {
                // Parse summary
                let passed = line.matches("passed").count();
                let failed = line.matches("failed").count();

                suite_result.passed = passed;
                suite_result.failed = failed;
                suite_result.total = passed + failed;
            } else if let Some(ref test_name) = current_test {
                if line.contains("ok") {
                    suite_result.add_result(ValidationResult::Unit(UnitTestResult {
                        name: test_name.clone(),
                        status: TestStatus::Passed,
                        duration: test_duration,
                        error: None,
                    }));
                    current_test = None;
                } else if line.contains("FAILED") || line.contains("FAIL") {
                    suite_result.add_result(ValidationResult::Unit(UnitTestResult {
                        name: test_name.clone(),
                        status: TestStatus::Failed,
                        duration: test_duration,
                        error: Some(line.to_string()),
                    }));
                    current_test = None;
                }
            }
        }

        // If parsing failed, create a summary result
        if suite_result.total == 0 {
            let status = if output.status.success() {
                TestStatus::Passed
            } else {
                TestStatus::Failed
            };

            suite_result.add_result(ValidationResult::Unit(UnitTestResult {
                name: "cargo test".to_string(),
                status,
                duration: Duration::ZERO,
                error: if !output.status.success() {
                    Some(stderr.to_string())
                } else {
                    None
                },
            }));
        }

        Ok(suite_result)
    }

    /// Run integration tests
    fn run_integration_tests(&self) -> Result<SuiteResult> {
        let mut suite_result = SuiteResult::new(TestSuite::Integration);

        if self.config.verbose {
            println!("{}", "Running integration tests...".bold().cyan());
        }

        // Run cargo test --test '*'
        let output = Command::new("cargo")
            .args(&["test", "--test", "*"])
            .output()
            .context("Failed to run integration tests")?;

        // Similar parsing as unit tests
        let stdout = String::from_utf8_lossy(&output.stdout);
        let status = if output.status.success() {
            TestStatus::Passed
        } else {
            TestStatus::Failed
        };

        suite_result.add_result(ValidationResult::Integration(IntegrationTestResult {
            name: "integration tests".to_string(),
            status,
            duration: Duration::ZERO,
            error: if !output.status.success() {
                Some(stdout.to_string())
            } else {
                None
            },
            file: None,
        }));

        Ok(suite_result)
    }

    /// Run evaluation suite
    fn run_evaluation_suite(&self) -> Result<SuiteResult> {
        let mut suite_result = SuiteResult::new(TestSuite::Evaluation);

        if self.config.verbose {
            println!("{}", "Running evaluation suite...".bold().cyan());
        }

        // Discover evaluation tests
        let tests = TestDiscovery::discover_all()?;
        let eval_tests: Vec<_> = tests
            .iter()
            .filter(|t| t.suite == TestSuite::Evaluation)
            .filter(|t| self.config.matches_file_pattern(&t.path))
            .collect();

        if eval_tests.is_empty() {
            // Try loading JSON exercises directly
            let json_files = vec![
                PathBuf::from("tests/fixtures/polyframe_exercises_001_040.json"),
                PathBuf::from("tests/fixtures/polyframe_exercises_041_100.json"),
                PathBuf::from("tests/fixtures/polyframe_exercises_101_150.json"),
            ];

            for json_file in json_files {
                if !json_file.exists() {
                    continue;
                }

                let source = crate::evaluation::DatasetSource::JsonFile(json_file.clone());
                let tasks = crate::evaluation::load_dataset(source)?;

                // Limit to first 10 for speed if not verbose
                let limit = if self.config.verbose { tasks.len() } else { 10 };
                let tasks_to_run = tasks.iter().take(limit);

                let pb = if self.config.verbose {
                    let p = ProgressBar::new(tasks.len() as u64);
                    p.set_style(
                        ProgressStyle::default_bar()
                            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                            .unwrap()
                            .progress_chars("#>-"),
                    );
                    Some(p)
                } else {
                    None
                };

                let results: Vec<_> = tasks_to_run
                    .par_bridge()
                    .map(|task| {
                        let start = Instant::now();
                        let task_name = task.name();
                        let result = crate::evaluation::run_model_task(task);
                        let duration = start.elapsed();

                        if let Some(ref p) = pb {
                            p.inc(1);
                        }

                        match result {
                            Ok(eval_result) => {
                                let status = if eval_result.comparison.passed {
                                    TestStatus::Passed
                                } else {
                                    TestStatus::Failed
                                };
                                
                                if self.config.verbose && !eval_result.comparison.passed {
                                    println!("  {} {} - Comparison failed", "✗".red(), task_name);
                                    println!("     {}: {:.2}%, {}: {:.2}%", 
                                        "Vertex delta".white(),
                                        eval_result.comparison.vertices_diff,
                                        "Triangle delta".white(),
                                        eval_result.comparison.triangles_diff
                                    );
                                }
                                
                                ValidationResult::Evaluation(EvaluationTestResult {
                                    model: task_name,
                                    status,
                                    duration,
                                    comparison_passed: eval_result.comparison.passed,
                                    error: None,
                                    metrics: Some(eval_result.metrics),
                                })
                            }
                            Err(e) => {
                                let error_msg = format!("{}", e);
                                if self.config.verbose {
                                    println!("  {} {} - Error: {}", "✗".red(), task_name, error_msg);
                                    // Show source code snippet for debugging
                                    if let Ok(source) = task.source() {
                                        let lines: Vec<&str> = source.lines().take(3).collect();
                                        if !lines.is_empty() {
                                            println!("     Source: {}", lines[0]);
                                            if lines.len() > 1 {
                                                println!("             {}", lines[1]);
                                            }
                                        }
                                    }
                                }
                                ValidationResult::Evaluation(EvaluationTestResult {
                                    model: task_name,
                                    status: TestStatus::Error,
                                    duration,
                                    comparison_passed: false,
                                    error: Some(error_msg),
                                    metrics: None,
                                })
                            }
                        }
                    })
                    .collect();

                if let Some(p) = pb {
                    p.finish();
                }

                for result in results {
                    suite_result.add_result(result);
                }
            }
        } else {
            // Run discovered tests
            for test in eval_tests {
                let start = Instant::now();
                let result = crate::evaluation::run_and_compare(&test.path);
                let duration = start.elapsed();

                let validation_result = match result {
                    Ok(eval_result) => ValidationResult::Evaluation(EvaluationTestResult {
                        model: test.path.display().to_string(),
                        status: if eval_result.comparison.passed {
                            TestStatus::Passed
                        } else {
                            TestStatus::Failed
                        },
                        duration,
                        comparison_passed: eval_result.comparison.passed,
                        error: None,
                        metrics: Some(eval_result.metrics),
                    }),
                    Err(e) => ValidationResult::Evaluation(EvaluationTestResult {
                        model: test.path.display().to_string(),
                        status: TestStatus::Error,
                        duration,
                        comparison_passed: false,
                        error: Some(e.to_string()),
                        metrics: None,
                    }),
                };

                suite_result.add_result(validation_result);
            }
        }

        Ok(suite_result)
    }

    /// Run comparison tests
    fn run_comparison_tests(&self) -> Result<SuiteResult> {
        let mut suite_result = SuiteResult::new(TestSuite::Comparison);

        if self.config.verbose {
            println!("{}", "Running comparison tests...".bold().cyan());
        }

        let preview_root = self
            .config
            .generate_visual_diffs
            .then(|| self.config.output_dir.join("previews"));

        // Discover comparison test files
        let tests = TestDiscovery::discover_all()?;
        let comp_tests: Vec<_> = tests
            .iter()
            .filter(|t| t.suite == TestSuite::Comparison)
            .filter(|t| self.config.matches_file_pattern(&t.path))
            .collect();

        if comp_tests.is_empty() {
            // Fallback to examples directory
            let examples_dir = PathBuf::from("examples");
            if examples_dir.exists() {
                for entry in std::fs::read_dir(&examples_dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.extension().map(|s| s == "scad").unwrap_or(false) {
                        let start = Instant::now();
                        let preview = preview_root
                            .as_ref()
                            .map(|root| crate::cli::PreviewConfig::for_input(root, &path));
                        let result = crate::cli::compare_with_openscad(
                            &path,
                            1e-5,
                            self.config.verbose,
                            preview,
                        );
                        let duration = start.elapsed();

                        let validation_result = match result {
                            Ok(comp_result) => {
                                let status = if comp_result.passed {
                                    TestStatus::Passed
                                } else {
                                    TestStatus::Failed
                                };
                                
                                if self.config.verbose && !comp_result.passed {
                                    println!("  {} {} - Comparison failed", "✗".red(), path.display());
                                    println!("     {}: {:.4}%, {}: {:.4}%", 
                                        "Vertex delta".white(),
                                        comp_result.vertex_delta,
                                        "Triangle delta".white(),
                                        comp_result.triangle_delta
                                    );
                                    println!("     {}: Polyframe={}, OpenSCAD={}",
                                        "Vertex counts".white(),
                                        comp_result.vertex_count_a.to_string().white(),
                                        comp_result.vertex_count_b.to_string().white()
                                    );
                                    println!("     {}: Polyframe={}, OpenSCAD={}",
                                        "Triangle counts".white(),
                                        comp_result.triangle_count_a.to_string().white(),
                                        comp_result.triangle_count_b.to_string().white()
                                    );
                                }
                                
                                ValidationResult::Comparison(ComparisonTestResult {
                                    file: path.clone(),
                                    status,
                                    duration,
                                    comparison_passed: comp_result.passed,
                                    vertex_delta: comp_result.vertex_delta as f32,
                                    triangle_delta: comp_result.triangle_delta as f32,
                                    visual_diff_delta: comp_result.visual_diff_delta,
                                    polyframe_preview: comp_result.polyframe_preview.clone(),
                                    openscad_preview: comp_result.openscad_preview.clone(),
                                    diff_preview: comp_result.diff_preview.clone(),
                                    polyframe_stl: comp_result.polyframe_stl.clone(),
                                    openscad_stl: comp_result.openscad_stl.clone(),
                                    error: None,
                                })
                            }
                            Err(e) => {
                                if self.config.verbose {
                                    println!("  {} {} - Error: {}", "✗".red(), path.display(), e);
                                }
                                ValidationResult::Comparison(ComparisonTestResult {
                                    file: path,
                                    status: TestStatus::Error,
                                    duration,
                                    comparison_passed: false,
                                    vertex_delta: 0.0,
                                    triangle_delta: 0.0,
                                    visual_diff_delta: None,
                                    polyframe_preview: None,
                                    openscad_preview: None,
                                    diff_preview: None,
                                    polyframe_stl: None,
                                    openscad_stl: None,
                                    error: Some(e.to_string()),
                                })
                            }
                        };

                        suite_result.add_result(validation_result);
                    }
                }
            }
        } else {
            // Run discovered tests
            for test in comp_tests {
                let start = Instant::now();
                let preview = preview_root
                    .as_ref()
                    .map(|root| crate::cli::PreviewConfig::for_input(root, &test.path));
                let result = crate::cli::compare_with_openscad(
                    &test.path,
                    1e-5,
                    self.config.verbose,
                    preview,
                );
                let duration = start.elapsed();

                let validation_result = match result {
                    Ok(comp_result) => ValidationResult::Comparison(ComparisonTestResult {
                        file: test.path.clone(),
                        status: if comp_result.passed {
                            TestStatus::Passed
                        } else {
                            TestStatus::Failed
                        },
                        duration,
                        comparison_passed: comp_result.passed,
                        vertex_delta: comp_result.vertex_delta as f32,
                        triangle_delta: comp_result.triangle_delta as f32,
                        visual_diff_delta: comp_result.visual_diff_delta,
                        polyframe_preview: comp_result.polyframe_preview.clone(),
                        openscad_preview: comp_result.openscad_preview.clone(),
                        diff_preview: comp_result.diff_preview.clone(),
                        polyframe_stl: comp_result.polyframe_stl.clone(),
                        openscad_stl: comp_result.openscad_stl.clone(),
                        error: None,
                    }),
                    Err(e) => ValidationResult::Comparison(ComparisonTestResult {
                        file: test.path.clone(),
                        status: TestStatus::Error,
                        duration,
                        comparison_passed: false,
                        vertex_delta: 0.0,
                        triangle_delta: 0.0,
                        visual_diff_delta: None,
                        polyframe_preview: None,
                        openscad_preview: None,
                        diff_preview: None,
                        polyframe_stl: None,
                        openscad_stl: None,
                        error: Some(e.to_string()),
                    }),
                };

                suite_result.add_result(validation_result);
            }
        }

        Ok(suite_result)
    }

    /// Run fuzz tests
    fn run_fuzz_tests(&self) -> Result<SuiteResult> {
        let mut suite_result = SuiteResult::new(TestSuite::Fuzz);

        if self.config.verbose {
            println!("{}", "Running fuzz tests...".bold().cyan());
        }

        let start = Instant::now();
        let config = crate::evaluation::FuzzerConfig {
            count: 100, // Reasonable default
            max_depth: 5,
            max_primitives: 10,
        };

        let mut fuzzer = crate::evaluation::Fuzzer::new(config);
        let generated = fuzzer.run();

        let mut parse_success = 0;
        let mut render_success = 0;
        let mut errors = Vec::new();

        for (name, scad_code) in &generated {
            // Test parsing
            match crate::parse_scad(scad_code) {
                Ok(_) => {
                    parse_success += 1;

                    // Test rendering
                    match crate::render(scad_code) {
                        Ok(_) => render_success += 1,
                        Err(e) => {
                            errors.push(format!("{}: Render failed: {}", name, e));
                        }
                    }
                }
                Err(e) => {
                    errors.push(format!("{}: Parse failed: {}", name, e));
                }
            }
        }

        let duration = start.elapsed();

        suite_result.add_result(ValidationResult::Fuzz(FuzzTestResult {
            generated_count: generated.len(),
            parse_success_count: parse_success,
            render_success_count: render_success,
            duration,
            errors,
        }));

        Ok(suite_result)
    }

    /// Run regression tests
    fn run_regression_tests(&self) -> Result<SuiteResult> {
        let mut suite_result = SuiteResult::new(TestSuite::Regression);

        if self.config.verbose {
            println!("{}", "Running regression tests...".bold().cyan());
        }

        // Discover regression tests
        let tests = TestDiscovery::discover_all()?;
        let reg_tests: Vec<_> = tests
            .iter()
            .filter(|t| t.suite == TestSuite::Regression)
            .filter(|t| self.config.matches_file_pattern(&t.path))
            .collect();

        for test in reg_tests {
            let start = Instant::now();
            let result = crate::evaluation::run_and_compare(&test.path);
            let duration = start.elapsed();

            let validation_result = match result {
                Ok(eval_result) => ValidationResult::Regression(RegressionTestResult {
                    file: test.path.clone(),
                    status: if eval_result.comparison.passed {
                        TestStatus::Passed
                    } else {
                        TestStatus::Failed
                    },
                    duration,
                    error: None,
                    fixed: false, // Would need to track previous state
                }),
                Err(e) => ValidationResult::Regression(RegressionTestResult {
                    file: test.path.clone(),
                    status: TestStatus::Error,
                    duration,
                    error: Some(e.to_string()),
                    fixed: false,
                }),
            };

            suite_result.add_result(validation_result);
        }

        Ok(suite_result)
    }
}

