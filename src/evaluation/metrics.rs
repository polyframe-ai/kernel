// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Performance metrics

use serde::{Deserialize, Serialize};

/// Performance metrics for a model evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub openscad_time_ms: u128,
    pub polyframe_time_ms: u128,
    pub speedup_ratio: f32,
}

impl Metrics {
    /// Create new metrics
    pub fn new(openscad_time_ms: u128, polyframe_time_ms: u128) -> Self {
        let speedup_ratio = if polyframe_time_ms > 0 {
            openscad_time_ms as f32 / polyframe_time_ms as f32
        } else {
            0.0
        };

        Self {
            openscad_time_ms,
            polyframe_time_ms,
            speedup_ratio,
        }
    }

    /// Format time in human-readable format
    pub fn format_time(ms: u128) -> String {
        if ms < 1000 {
            format!("{}ms", ms)
        } else {
            format!("{:.2}s", ms as f64 / 1000.0)
        }
    }

    /// Get formatted OpenSCAD time
    pub fn openscad_time_str(&self) -> String {
        Self::format_time(self.openscad_time_ms)
    }

    /// Get formatted Polyframe time
    pub fn polyframe_time_str(&self) -> String {
        Self::format_time(self.polyframe_time_ms)
    }

    /// Get formatted speedup
    pub fn speedup_str(&self) -> String {
        if self.speedup_ratio > 0.0 {
            format!("{:.1}×", self.speedup_ratio)
        } else {
            "N/A".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = Metrics::new(200, 40);
        assert_eq!(metrics.speedup_ratio, 5.0);
    }

    #[test]
    fn test_format_time() {
        assert_eq!(Metrics::format_time(500), "500ms");
        assert_eq!(Metrics::format_time(1500), "1.50s");
    }
}
