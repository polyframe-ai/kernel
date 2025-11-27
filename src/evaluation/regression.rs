// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Regression suite manager
//! Automatically tracks failed tests and supports regression replay

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Regression metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionMetadata {
    pub file_name: String,
    pub original_path: PathBuf,
    pub added_date: String,
    pub expected_behavior: String,
    pub error_message: Option<String>,
    pub openscad_result: Option<String>,
    pub polyframe_result: Option<String>,
}

/// Regression suite manager
pub struct RegressionSuite {
    regressions_dir: PathBuf,
}

impl RegressionSuite {
    /// Create a new regression suite manager
    pub fn new(regressions_dir: impl AsRef<Path>) -> Self {
        Self {
            regressions_dir: regressions_dir.as_ref().to_path_buf(),
        }
    }

    /// Initialize the regression directory
    pub fn initialize(&self) -> Result<()> {
        fs::create_dir_all(&self.regressions_dir)
            .context("Failed to create regressions directory")?;
        Ok(())
    }

    /// Add a failed test to the regression suite
    pub fn add_regression(
        &self,
        scad_file: &Path,
        error_message: Option<&str>,
        expected_behavior: &str,
        openscad_result: Option<&str>,
        polyframe_result: Option<&str>,
    ) -> Result<PathBuf> {
        self.initialize()?;

        let file_name = scad_file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.scad")
            .to_string();

        // Copy SCAD file to regressions directory
        let regression_path = self.regressions_dir.join(&file_name);
        fs::copy(scad_file, &regression_path)
            .context(format!("Failed to copy {} to regressions", scad_file.display()))?;

        // Create metadata
        let metadata = RegressionMetadata {
            file_name: file_name.clone(),
            original_path: scad_file.to_path_buf(),
            added_date: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            expected_behavior: expected_behavior.to_string(),
            error_message: error_message.map(|s| s.to_string()),
            openscad_result: openscad_result.map(|s| s.to_string()),
            polyframe_result: polyframe_result.map(|s| s.to_string()),
        };

        // Write metadata JSON
        let metadata_path = self.regressions_dir.join(format!("{}.json", file_name));
        let json = serde_json::to_string_pretty(&metadata)
            .context("Failed to serialize regression metadata")?;
        fs::write(&metadata_path, json)
            .context("Failed to write regression metadata")?;

        Ok(regression_path)
    }

    /// Load all regressions from the directory
    pub fn load_regressions(&self) -> Result<Vec<RegressionMetadata>> {
        self.initialize()?;

        let mut regressions = Vec::new();

        for entry in fs::read_dir(&self.regressions_dir)
            .context("Failed to read regressions directory")?
        {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "json") {
                let metadata: RegressionMetadata = serde_json::from_str(
                    &fs::read_to_string(&path)
                        .context(format!("Failed to read {}", path.display()))?,
                )
                .context(format!("Failed to parse {}", path.display()))?;
                regressions.push(metadata);
            }
        }

        // Sort by date
        regressions.sort_by_key(|r| r.added_date.clone());

        Ok(regressions)
    }

    /// Get regression file path by name
    pub fn get_regression_file(&self, name: &str) -> PathBuf {
        self.regressions_dir.join(name)
    }

    /// Replay a regression test
    pub fn replay_regression(&self, name: &str) -> Result<RegressionMetadata> {
        let metadata_path = self.regressions_dir.join(format!("{}.json", name));
        let metadata: RegressionMetadata = serde_json::from_str(
            &fs::read_to_string(&metadata_path)
                .context(format!("Failed to read {}", metadata_path.display()))?,
        )
        .context(format!("Failed to parse {}", metadata_path.display()))?;

        Ok(metadata)
    }

    /// Get the regressions directory path
    pub fn regressions_dir(&self) -> &Path {
        &self.regressions_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_regression_suite() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let suite = RegressionSuite::new(temp_dir.path());
        
        suite.initialize()?;
        
        // Create a test SCAD file
        let test_scad = temp_dir.path().join("test.scad");
        fs::write(&test_scad, "cube([10,10,10]);")?;
        
        // Add regression
        let regression_path = suite.add_regression(
            &test_scad,
            Some("Test error"),
            "Should render a cube",
            Some("Success"),
            Some("Failed"),
        )?;
        
        assert!(regression_path.exists());
        
        // Load regressions
        let regressions = suite.load_regressions()?;
        assert_eq!(regressions.len(), 1);
        assert_eq!(regressions[0].file_name, "test.scad");
        
        Ok(())
    }
}

