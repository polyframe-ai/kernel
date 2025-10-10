// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! CLI subsystem for Polyframe Kernel

pub mod compare;
pub mod diff;
pub mod reporter;
pub mod runner;

pub use compare::{batch_compare, compare_with_openscad};
pub use diff::{ComparisonResult, MeshDiff};
pub use reporter::Reporter;
pub use runner::Runner;
