// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Benchmark metrics collection and reporting

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Performance metrics for a single operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationMetrics {
    pub name: String,
    pub duration: Duration,
    pub memory_kb: usize,
    pub mesh_count: usize,
    pub vertex_count: usize,
    pub triangle_count: usize,
}

impl OperationMetrics {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            duration: Duration::default(),
            memory_kb: 0,
            mesh_count: 0,
            vertex_count: 0,
            triangle_count: 0,
        }
    }

    /// Format duration in milliseconds
    pub fn duration_ms(&self) -> f64 {
        self.duration.as_secs_f64() * 1000.0
    }

    /// Format memory in megabytes
    pub fn memory_mb(&self) -> f64 {
        self.memory_kb as f64 / 1024.0
    }
}

/// Benchmark report comparing multiple metrics
#[derive(Debug)]
pub struct BenchmarkReport {
    metrics: HashMap<String, OperationMetrics>,
}

impl BenchmarkReport {
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
        }
    }

    /// Add metrics to the report
    pub fn add_metrics(&mut self, metrics: OperationMetrics) {
        self.metrics.insert(metrics.name.clone(), metrics);
    }

    /// Calculate speedup between two operations
    pub fn speedup(&self, baseline: &str, optimized: &str) -> Option<f64> {
        let baseline_metrics = self.metrics.get(baseline)?;
        let optimized_metrics = self.metrics.get(optimized)?;

        Some(baseline_metrics.duration.as_secs_f64() / optimized_metrics.duration.as_secs_f64())
    }

    /// Print formatted report to console
    pub fn print_report(&self) {
        println!("\n╔══════════════════════════════════════════════════════════════════════╗");
        println!("║              POLYFRAME KERNEL - PERFORMANCE REPORT                  ║");
        println!("╠══════════════════════════════════════════════════════════════════════╣");

        let mut metrics_vec: Vec<_> = self.metrics.values().collect();
        metrics_vec.sort_by(|a, b| a.duration.cmp(&b.duration));

        for metrics in metrics_vec {
            println!("║                                                                      ║");
            println!("║ Operation: {:<58} ║", metrics.name);
            println!("║ ──────────────────────────────────────────────────────────────────── ║");
            println!("║   Time (ms):      {:<48.2} ║", metrics.duration_ms());
            println!("║   Memory (MB):    {:<48.2} ║", metrics.memory_mb());
            println!("║   Mesh Count:     {:<48} ║", metrics.mesh_count);
            println!("║   Vertices:       {:<48} ║", metrics.vertex_count);
            println!("║   Triangles:      {:<48} ║", metrics.triangle_count);
        }

        println!("║                                                                      ║");
        println!("╠══════════════════════════════════════════════════════════════════════╣");

        // Calculate and display speedups
        if let Some(speedup) = self.speedup("full_evaluation", "incremental_cached") {
            println!("║ INCREMENTAL SPEEDUP: {:<46.2}× ║", speedup);
        }

        if let Some(speedup) = self.speedup("sequential", "parallel") {
            println!("║ PARALLEL SPEEDUP:    {:<46.2}× ║", speedup);
        }

        println!("╚══════════════════════════════════════════════════════════════════════╝\n");
    }

    /// Export report as JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self.metrics).unwrap_or_default()
    }
}

impl Default for BenchmarkReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Timer for measuring operation duration
pub struct Timer {
    start: Instant,
}

impl Timer {
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn elapsed_ms(&self) -> f64 {
        self.elapsed().as_secs_f64() * 1000.0
    }
}

/// Estimate memory usage for a mesh
pub fn estimate_mesh_memory(vertex_count: usize, triangle_count: usize) -> usize {
    // Rough estimate:
    // Each vertex: position (12 bytes) + normal (12 bytes) = 24 bytes
    // Each triangle: 3 indices (12 bytes for usize on 64-bit)
    let vertex_bytes = vertex_count * 24;
    let triangle_bytes = triangle_count * 12;
    (vertex_bytes + triangle_bytes) / 1024 // Convert to KB
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = OperationMetrics::new("test_op");
        assert_eq!(metrics.name, "test_op");
        assert_eq!(metrics.duration, Duration::default());
    }

    #[test]
    fn test_benchmark_report() {
        let mut report = BenchmarkReport::new();

        let mut metrics1 = OperationMetrics::new("baseline");
        metrics1.duration = Duration::from_millis(100);

        let mut metrics2 = OperationMetrics::new("optimized");
        metrics2.duration = Duration::from_millis(50);

        report.add_metrics(metrics1);
        report.add_metrics(metrics2);

        let speedup = report.speedup("baseline", "optimized").unwrap();
        assert_eq!(speedup, 2.0);
    }

    #[test]
    fn test_timer() {
        let timer = Timer::start();
        std::thread::sleep(Duration::from_millis(10));
        assert!(timer.elapsed_ms() >= 10.0);
    }
}
