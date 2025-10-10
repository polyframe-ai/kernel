// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! Dataset discovery for .scad files and JSON exercises

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Exercise definition from JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exercise {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    pub input: String,
    #[serde(default)]
    pub validation: Option<Validation>,
}

/// Validation criteria for an exercise
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validation {
    pub vertex_count: Option<usize>,
    pub triangle_count: Option<usize>,
    pub bbox: Option<[f32; 6]>, // [minX, minY, minZ, maxX, maxY, maxZ]
}

/// Dataset source type
#[derive(Debug, Clone)]
pub enum DatasetSource {
    Folder(PathBuf),
    JsonFile(PathBuf),
}

/// Model task to execute
#[derive(Debug, Clone)]
pub enum ModelTask {
    FromFile(PathBuf),
    FromJson(Exercise),
}

impl ModelTask {
    /// Get a display name for this task
    pub fn name(&self) -> String {
        match self {
            ModelTask::FromFile(path) => path.file_name().unwrap().to_str().unwrap().to_string(),
            ModelTask::FromJson(exercise) => exercise.id.clone(),
        }
    }

    /// Get SCAD source code
    pub fn source(&self) -> Result<String> {
        match self {
            ModelTask::FromFile(path) => {
                std::fs::read_to_string(path).context(format!("Failed to read {}", path.display()))
            }
            ModelTask::FromJson(exercise) => Ok(exercise.input.clone()),
        }
    }
}

/// Load dataset from a source
pub fn load_dataset(source: DatasetSource) -> Result<Vec<ModelTask>> {
    match source {
        DatasetSource::Folder(path) => discover_scad_files(&path),
        DatasetSource::JsonFile(path) => load_json_exercises(&path),
    }
}

/// Detect dataset source from path
pub fn detect_source(path: &Path) -> DatasetSource {
    if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
        DatasetSource::JsonFile(path.to_path_buf())
    } else {
        DatasetSource::Folder(path.to_path_buf())
    }
}

/// Discover .scad files from a folder
fn discover_scad_files(path: &Path) -> Result<Vec<ModelTask>> {
    let mut models = Vec::new();

    if path.is_file() && path.extension().is_some_and(|ext| ext == "scad") {
        models.push(ModelTask::FromFile(path.to_path_buf()));
    } else if path.is_dir() {
        for entry in WalkDir::new(path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let entry_path = entry.path();
            if entry_path.is_file() && entry_path.extension().is_some_and(|ext| ext == "scad") {
                models.push(ModelTask::FromFile(entry_path.to_path_buf()));
            }
        }
    }

    // Sort for consistent ordering
    models.sort_by_key(|a| a.name());

    Ok(models)
}

/// Load exercises from JSON file
fn load_json_exercises(path: &Path) -> Result<Vec<ModelTask>> {
    let file = std::fs::File::open(path)
        .context(format!("Failed to open JSON file: {}", path.display()))?;

    let exercises: Vec<Exercise> =
        serde_json::from_reader(file).context("Failed to parse JSON exercises")?;

    Ok(exercises.into_iter().map(ModelTask::FromJson).collect())
}

/// Legacy function for backwards compatibility
pub fn discover_models(paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut models = Vec::new();

    for path in paths {
        if path.is_file() && path.extension().is_some_and(|ext| ext == "scad") {
            models.push(path.clone());
        } else if path.is_dir() {
            for entry in WalkDir::new(path)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let entry_path = entry.path();
                if entry_path.is_file() && entry_path.extension().is_some_and(|ext| ext == "scad") {
                    models.push(entry_path.to_path_buf());
                }
            }
        }
    }

    models.sort();
    Ok(models)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_discover_models() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let scad_file = temp_dir.path().join("test.scad");
        fs::write(&scad_file, "cube([10,10,10]);")?;

        let models = discover_models(&[temp_dir.path().to_path_buf()])?;
        assert_eq!(models.len(), 1);
        assert_eq!(models[0], scad_file);

        Ok(())
    }

    #[test]
    fn test_load_dataset_folder() {
        let temp_dir = TempDir::new().unwrap();
        let scad_file = temp_dir.path().join("test.scad");
        fs::write(&scad_file, "cube([10,10,10]);").unwrap();

        let source = DatasetSource::Folder(temp_dir.path().to_path_buf());
        let tasks = load_dataset(source).unwrap();

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name(), "test.scad");
    }
}
