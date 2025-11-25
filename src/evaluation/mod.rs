// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Evaluation harness for comparing Polyframe vs OpenSCAD outputs

pub mod comparator;
pub mod dataset;
pub mod metrics;
pub mod reporter;
pub mod runner;

pub use comparator::{compare_stl_files, Comparison};
pub use dataset::{
    detect_source, discover_models, load_dataset, DatasetSource, Exercise, ModelTask,
};
pub use metrics::Metrics;
pub use reporter::{EvaluationReport, Reporter};
pub use runner::{run_and_compare, run_model_task, run_openscad, run_polyframe, RunResult};

use anyhow::Result;
use std::path::PathBuf;

/// Main evaluation entry point
pub fn evaluate(
    dataset_paths: &[PathBuf],
    output_dir: Option<PathBuf>,
) -> Result<EvaluationReport> {
    let models = discover_models(dataset_paths)?;
    let mut report = EvaluationReport::new();

    for model in models {
        match run_and_compare(&model) {
            Ok(result) => {
                report.add_result(result);
            }
            Err(e) => {
                eprintln!("Error evaluating {}: {}", model.display(), e);
            }
        }
    }

    // Write reports
    let output_dir = output_dir.unwrap_or_else(|| PathBuf::from("evaluation/results"));
    std::fs::create_dir_all(&output_dir)?;

    Reporter::write_json(&report, &output_dir.join("latest.json"))?;
    Reporter::write_markdown(&report, &output_dir.join("report.md"))?;

    Ok(report)
}
