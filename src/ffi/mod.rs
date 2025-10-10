// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! FFI bindings for WASM and Node.js

#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(feature = "napi")]
pub mod bindings;

