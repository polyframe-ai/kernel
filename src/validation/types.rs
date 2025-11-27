// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Unified validation result types

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::path::PathBuf;
use std::time::Duration;

// Custom serialization for Duration
fn serialize_duration<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_f64(duration.as_secs_f64())
}

fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let secs = f64::deserialize(deserializer)?;
    Ok(Duration::from_secs_f64(secs))
}

/// Test suite type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestSuite {
    Unit,
    Integration,
    Evaluation,
    Comparison,
    Fuzz,
    Regression,
}

impl TestSuite {
    pub fn as_str(&self) -> &'static str {
        match self {
            TestSuite::Unit => "unit",
            TestSuite::Integration => "integration",
            TestSuite::Evaluation => "evaluation",
            TestSuite::Comparison => "comparison",
            TestSuite::Fuzz => "fuzz",
            TestSuite::Regression => "regression",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "unit" => Some(TestSuite::Unit),
            "integration" => Some(TestSuite::Integration),
            "evaluation" => Some(TestSuite::Evaluation),
            "comparison" => Some(TestSuite::Comparison),
            "fuzz" => Some(TestSuite::Fuzz),
            "regression" => Some(TestSuite::Regression),
            _ => None,
        }
    }
}

/// Test result status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Error,
}

/// Unit test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitTestResult {
    pub name: String,
    pub status: TestStatus,
    #[serde(serialize_with = "serialize_duration", deserialize_with = "deserialize_duration")]
    pub duration: Duration,
    pub error: Option<String>,
}

/// Integration test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationTestResult {
    pub name: String,
    pub status: TestStatus,
    #[serde(serialize_with = "serialize_duration", deserialize_with = "deserialize_duration")]
    pub duration: Duration,
    pub error: Option<String>,
    pub file: Option<PathBuf>,
}

/// Evaluation test result (wraps existing EvaluationResult)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationTestResult {
    pub model: String,
    pub status: TestStatus,
    #[serde(serialize_with = "serialize_duration", deserialize_with = "deserialize_duration")]
    pub duration: Duration,
    pub comparison_passed: bool,
    pub error: Option<String>,
    pub metrics: Option<crate::evaluation::Metrics>,
}

/// Comparison test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonTestResult {
    pub file: PathBuf,
    pub status: TestStatus,
    #[serde(serialize_with = "serialize_duration", deserialize_with = "deserialize_duration")]
    pub duration: Duration,
    pub comparison_passed: bool,
    pub vertex_delta: f32,
    pub triangle_delta: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visual_diff_delta: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub polyframe_preview: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openscad_preview: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_preview: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub polyframe_stl: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openscad_stl: Option<PathBuf>,
    pub error: Option<String>,
}

/// Fuzz test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzTestResult {
    pub generated_count: usize,
    pub parse_success_count: usize,
    pub render_success_count: usize,
    #[serde(serialize_with = "serialize_duration", deserialize_with = "deserialize_duration")]
    pub duration: Duration,
    pub errors: Vec<String>,
}

/// Regression test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionTestResult {
    pub file: PathBuf,
    pub status: TestStatus,
    #[serde(serialize_with = "serialize_duration", deserialize_with = "deserialize_duration")]
    pub duration: Duration,
    pub error: Option<String>,
    pub fixed: bool, // True if previously failing test now passes
}

/// Unified validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationResult {
    Unit(UnitTestResult),
    Integration(IntegrationTestResult),
    Evaluation(EvaluationTestResult),
    Comparison(ComparisonTestResult),
    Fuzz(FuzzTestResult),
    Regression(RegressionTestResult),
}

impl ValidationResult {
    pub fn status(&self) -> TestStatus {
        match self {
            ValidationResult::Unit(r) => r.status,
            ValidationResult::Integration(r) => r.status,
            ValidationResult::Evaluation(r) => r.status,
            ValidationResult::Comparison(r) => r.status,
            ValidationResult::Fuzz(r) => {
                if r.errors.is_empty() {
                    TestStatus::Passed
                } else {
                    TestStatus::Failed
                }
            }
            ValidationResult::Regression(r) => r.status,
        }
    }

    pub fn duration(&self) -> Duration {
        match self {
            ValidationResult::Unit(r) => r.duration,
            ValidationResult::Integration(r) => r.duration,
            ValidationResult::Evaluation(r) => r.duration,
            ValidationResult::Comparison(r) => r.duration,
            ValidationResult::Fuzz(r) => r.duration,
            ValidationResult::Regression(r) => r.duration,
        }
    }

    pub fn suite_type(&self) -> TestSuite {
        match self {
            ValidationResult::Unit(_) => TestSuite::Unit,
            ValidationResult::Integration(_) => TestSuite::Integration,
            ValidationResult::Evaluation(_) => TestSuite::Evaluation,
            ValidationResult::Comparison(_) => TestSuite::Comparison,
            ValidationResult::Fuzz(_) => TestSuite::Fuzz,
            ValidationResult::Regression(_) => TestSuite::Regression,
        }
    }
}

/// Suite-level result summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteResult {
    pub suite: TestSuite,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub errors: usize,
    #[serde(serialize_with = "serialize_duration", deserialize_with = "deserialize_duration")]
    pub duration: Duration,
    pub results: Vec<ValidationResult>,
}

impl SuiteResult {
    pub fn new(suite: TestSuite) -> Self {
        Self {
            suite,
            total: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            errors: 0,
            duration: Duration::ZERO,
            results: Vec::new(),
        }
    }

    pub fn add_result(&mut self, result: ValidationResult) {
        self.total += 1;
        match result.status() {
            TestStatus::Passed => self.passed += 1,
            TestStatus::Failed => self.failed += 1,
            TestStatus::Skipped => self.skipped += 1,
            TestStatus::Error => self.errors += 1,
        }
        self.duration += result.duration();
        self.results.push(result);
    }

    pub fn pass_rate(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            (self.passed as f32 / self.total as f32) * 100.0
        }
    }
}

/// Complete validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub timestamp: String,
    pub total_suites: usize,
    pub total_tests: usize,
    pub total_passed: usize,
    pub total_failed: usize,
    pub total_skipped: usize,
    pub total_errors: usize,
    #[serde(serialize_with = "serialize_duration", deserialize_with = "deserialize_duration")]
    pub total_duration: Duration,
    pub suite_results: Vec<SuiteResult>,
    pub compatibility_score: f32, // 0-100 score based on all tests
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            total_suites: 0,
            total_tests: 0,
            total_passed: 0,
            total_failed: 0,
            total_skipped: 0,
            total_errors: 0,
            total_duration: Duration::ZERO,
            suite_results: Vec::new(),
            compatibility_score: 0.0,
        }
    }

    pub fn add_suite_result(&mut self, suite_result: SuiteResult) {
        self.total_suites += 1;
        self.total_tests += suite_result.total;
        self.total_passed += suite_result.passed;
        self.total_failed += suite_result.failed;
        self.total_skipped += suite_result.skipped;
        self.total_errors += suite_result.errors;
        self.total_duration += suite_result.duration;
        self.suite_results.push(suite_result);

        // Recalculate compatibility score
        self.compatibility_score = if self.total_tests > 0 {
            (self.total_passed as f32 / self.total_tests as f32) * 100.0
        } else {
            0.0
        };
    }

    pub fn overall_pass_rate(&self) -> f32 {
        if self.total_tests == 0 {
            0.0
        } else {
            (self.total_passed as f32 / self.total_tests as f32) * 100.0
        }
    }

    pub fn has_failures(&self) -> bool {
        self.total_failed > 0 || self.total_errors > 0
    }
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

