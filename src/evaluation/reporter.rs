// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Report generation (JSON and Markdown)

use super::runner::EvaluationResult;
use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Error information for failed evaluations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationError {
    pub model: String,
    pub error: String,
}

/// Complete evaluation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationReport {
    pub timestamp: String,
    pub total_models: usize,
    pub passed: usize,
    pub failed: usize,
    pub errors: usize,
    pub avg_speedup: f32,
    pub results: Vec<EvaluationResult>,
    pub error_details: Vec<EvaluationError>,
}

impl EvaluationReport {
    pub fn new() -> Self {
        Self {
            timestamp: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            total_models: 0,
            passed: 0,
            failed: 0,
            errors: 0,
            avg_speedup: 0.0,
            results: Vec::new(),
            error_details: Vec::new(),
        }
    }

    pub fn add_result(&mut self, result: EvaluationResult) {
        if result.comparison.passed {
            self.passed += 1;
        } else {
            self.failed += 1;
        }

        self.total_models += 1;
        self.results.push(result);

        // Recalculate average speedup
        let total_speedup: f32 = self.results.iter().map(|r| r.metrics.speedup_ratio).sum();

        self.avg_speedup = if self.total_models > 0 {
            total_speedup / self.total_models as f32
        } else {
            0.0
        };
    }

    pub fn add_error(&mut self, model: String, error: String) {
        self.total_models += 1;
        self.errors += 1;
        self.error_details.push(EvaluationError { model, error });
    }

    pub fn pass_rate(&self) -> f32 {
        if self.total_models == 0 {
            0.0
        } else {
            (self.passed as f32 / self.total_models as f32) * 100.0
        }
    }

    pub fn success_rate(&self) -> f32 {
        let successful = self.total_models - self.errors;
        if self.total_models == 0 {
            0.0
        } else {
            (successful as f32 / self.total_models as f32) * 100.0
        }
    }
}

impl Default for EvaluationReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Report writer
pub struct Reporter;

impl Reporter {
    /// Write JSON report
    pub fn write_json(report: &EvaluationReport, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(report)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Write Markdown report
    pub fn write_markdown(report: &EvaluationReport, path: &Path) -> Result<()> {
        let mut md = String::new();

        // Header
        md.push_str(&format!(
            "# Polyframe Evaluation Report ({})\n\n",
            Utc::now().format("%Y-%m-%d")
        ));

        // Summary
        md.push_str("## Summary\n\n");
        md.push_str(&format!("- **Total Tests**: {}\n", report.total_models));
        
        let minor_diffs = report.results.iter()
            .filter(|r| !r.comparison.passed && r.comparison.vertices_diff < 0.1)
            .count();
        
        md.push_str(&format!(
            "- **Passed**: {} ({:.1}%)\n",
            report.passed,
            report.pass_rate()
        ));
        md.push_str(&format!(
            "- **Minor Differences**: {} ({:.1}%)\n",
            minor_diffs,
            if report.total_models > 0 {
                (minor_diffs as f32 / report.total_models as f32) * 100.0
            } else {
                0.0
            }
        ));
        md.push_str(&format!(
            "- **Failures**: {} ({:.1}%)\n",
            report.failed,
            if report.total_models > 0 {
                (report.failed as f32 / report.total_models as f32) * 100.0
            } else {
                0.0
            }
        ));
        md.push_str(&format!("- **Errors**: {}\n", report.errors));
        md.push_str(&format!(
            "- **Average Speedup**: {:.2}×\n\n",
            report.avg_speedup
        ));

        // Table header
        md.push_str("## Detailed Results\n\n");
        md.push_str("| Model | OpenSCAD Time | Polyframe Time | ΔVertices | ΔTriangles | ΔBBox | Speedup | Pass |\n");
        md.push_str("|-------|---------------|----------------|-----------|------------|-------|---------|------|\n");

        // Table rows
        for result in &report.results {
            let model_name = std::path::Path::new(&result.model)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap();

            let openscad_time = if let Some(ref openscad) = result.openscad_result {
                format!("{}ms", openscad.time_ms)
            } else {
                "N/A".to_string()
            };

            let polyframe_time = format!("{}ms", result.polyframe_result.time_ms);
            let vertices_diff = format!("{:.1}%", result.comparison.vertices_diff * 100.0);
            let triangles_diff = format!("{:.1}%", result.comparison.triangles_diff * 100.0);
            let bbox_diff = format!("{:.5}", result.comparison.bbox_diff);
            let speedup = result.metrics.speedup_str();
            let pass = if result.comparison.passed {
                "✅"
            } else {
                "❌"
            };

            md.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
                model_name,
                openscad_time,
                polyframe_time,
                vertices_diff,
                triangles_diff,
                bbox_diff,
                speedup,
                pass
            ));
        }

        // Delta statistics
        if !report.results.is_empty() {
            md.push_str("\n## Deltas\n\n");
            md.push_str("| Metric | Avg Diff | Max Diff |\n");
            md.push_str("|--------|----------|----------|\n");
            
            let vertex_diffs: Vec<f32> = report.results.iter()
                .map(|r| r.comparison.vertices_diff * 100.0)
                .collect();
            let triangle_diffs: Vec<f32> = report.results.iter()
                .map(|r| r.comparison.triangles_diff * 100.0)
                .collect();
            let bbox_diffs: Vec<f32> = report.results.iter()
                .map(|r| r.comparison.bbox_diff)
                .collect();
            
            let avg_vertex = vertex_diffs.iter().sum::<f32>() / vertex_diffs.len() as f32;
            let max_vertex = vertex_diffs.iter().fold(0.0f32, |a, &b| a.max(b));
            
            let avg_triangle = triangle_diffs.iter().sum::<f32>() / triangle_diffs.len() as f32;
            let max_triangle = triangle_diffs.iter().fold(0.0f32, |a, &b| a.max(b));
            
            let avg_bbox = bbox_diffs.iter().sum::<f32>() / bbox_diffs.len() as f32;
            let max_bbox = bbox_diffs.iter().fold(0.0f32, |a, &b| a.max(b));
            
            md.push_str(&format!(
                "| Vertex Count | {:.2}% | {:.2}% |\n",
                avg_vertex, max_vertex
            ));
            md.push_str(&format!(
                "| Triangle Count | {:.2}% | {:.2}% |\n",
                avg_triangle, max_triangle
            ));
            md.push_str(&format!(
                "| Bounding Box | {:.5} | {:.5} |\n",
                avg_bbox, max_bbox
            ));
        }

        // Failed models section
        if report.failed > 0 {
            md.push_str("\n## Failures\n\n");
            md.push_str("| File | Error | Notes |\n");
            md.push_str("|------|-------|-------|\n");
            
            for result in &report.results {
                if !result.comparison.passed {
                    let model_name = std::path::Path::new(&result.model)
                        .file_name()
                        .unwrap_or_else(|| std::path::Path::new(&result.model).as_os_str())
                        .to_str()
                        .unwrap();

                    let error_note = if result.comparison.vertices_diff > 0.35 {
                        format!("Vertex mismatch {:.1}%", result.comparison.vertices_diff * 100.0)
                    } else if result.comparison.bbox_diff > 0.001 {
                        "Bbox diverged".to_string()
                    } else {
                        "Minor differences".to_string()
                    };
                    
                    md.push_str(&format!(
                        "| {} | {} | {} |\n",
                        model_name,
                        if result.comparison.vertex_count_poly == 0 {
                            "Polyframe crash"
                        } else if result.comparison.vertex_count_openscad == 0 {
                            "OpenSCAD crash"
                        } else {
                            "Geometry mismatch"
                        },
                        error_note
                    ));
                }
            }
        }

        // Errors section
        if report.errors > 0 {
            md.push_str("\n## Execution Errors\n\n");
            md.push_str(&format!("{} models failed to execute:\n\n", report.errors));
            for error in &report.error_details {
                md.push_str(&format!("- ⚠️ **{}**\n", error.model));
                md.push_str(&format!("  ```\n  {}\n  ```\n", error.error));
            }
        }

        // Visual Diffs section (if available)
        md.push_str("\n## Visual Diffs\n\n");
        md.push_str("Visual diff images are available in `tests/evaluation/outputs/diffs/` for failed tests.\n\n");

        // Footer
        md.push_str(&format!("\n---\n\n*Generated on {}*\n", report.timestamp));

        fs::write(path, md)?;
        Ok(())
    }

    /// Generate report from existing JSON report file
    pub fn generate_report(json_path: &Path, output_path: &Path) -> Result<()> {
        let json_content = fs::read_to_string(json_path)
            .context(format!("Failed to read JSON report: {}", json_path.display()))?;
        let report: EvaluationReport = serde_json::from_str(&json_content)
            .context("Failed to parse JSON report")?;
        Self::write_markdown(&report, output_path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_creation() {
        let report = EvaluationReport::new();
        assert_eq!(report.total_models, 0);
        assert_eq!(report.passed, 0);
        assert_eq!(report.failed, 0);
    }
}
