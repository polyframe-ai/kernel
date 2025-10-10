// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Integration tests for evaluation harness

use polyframe::evaluation;
use std::path::PathBuf;

#[test]
fn test_evaluation_basic_models() {
    // Discover models in examples directory
    let dataset = evaluation::discover_models(&[PathBuf::from("examples/")])
        .expect("Failed to discover models");
    
    if dataset.is_empty() {
        println!("No models found in examples/, skipping test");
        return;
    }
    
    println!("Testing {} models", dataset.len());
    
    let mut results = Vec::new();
    
    for model in &dataset {
        println!("Evaluating: {}", model.display());
        
        match evaluation::run_and_compare(model) {
            Ok(result) => {
                println!("  Polyframe time: {}ms", result.polyframe_result.time_ms);
                if let Some(ref openscad) = result.openscad_result {
                    println!("  OpenSCAD time: {}ms", openscad.time_ms);
                    println!("  Speedup: {:.1}×", result.metrics.speedup_ratio);
                }
                println!("  Passed: {}", result.comparison.passed);
                results.push(result);
            }
            Err(e) => {
                // Don't fail test if OpenSCAD is not available
                println!("  Skipped: {}", e);
            }
        }
    }
    
    // At least some models should have been evaluated
    assert!(!results.is_empty(), "No models were successfully evaluated");
}

#[test]
fn test_dataset_discovery() {
    let models = evaluation::discover_models(&[PathBuf::from("examples/")])
        .expect("Failed to discover models");
    
    // Should find at least the example files we created
    assert!(models.len() >= 6, "Expected at least 6 example models, found {}", models.len());
    
    // All should end with .scad
    for model in &models {
        assert!(model.extension().unwrap() == "scad");
    }
}

#[test]
fn test_report_generation() {
    use tempfile::TempDir;
    
    // Create a test report
    let mut report = evaluation::reporter::EvaluationReport::new();
    
    // Simulate adding a result
    // (in real test, this would come from run_and_compare)
    
    assert_eq!(report.total_models, 0);
    assert_eq!(report.passed, 0);
    assert_eq!(report.failed, 0);
    
    // Test JSON writing
    let temp_dir = TempDir::new().unwrap();
    let json_path = temp_dir.path().join("test.json");
    let md_path = temp_dir.path().join("test.md");
    
    evaluation::Reporter::write_json(&report, &json_path)
        .expect("Failed to write JSON");
    evaluation::Reporter::write_markdown(&report, &md_path)
        .expect("Failed to write Markdown");
    
    assert!(json_path.exists());
    assert!(md_path.exists());
    
    // Verify JSON can be read back
    let json_content = std::fs::read_to_string(&json_path).unwrap();
    let _parsed: evaluation::reporter::EvaluationReport = 
        serde_json::from_str(&json_content).unwrap();
}

#[test]
fn test_json_exercise_suite() {
    use polyframe::evaluation;
    
    let json_path = PathBuf::from("tests/fixtures/polyframe_exercises_001_040.json");
    
    if !json_path.exists() {
        println!("JSON test file not found, skipping");
        return;
    }
    
    let source = evaluation::DatasetSource::JsonFile(json_path);
    let tasks = evaluation::load_dataset(source).expect("Failed to load JSON exercises");
    
    assert!(!tasks.is_empty(), "No exercises found in JSON file");
    println!("Loaded {} exercises from JSON", tasks.len());
    
    // Test first 3 exercises (for speed)
    for task in tasks.iter().take(3) {
        println!("Testing: {}", task.name());
        
        match evaluation::run_model_task(task) {
            Ok(result) => {
                println!("  Polyframe time: {}ms", result.polyframe_result.time_ms);
                println!("  Passed: {}", result.comparison.passed);
                // Note: We don't assert passed=true because OpenSCAD may not be available
            }
            Err(e) => {
                println!("  Skipped: {}", e);
            }
        }
    }
}

#[test]
fn test_json_loading() {
    use polyframe::evaluation;
    
    let json_path = PathBuf::from("tests/fixtures/polyframe_exercises_001_040.json");
    
    if !json_path.exists() {
        println!("JSON file not found, skipping");
        return;
    }
    
    let source = evaluation::DatasetSource::JsonFile(json_path);
    let tasks = evaluation::load_dataset(source).expect("Failed to load exercises");
    
    assert_eq!(tasks.len(), 40, "Expected 40 exercises in 001_040 JSON");
    
    // Verify first exercise structure
    if let evaluation::ModelTask::FromJson(exercise) = &tasks[0] {
        assert_eq!(exercise.id, "001_basic_cube");
        assert!(exercise.title.is_some());
        assert!(!exercise.input.is_empty());
    } else {
        panic!("Expected FromJson task");
    }
}

