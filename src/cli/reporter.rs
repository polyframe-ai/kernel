// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! CLI output reporter with colored formatting

use super::diff::ComparisonResult;
use colored::*;
use std::time::Duration;

/// CLI reporter for formatted output
pub struct Reporter;

impl Reporter {
    /// Report comparison result with colors
    pub fn report_comparison(
        file: &str,
        result: &ComparisonResult,
        poly_time: Duration,
        openscad_time: Option<Duration>,
    ) {
        println!("\n{}", "━".repeat(80).bright_black());
        println!("{} {}", "File:".bold(), file.cyan());
        println!("{}", "━".repeat(80).bright_black());

        // Status
        if result.passed {
            println!(
                "{} {}",
                "✅".green(),
                "Geometry equivalence passed".green().bold()
            );
        } else {
            println!(
                "{} {}",
                "❌".red(),
                "Geometry equivalence failed".red().bold()
            );
        }

        // Metrics
        println!("\n{}", "Metrics:".bold());
        Self::print_metric(
            "Vertices",
            &format!("{} vs {}", result.vertex_count_a, result.vertex_count_b),
            result.vertex_delta,
        );
        Self::print_metric(
            "Triangles",
            &format!("{} vs {}", result.triangle_count_a, result.triangle_count_b),
            result.triangle_delta,
        );
        Self::print_metric(
            "BBox Delta",
            &format!("{:.5}", result.bbox_delta),
            result.bbox_delta,
        );

        // Show note if present
        if let Some(ref note) = result.note {
            println!("\n{}", note.yellow());
        }

        // Timing
        println!("\n{}", "Performance:".bold());
        if let Some(openscad_duration) = openscad_time {
            let speedup = openscad_duration.as_secs_f64() / poly_time.as_secs_f64();
            println!(
                "  {} {:>8} | {} {:>8} | {} {:.2}x",
                "OpenSCAD:".bright_black(),
                Self::format_duration(openscad_duration).yellow(),
                "Polyframe:".bright_black(),
                Self::format_duration(poly_time).cyan(),
                "Speedup:".bright_black(),
                speedup
            );
        } else {
            println!(
                "  {} {}",
                "Polyframe:".bright_black(),
                Self::format_duration(poly_time).cyan()
            );
        }

        println!("{}", "━".repeat(80).bright_black());
    }

    /// Report simple render result
    pub fn report_render(file: &str, vertices: usize, triangles: usize, duration: Duration) {
        println!("\n{}", "━".repeat(80).bright_black());
        println!("{} {}", "Rendered:".bold(), file.cyan());
        println!("{}", "━".repeat(80).bright_black());
        println!(
            "  {} {}",
            "Vertices:".bright_black(),
            vertices.to_string().cyan()
        );
        println!(
            "  {} {}",
            "Triangles:".bright_black(),
            triangles.to_string().cyan()
        );
        println!(
            "  {} {}",
            "Time:".bright_black(),
            Self::format_duration(duration).yellow()
        );
        println!("{}", "━".repeat(80).bright_black());
    }

    /// Report error
    pub fn report_error(message: &str) {
        eprintln!("\n{} {}", "❌ Error:".red().bold(), message);
    }

    /// Report warning
    pub fn report_warning(message: &str) {
        println!("\n{} {}", "⚠️  Warning:".yellow().bold(), message);
    }

    /// Report info
    pub fn report_info(message: &str) {
        println!("{} {}", "ℹ️".bright_blue(), message);
    }

    /// Print a metric with color coding based on delta
    fn print_metric(name: &str, value: &str, delta: f64) {
        let color = if delta < 0.01 {
            "green"
        } else if delta < 0.05 {
            "yellow"
        } else {
            "red"
        };

        let formatted_value = match color {
            "green" => value.green(),
            "yellow" => value.yellow(),
            _ => value.red(),
        };

        let delta_str = if delta > 0.0 {
            format!("(Δ{:.2}%)", delta * 100.0)
        } else {
            String::new()
        };

        println!(
            "  {} {} {}",
            format!("{}:", name).bright_black(),
            formatted_value,
            delta_str.bright_black()
        );
    }

    /// Format duration for display
    fn format_duration(duration: Duration) -> String {
        let micros = duration.as_micros();

        if micros < 1_000 {
            format!("{}µs", micros)
        } else if micros < 1_000_000 {
            format!("{:.2}ms", micros as f64 / 1_000.0)
        } else {
            format!("{:.2}s", micros as f64 / 1_000_000.0)
        }
    }

    /// Print progress bar
    pub fn progress(message: &str) {
        println!("{} {}...", "⏳".bright_blue(), message.bright_black());
    }

    /// Print success message
    pub fn success(message: &str) {
        println!("{} {}", "✅".green(), message.green());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(
            Reporter::format_duration(Duration::from_micros(500)),
            "500µs"
        );
        assert_eq!(
            Reporter::format_duration(Duration::from_millis(5)),
            "5.00ms"
        );
        assert_eq!(Reporter::format_duration(Duration::from_secs(2)), "2.00s");
    }
}
