// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! SCAD file importer

use anyhow::{Result, Context};
use crate::ast::Node;
use std::fs;

/// Import a .scad file and parse it into an AST
pub fn import_scad_file(path: &str) -> Result<Node> {
    let source = fs::read_to_string(path)
        .context(format!("Failed to read SCAD file: {}", path))?;
    
    super::parse_scad(&source)
        .context(format!("Failed to parse SCAD file: {}", path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_import_scad_file() -> Result<()> {
        let mut file = NamedTempFile::new()?;
        writeln!(file, "cube([10, 10, 10]);")?;
        
        let result = import_scad_file(file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        Ok(())
    }
}

