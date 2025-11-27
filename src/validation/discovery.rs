// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Test discovery and categorization

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::types::TestSuite;

/// Test category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestCategory {
    Primitives,
    Transforms,
    Booleans,
    Complex,
    Export,
    Performance,
    Other,
}

/// Test complexity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestComplexity {
    Basic,
    Intermediate,
    Advanced,
}

/// Discovered test file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredTest {
    pub path: PathBuf,
    pub suite: TestSuite,
    pub category: Option<TestCategory>,
    pub complexity: Option<TestComplexity>,
    pub tags: Vec<String>,
}

/// Test discovery system
pub struct TestDiscovery;

impl TestDiscovery {
    /// Discover all test files
    pub fn discover_all() -> Result<Vec<DiscoveredTest>> {
        let mut tests = Vec::new();

        // Discover JSON exercise files
        tests.extend(Self::discover_json_exercises()?);

        // Discover SCAD files in test directories
        tests.extend(Self::discover_scad_tests()?);

        // Discover regression tests
        tests.extend(Self::discover_regression_tests()?);

        Ok(tests)
    }

    /// Discover JSON exercise files
    fn discover_json_exercises() -> Result<Vec<DiscoveredTest>> {
        let mut tests = Vec::new();
        let fixtures_dir = PathBuf::from("tests/fixtures");

        if !fixtures_dir.exists() {
            return Ok(tests);
        }

        for entry in WalkDir::new(&fixtures_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|s| s == "json").unwrap_or(false))
        {
            let path = entry.path().to_path_buf();
            let category = Self::categorize_from_path(&path);
            let complexity = Self::assess_complexity(&path);
            let tags = Self::extract_tags(&path);

            tests.push(DiscoveredTest {
                path,
                suite: TestSuite::Evaluation,
                category,
                complexity,
                tags,
            });
        }

        Ok(tests)
    }

    /// Discover SCAD test files
    fn discover_scad_tests() -> Result<Vec<DiscoveredTest>> {
        let mut tests = Vec::new();
        let test_dirs = vec![
            PathBuf::from("examples"),
            PathBuf::from("tests/fixtures/validation"),
        ];

        for test_dir in test_dirs {
            if !test_dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&test_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map(|s| s == "scad").unwrap_or(false))
            {
                let path = entry.path().to_path_buf();
                let category = Self::categorize_from_path(&path);
                let complexity = Self::assess_complexity(&path);
                let tags = Self::extract_tags(&path);

                tests.push(DiscoveredTest {
                    path,
                    suite: TestSuite::Comparison,
                    category,
                    complexity,
                    tags,
                });
            }
        }

        Ok(tests)
    }

    /// Discover regression test files
    fn discover_regression_tests() -> Result<Vec<DiscoveredTest>> {
        let mut tests = Vec::new();
        let regressions_dir = PathBuf::from("tests/evaluation/datasets/regressions");

        if !regressions_dir.exists() {
            return Ok(tests);
        }

        for entry in WalkDir::new(&regressions_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|s| s == "scad").unwrap_or(false))
        {
            let path = entry.path().to_path_buf();
            let category = Self::categorize_from_path(&path);
            let complexity = Self::assess_complexity(&path);

            tests.push(DiscoveredTest {
                path,
                suite: TestSuite::Regression,
                category,
                complexity,
                tags: vec!["regression".to_string()],
            });
        }

        Ok(tests)
    }

    /// Categorize test from file path
    fn categorize_from_path(path: &Path) -> Option<TestCategory> {
        let path_str = path.to_string_lossy().to_lowercase();

        if path_str.contains("primitive") {
            Some(TestCategory::Primitives)
        } else if path_str.contains("transform") || path_str.contains("translate") || path_str.contains("rotate") {
            Some(TestCategory::Transforms)
        } else if path_str.contains("boolean") || path_str.contains("difference") || path_str.contains("union") || path_str.contains("intersection") {
            Some(TestCategory::Booleans)
        } else if path_str.contains("complex") {
            Some(TestCategory::Complex)
        } else if path_str.contains("export") || path_str.contains("stl") || path_str.contains("3mf") {
            Some(TestCategory::Export)
        } else if path_str.contains("performance") || path_str.contains("benchmark") {
            Some(TestCategory::Performance)
        } else {
            None
        }
    }

    /// Assess test complexity from path
    fn assess_complexity(path: &Path) -> Option<TestComplexity> {
        let path_str = path.to_string_lossy().to_lowercase();

        if path_str.contains("basic") || path_str.contains("simple") {
            Some(TestComplexity::Basic)
        } else if path_str.contains("advanced") || path_str.contains("complex") {
            Some(TestComplexity::Advanced)
        } else {
            Some(TestComplexity::Intermediate)
        }
    }

    /// Extract tags from file path
    fn extract_tags(path: &Path) -> Vec<String> {
        let mut tags = Vec::new();
        let path_str = path.to_string_lossy().to_lowercase();

        // Extract directory names as tags
        for component in path.components() {
            if let std::path::Component::Normal(name) = component {
                let name_str = name.to_string_lossy().to_lowercase();
                if !name_str.contains('.') && name_str.len() > 2 {
                    tags.push(name_str);
                }
            }
        }

        // Add specific feature tags
        if path_str.contains("cube") {
            tags.push("cube".to_string());
        }
        if path_str.contains("sphere") {
            tags.push("sphere".to_string());
        }
        if path_str.contains("cylinder") {
            tags.push("cylinder".to_string());
        }

        tags
    }

    /// Filter tests by suite
    pub fn filter_by_suite(tests: &[DiscoveredTest], suite: TestSuite) -> Vec<DiscoveredTest> {
        tests.iter()
            .filter(|t| t.suite == suite)
            .cloned()
            .collect()
    }

    /// Filter tests by category
    pub fn filter_by_category(tests: &[DiscoveredTest], category: TestCategory) -> Vec<DiscoveredTest> {
        tests.iter()
            .filter(|t| t.category == Some(category))
            .cloned()
            .collect()
    }

    /// Filter tests by tags
    pub fn filter_by_tags(tests: &[DiscoveredTest], tags: &[String]) -> Vec<DiscoveredTest> {
        tests.iter()
            .filter(|t| tags.iter().any(|tag| t.tags.contains(tag)))
            .cloned()
            .collect()
    }
}

