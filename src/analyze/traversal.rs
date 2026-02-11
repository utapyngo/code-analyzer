// Copyright 2024 Block, Inc. (original code from https://github.com/block/goose)
// Copyright 2025 utapyngo (modifications)
// SPDX-License-Identifier: Apache-2.0

use rayon::prelude::*;
use std::path::{Path, PathBuf};

use super::types::{AnalysisResult, EntryType};
use crate::lang;

/// Handles file system traversal for analysis
pub struct FileTraverser;

impl Default for FileTraverser {
    fn default() -> Self {
        Self
    }
}

impl FileTraverser {
    pub fn new() -> Self {
        Self
    }

    /// Validate that a path exists
    pub fn validate_path(&self, path: &Path) -> Result<(), String> {
        if !path.exists() {
            return Err(format!("Path '{}' does not exist", path.display()));
        }
        Ok(())
    }

    /// Collect all files for focused analysis
    pub fn collect_files_for_focused(
        &self,
        path: &Path,
        max_depth: u32,
    ) -> Result<Vec<PathBuf>, String> {
        let files = self.collect_files_recursive(path, 0, max_depth)?;
        Ok(files)
    }

    /// Recursively collect files
    fn collect_files_recursive(
        &self,
        path: &Path,
        current_depth: u32,
        max_depth: u32,
    ) -> Result<Vec<PathBuf>, String> {
        let mut files = Vec::new();

        if path.is_file() {
            let lang_id = lang::get_language_identifier(path);
            if !lang_id.is_empty() {
                files.push(path.to_path_buf());
            }
            return Ok(files);
        }

        // max_depth of 0 means unlimited depth
        if max_depth > 0 && current_depth >= max_depth {
            return Ok(files);
        }

        let entries = std::fs::read_dir(path)
            .map_err(|e| format!("Failed to read directory '{}': {}", path.display(), e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;

            let entry_path = entry.path();

            // Skip hidden directories and common non-source directories
            if let Some(name) = entry_path.file_name().and_then(|n| n.to_str())
                && (name.starts_with('.')
                    || name == "node_modules"
                    || name == "target"
                    || name == "__pycache__"
                    || name == "vendor")
            {
                continue;
            }

            if entry_path.is_file() {
                let lang_id = lang::get_language_identifier(&entry_path);
                if !lang_id.is_empty() {
                    files.push(entry_path);
                }
            } else if entry_path.is_dir() {
                let mut sub_files =
                    self.collect_files_recursive(&entry_path, current_depth + 1, max_depth)?;
                files.append(&mut sub_files);
            }
        }

        Ok(files)
    }

    /// Collect directory results for analysis with parallel processing
    pub fn collect_directory_results<F>(
        &self,
        path: &Path,
        max_depth: u32,
        analyze_file: F,
    ) -> Result<Vec<(PathBuf, EntryType)>, String>
    where
        F: Fn(&Path) -> Result<AnalysisResult, String> + Sync,
    {
        let files_to_analyze = self.collect_files_recursive(path, 0, max_depth)?;

        let results: Result<Vec<_>, String> = files_to_analyze
            .par_iter()
            .map(|file_path| {
                analyze_file(file_path).map(|result| (file_path.clone(), EntryType::File(result)))
            })
            .collect();

        results
    }
}
