// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Unified validation system for OpenSCAD compatibility testing

pub mod config;
pub mod coordinator;
pub mod discovery;
pub mod reporter;
pub mod types;

pub use config::ValidationConfig;
pub use coordinator::ValidationCoordinator;
pub use discovery::{DiscoveredTest, TestCategory, TestComplexity, TestDiscovery};
pub use reporter::ValidationReporter;
pub use types::{
    ComparisonTestResult, EvaluationTestResult, FuzzTestResult, IntegrationTestResult,
    RegressionTestResult, SuiteResult, TestStatus, TestSuite, UnitTestResult, ValidationReport,
    ValidationResult,
};

