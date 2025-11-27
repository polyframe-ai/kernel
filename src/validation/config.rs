// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Validation configuration system

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::types::TestSuite;

/// Validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Test suites to run
    pub suites: Vec<TestSuite>,
    /// Maximum parallel workers
    pub parallelism: Option<usize>,
    /// Timeout for individual tests
    pub test_timeout: Option<Duration>,
    /// Timeout for entire suite
    pub suite_timeout: Option<Duration>,
    /// Output directory for reports
    pub output_dir: PathBuf,
    /// OpenSCAD executable path
    pub openscad_path: Option<String>,
    /// Filter patterns for test names
    pub filters: Vec<String>,
    /// Test file patterns
    pub file_patterns: Vec<String>,
    /// Whether to generate visual diffs
    pub generate_visual_diffs: bool,
    /// Whether to stop on first failure
    pub fail_fast: bool,
    /// Verbose output
    pub verbose: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            suites: vec![
                TestSuite::Unit,
                TestSuite::Integration,
                TestSuite::Evaluation,
                TestSuite::Comparison,
            ],
            parallelism: None, // Auto-detect
            test_timeout: Some(Duration::from_secs(30)),
            suite_timeout: Some(Duration::from_secs(300)),
            output_dir: PathBuf::from("tests/evaluation/outputs"),
            openscad_path: None, // Auto-detect
            filters: Vec::new(),
            file_patterns: Vec::new(),
            generate_visual_diffs: false,
            fail_fast: false,
            verbose: false,
        }
    }
}

impl ValidationConfig {
    /// Load configuration from file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read config file: {:?}", path.as_ref()))?;
        let config: ValidationConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", path.as_ref()))?;
        Ok(config)
    }

    /// Load configuration with environment variable overrides
    pub fn load() -> Result<Self> {
        let mut config = if PathBuf::from("validation.toml").exists() {
            Self::from_file("validation.toml")?
        } else {
            Self::default()
        };

        // Apply environment variable overrides
        if let Ok(openscad) = std::env::var("OPENSCAD_PATH") {
            config.openscad_path = Some(openscad);
        }

        if let Ok(parallelism) = std::env::var("VALIDATION_PARALLELISM") {
            config.parallelism = parallelism.parse().ok();
        }

        if let Ok(verbose) = std::env::var("VALIDATION_VERBOSE") {
            config.verbose = verbose.parse().unwrap_or(false);
        }

        if let Ok(output_dir) = std::env::var("VALIDATION_OUTPUT_DIR") {
            config.output_dir = PathBuf::from(output_dir);
        }

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        std::fs::write(path.as_ref(), content)
            .with_context(|| format!("Failed to write config file: {:?}", path.as_ref()))?;
        Ok(())
    }

    /// Check if a test suite should be run
    pub fn should_run_suite(&self, suite: TestSuite) -> bool {
        self.suites.contains(&suite) || self.suites.is_empty()
    }

    /// Check if a test name matches filters
    pub fn matches_filter(&self, name: &str) -> bool {
        if self.filters.is_empty() {
            return true;
        }
        self.filters.iter().any(|filter| name.contains(filter))
    }

    /// Check if a file path matches patterns
    pub fn matches_file_pattern(&self, path: &Path) -> bool {
        if self.file_patterns.is_empty() {
            return true;
        }
        let path_str = path.to_string_lossy();
        self.file_patterns.iter().any(|pattern| path_str.contains(pattern))
    }
}

