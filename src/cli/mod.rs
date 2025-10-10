// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! CLI subsystem for Polyframe Kernel

pub mod runner;
pub mod diff;
pub mod reporter;
pub mod compare;

pub use runner::Runner;
pub use diff::{MeshDiff, ComparisonResult};
pub use reporter::Reporter;
pub use compare::{compare_with_openscad, batch_compare};

