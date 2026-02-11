// Copyright 2024 Block, Inc. (original code from https://github.com/block/goose)
// Copyright 2025 utapyngo (modifications)
// SPDX-License-Identifier: Apache-2.0

pub mod cache;
pub mod formatter;
pub mod graph;
pub mod languages;
pub mod parser;
pub mod traversal;
pub mod types;

use std::path::{Path, PathBuf};

use self::cache::AnalysisCache;
use self::formatter::Formatter;
use self::graph::CallGraph;
use self::parser::{ElementExtractor, ParserManager};
use self::traversal::FileTraverser;
use self::types::{AnalysisMode, AnalysisResult, FocusedAnalysisData};

use crate::lang;

/// Helper to safely lock a mutex with poison recovery
pub(crate) fn lock_or_recover<T, F>(
    mutex: &std::sync::Mutex<T>,
    recovery: F,
) -> std::sync::MutexGuard<'_, T>
where
    F: FnOnce(&mut T),
{
    mutex.lock().unwrap_or_else(|poisoned| {
        let mut guard = poisoned.into_inner();
        recovery(&mut guard);
        guard
    })
}

/// Code analyzer with caching and tree-sitter parsing
#[derive(Clone)]
pub struct CodeAnalyzer {
    parser_manager: ParserManager,
    cache: AnalysisCache,
}

impl Default for CodeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeAnalyzer {
    pub fn new() -> Self {
        Self {
            parser_manager: ParserManager::new(),
            cache: AnalysisCache::new(100),
        }
    }

    fn determine_mode(&self, focus: &Option<String>, path: &Path) -> AnalysisMode {
        if focus.is_some() {
            return AnalysisMode::Focused;
        }

        if path.is_file() {
            AnalysisMode::Semantic
        } else {
            AnalysisMode::Structure
        }
    }

    fn analyze_file(
        &self,
        path: &Path,
        mode: &AnalysisMode,
        ast_recursion_limit: Option<usize>,
    ) -> Result<AnalysisResult, String> {
        let metadata = std::fs::metadata(path)
            .map_err(|e| format!("Failed to get metadata for '{}': {}", path.display(), e))?;

        let modified = metadata.modified().map_err(|e| {
            format!(
                "Failed to get modification time for '{}': {}",
                path.display(),
                e
            )
        })?;

        if let Some(cached) = self.cache.get(path, modified, mode) {
            return Ok(cached);
        }

        let content = match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(_) => {
                return Ok(AnalysisResult::empty(0));
            }
        };

        let line_count = content.lines().count();

        let language = lang::get_language_identifier(path);
        if language.is_empty() {
            return Ok(AnalysisResult::empty(line_count));
        }

        let language_supported = languages::get_language_info(language)
            .map(|info| !info.element_query.is_empty())
            .unwrap_or(false);

        if !language_supported {
            return Ok(AnalysisResult::empty(line_count));
        }

        let tree = self.parser_manager.parse(&content, language)?;

        let depth = mode.as_str();
        let mut result = ElementExtractor::extract_with_depth(
            &tree,
            &content,
            language,
            depth,
            ast_recursion_limit,
        )?;

        result.line_count = line_count;

        self.cache
            .put(path.to_path_buf(), modified, mode, result.clone());

        Ok(result)
    }

    fn analyze_directory(
        &self,
        path: &Path,
        max_depth: u32,
        ast_recursion_limit: Option<usize>,
        traverser: &FileTraverser,
        mode: &AnalysisMode,
    ) -> Result<String, String> {
        let mode = *mode;

        let results = traverser.collect_directory_results(path, max_depth, |file_path| {
            self.analyze_file(file_path, &mode, ast_recursion_limit)
        })?;

        Ok(Formatter::format_directory_structure(
            path, &results, max_depth,
        ))
    }

    fn analyze_focused(
        &self,
        path: &Path,
        focus: &str,
        follow_depth: u32,
        max_depth: u32,
        ast_recursion_limit: Option<usize>,
        traverser: &FileTraverser,
    ) -> Result<String, String> {
        let files_to_analyze = if path.is_file() {
            vec![path.to_path_buf()]
        } else {
            traverser.collect_files_for_focused(path, max_depth)?
        };

        use rayon::prelude::*;
        let all_results: Result<Vec<_>, _> = files_to_analyze
            .par_iter()
            .map(|file_path| {
                self.analyze_file(file_path, &AnalysisMode::Semantic, ast_recursion_limit)
                    .map(|result| (file_path.clone(), result))
            })
            .collect();
        let all_results = all_results?;

        let graph = CallGraph::build_from_results(&all_results);

        let incoming_chains = if follow_depth > 0 {
            graph.find_incoming_chains(focus, follow_depth)
        } else {
            vec![]
        };

        let outgoing_chains = if follow_depth > 0 {
            graph.find_outgoing_chains(focus, follow_depth)
        } else {
            vec![]
        };

        let definitions = graph.definitions.get(focus).cloned().unwrap_or_default();

        let focus_data = FocusedAnalysisData {
            focus_symbol: focus,
            follow_depth,
            files_analyzed: &files_to_analyze,
            definitions: &definitions,
            incoming_chains: &incoming_chains,
            outgoing_chains: &outgoing_chains,
        };

        let mut output = Formatter::format_focused_output(&focus_data);

        if path.is_file() {
            let hint = "NOTE: Focus mode works best with directory paths. \
                        Use a parent directory in the path for cross-file analysis.\n\n";
            output = format!("{}{}", hint, output);
        }

        Ok(output)
    }
}

/// Simplified public API for the analyze tool
use std::sync::OnceLock;

static ANALYZER: OnceLock<CodeAnalyzer> = OnceLock::new();

fn get_analyzer() -> &'static CodeAnalyzer {
    ANALYZER.get_or_init(CodeAnalyzer::new)
}

pub fn analyze(
    path: &str,
    focus: Option<&str>,
    follow_depth: u32,
    max_depth: u32,
    ast_recursion_limit: Option<usize>,
    cwd: &str,
) -> String {
    let abs_path = if Path::new(path).is_absolute() {
        PathBuf::from(path)
    } else {
        PathBuf::from(cwd).join(path)
    };

    let analyzer = get_analyzer();
    let traverser = FileTraverser::new();

    if let Err(e) = traverser.validate_path(&abs_path) {
        return e;
    }

    let focus_owned = focus.map(|s| s.to_string());
    let mode = analyzer.determine_mode(&focus_owned, &abs_path);

    let mut output = match mode {
        AnalysisMode::Focused => {
            match analyzer.analyze_focused(
                &abs_path,
                focus.unwrap_or(""),
                follow_depth,
                max_depth,
                ast_recursion_limit,
                &traverser,
            ) {
                Ok(output) => output,
                Err(e) => return format!("Analysis error: {}", e),
            }
        }
        AnalysisMode::Semantic => {
            if abs_path.is_file() {
                match analyzer.analyze_file(&abs_path, &mode, ast_recursion_limit) {
                    Ok(result) => Formatter::format_analysis_result(&abs_path, &result, &mode),
                    Err(e) => return format!("Analysis error: {}", e),
                }
            } else {
                match analyzer.analyze_directory(
                    &abs_path,
                    max_depth,
                    ast_recursion_limit,
                    &traverser,
                    &mode,
                ) {
                    Ok(output) => output,
                    Err(e) => return format!("Analysis error: {}", e),
                }
            }
        }
        AnalysisMode::Structure => {
            if abs_path.is_file() {
                match analyzer.analyze_file(&abs_path, &mode, ast_recursion_limit) {
                    Ok(result) => Formatter::format_analysis_result(&abs_path, &result, &mode),
                    Err(e) => return format!("Analysis error: {}", e),
                }
            } else {
                match analyzer.analyze_directory(
                    &abs_path,
                    max_depth,
                    ast_recursion_limit,
                    &traverser,
                    &mode,
                ) {
                    Ok(output) => output,
                    Err(e) => return format!("Analysis error: {}", e),
                }
            }
        }
    };

    // If focus is specified with non-focused mode, filter results
    if let Some(focus_str) = focus
        && mode != AnalysisMode::Focused
    {
        output = Formatter::filter_by_focus(&output, focus_str);
    }

    output
}
