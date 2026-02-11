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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_existing_path() {
        let t = FileTraverser::new();
        assert!(
            t.validate_path(Path::new(env!("CARGO_MANIFEST_DIR")))
                .is_ok()
        );
    }

    #[test]
    fn validate_nonexistent_path() {
        let t = FileTraverser::new();
        let result = t.validate_path(Path::new("/tmp/nonexistent_xyz_12345"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));
    }

    #[test]
    fn collect_files_from_fixtures() {
        let t = FileTraverser::new();
        let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        let files = t.collect_files_for_focused(&fixtures, 3).unwrap();
        assert!(
            files.len() >= 4,
            "expected at least 4 fixture files, got {}",
            files.len()
        );

        let names: Vec<String> = files
            .iter()
            .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .collect();
        assert!(names.contains(&"sample.rs".to_string()));
        assert!(names.contains(&"sample.py".to_string()));
        assert!(names.contains(&"sample.js".to_string()));
        assert!(names.contains(&"sample.go".to_string()));
    }

    #[test]
    fn collect_files_skips_hidden_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let hidden = dir.path().join(".hidden");
        std::fs::create_dir(&hidden).unwrap();
        std::fs::write(hidden.join("secret.rs"), "fn hidden() {}").unwrap();
        std::fs::write(dir.path().join("visible.rs"), "fn visible() {}").unwrap();

        let t = FileTraverser::new();
        let files = t.collect_files_for_focused(dir.path(), 3).unwrap();

        let names: Vec<String> = files
            .iter()
            .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .collect();
        assert!(names.contains(&"visible.rs".to_string()));
        assert!(!names.contains(&"secret.rs".to_string()));
    }

    #[test]
    fn collect_files_respects_max_depth() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("top.rs"), "fn top() {}").unwrap();
        let sub = dir.path().join("sub");
        std::fs::create_dir(&sub).unwrap();
        std::fs::write(sub.join("deep.rs"), "fn deep() {}").unwrap();

        let t = FileTraverser::new();
        // max_depth=1 should only get top-level files
        let files = t.collect_files_for_focused(dir.path(), 1).unwrap();
        let names: Vec<String> = files
            .iter()
            .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .collect();
        assert!(names.contains(&"top.rs".to_string()));
        assert!(!names.contains(&"deep.rs".to_string()));
    }

    #[test]
    fn collect_files_unlimited_depth() {
        let dir = tempfile::tempdir().unwrap();
        let deep = dir.path().join("a").join("b").join("c");
        std::fs::create_dir_all(&deep).unwrap();
        std::fs::write(deep.join("deep.rs"), "fn deep() {}").unwrap();

        let t = FileTraverser::new();
        let files = t.collect_files_for_focused(dir.path(), 0).unwrap();
        let names: Vec<String> = files
            .iter()
            .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .collect();
        assert!(names.contains(&"deep.rs".to_string()));
    }

    #[test]
    fn collect_files_ignores_non_source() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("readme.txt"), "hello").unwrap();
        std::fs::write(dir.path().join("code.rs"), "fn f() {}").unwrap();

        let t = FileTraverser::new();
        let files = t.collect_files_for_focused(dir.path(), 3).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].to_string_lossy().contains("code.rs"));
    }

    #[test]
    fn collect_directory_results_works() {
        let t = FileTraverser::new();
        let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        let results = t
            .collect_directory_results(&fixtures, 3, |_path| Ok(AnalysisResult::empty(1)))
            .unwrap();
        assert!(results.len() >= 4);
    }

    #[test]
    fn default_traverser() {
        let _t = FileTraverser::default();
    }
}
