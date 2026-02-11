// Copyright 2024 Block, Inc. (original code from https://github.com/block/goose)
// Copyright 2025 utapyngo (modifications)
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tree_sitter::{Language, Parser, StreamingIterator, Tree};

use super::lock_or_recover;
use super::types::{
    AnalysisResult, CallInfo, ClassInfo, ElementQueryResult, FunctionInfo, ReferenceInfo,
    ReferenceType,
};

#[derive(Clone)]
pub struct ParserManager {
    parsers: Arc<Mutex<HashMap<String, Arc<Mutex<Parser>>>>>,
}

impl ParserManager {
    pub fn new() -> Self {
        Self {
            parsers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_or_create_parser(&self, language: &str) -> Result<Arc<Mutex<Parser>>, String> {
        let mut cache = lock_or_recover(&self.parsers, |c| c.clear());

        if let Some(parser) = cache.get(language) {
            return Ok(Arc::clone(parser));
        }

        let mut parser = Parser::new();
        let language_config: Language = match language {
            "python" => tree_sitter_python::LANGUAGE.into(),
            "rust" => tree_sitter_rust::LANGUAGE.into(),
            "javascript" | "typescript" => tree_sitter_javascript::LANGUAGE.into(),
            "go" => tree_sitter_go::LANGUAGE.into(),
            "java" => tree_sitter_java::LANGUAGE.into(),
            "kotlin" => tree_sitter_kotlin_ng::LANGUAGE.into(),
            "swift" => tree_sitter_swift::LANGUAGE.into(),
            "ruby" => tree_sitter_ruby::LANGUAGE.into(),
            _ => {
                return Err(format!("Unsupported language: {}", language));
            }
        };

        parser
            .set_language(&language_config)
            .map_err(|e| format!("Failed to set language for {}: {}", language, e))?;

        let parser_arc = Arc::new(Mutex::new(parser));
        cache.insert(language.to_string(), Arc::clone(&parser_arc));
        Ok(parser_arc)
    }

    pub fn parse(&self, content: &str, language: &str) -> Result<Tree, String> {
        let parser_arc = self.get_or_create_parser(language)?;
        let mut parser = lock_or_recover(&parser_arc, |_| {});

        parser
            .parse(content, None)
            .ok_or_else(|| format!("Failed to parse file as {}", language))
    }
}

impl Default for ParserManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ElementExtractor;

impl ElementExtractor {
    fn extract_text_from_child(
        node: &tree_sitter::Node,
        source: &str,
        kinds: &[&str],
    ) -> Option<String> {
        (0..node.child_count() as u32)
            .filter_map(|i| node.child(i))
            .find(|child| kinds.contains(&child.kind()))
            .and_then(|child| source.get(child.byte_range()).map(|s| s.to_string()))
    }

    pub fn extract_with_depth(
        tree: &Tree,
        source: &str,
        language: &str,
        depth: &str,
        ast_recursion_limit: Option<usize>,
    ) -> Result<AnalysisResult, String> {
        use super::languages;

        let mut result = Self::extract_elements(tree, source, language)?;

        if depth == "structure" {
            result.functions.clear();
            result.classes.clear();
            result.imports.clear();
        } else if depth == "semantic" {
            let calls = Self::extract_calls(tree, source, language)?;
            result.calls = calls;

            for call in &result.calls {
                result.references.push(ReferenceInfo {
                    symbol: call.callee_name.clone(),
                    ref_type: ReferenceType::Call,
                    line: call.line,
                    context: call.context.clone(),
                    associated_type: None,
                });
            }

            if let Some(info) = languages::get_language_info(language)
                && !info.reference_query.is_empty()
            {
                let references =
                    Self::extract_references(tree, source, language, ast_recursion_limit)?;
                result.references.extend(references);
            }
        }

        Ok(result)
    }

    pub fn extract_elements(
        tree: &Tree,
        source: &str,
        language: &str,
    ) -> Result<AnalysisResult, String> {
        use super::languages;

        let info = match languages::get_language_info(language) {
            Some(info) if !info.element_query.is_empty() => info,
            _ => return Ok(Self::empty_analysis_result()),
        };

        let query_str = info.element_query;

        let (functions, classes, imports) = Self::process_element_query(tree, source, query_str)?;

        let main_line = functions.iter().find(|f| f.name == "main").map(|f| f.line);

        Ok(AnalysisResult {
            function_count: functions.len(),
            class_count: classes.len(),
            import_count: imports.len(),
            functions,
            classes,
            imports,
            calls: vec![],
            references: vec![],
            line_count: 0,
            main_line,
        })
    }

    fn process_element_query(
        tree: &Tree,
        source: &str,
        query_str: &str,
    ) -> Result<ElementQueryResult, String> {
        use tree_sitter::{Query, QueryCursor};

        let mut functions = Vec::new();
        let mut classes = Vec::new();
        let mut imports = Vec::new();

        let query = Query::new(&tree.language(), query_str)
            .map_err(|e| format!("Failed to create query: {}", e))?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

        while let Some(match_) = matches.next() {
            for capture in match_.captures {
                let node = capture.node;
                let Some(text) = source.get(node.byte_range()) else {
                    continue;
                };
                let line = source
                    .get(..node.start_byte())
                    .map(|s: &str| s.lines().count() + 1)
                    .unwrap_or(1);

                match query.capture_names()[capture.index as usize] {
                    "func" | "const" => {
                        functions.push(FunctionInfo {
                            name: text.to_string(),
                            line,
                            params: vec![],
                        });
                    }
                    "class" | "struct" => {
                        classes.push(ClassInfo {
                            name: text.to_string(),
                            line,
                            methods: vec![],
                        });
                    }
                    "import" => {
                        imports.push(text.to_string());
                    }
                    _ => {}
                }
            }
        }

        Ok((functions, classes, imports))
    }

    fn extract_calls(tree: &Tree, source: &str, language: &str) -> Result<Vec<CallInfo>, String> {
        use super::languages;
        use tree_sitter::{Query, QueryCursor};

        let mut calls = Vec::new();

        let info = match languages::get_language_info(language) {
            Some(info) if !info.call_query.is_empty() => info,
            _ => return Ok(calls),
        };

        let query_str = info.call_query;

        let query = Query::new(&tree.language(), query_str)
            .map_err(|e| format!("Failed to create call query: {}", e))?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

        while let Some(match_) = matches.next() {
            for capture in match_.captures {
                let node = capture.node;
                let Some(text) = source.get(node.byte_range()) else {
                    continue;
                };
                let start_pos = node.start_position();

                let line_start = source
                    .get(..node.start_byte())
                    .and_then(|s: &str| s.rfind('\n'))
                    .map(|i| i + 1)
                    .unwrap_or(0);
                let line_end = source
                    .get(node.end_byte()..)
                    .and_then(|s: &str| s.find('\n'))
                    .map(|i| node.end_byte() + i)
                    .unwrap_or(source.len());
                let context = source
                    .get(line_start..line_end)
                    .map(|s: &str| s.trim().to_string())
                    .unwrap_or_default();

                let caller_name = Self::find_containing_function(&node, source, language);

                match query.capture_names()[capture.index as usize] {
                    "function.call"
                    | "method.call"
                    | "scoped.call"
                    | "macro.call"
                    | "constructor.call"
                    | "identifier.reference" => {
                        calls.push(CallInfo {
                            caller_name,
                            callee_name: text.to_string(),
                            line: start_pos.row + 1,
                            column: start_pos.column,
                            context,
                        });
                    }
                    _ => {}
                }
            }
        }

        Ok(calls)
    }

    fn extract_references(
        tree: &Tree,
        source: &str,
        language: &str,
        ast_recursion_limit: Option<usize>,
    ) -> Result<Vec<ReferenceInfo>, String> {
        use super::languages;
        use tree_sitter::{Query, QueryCursor};

        let mut references = Vec::new();

        let info = match languages::get_language_info(language) {
            Some(info) if !info.reference_query.is_empty() => info,
            _ => return Ok(references),
        };

        let query_str = info.reference_query;

        let query = Query::new(&tree.language(), query_str)
            .map_err(|e| format!("Failed to create reference query: {}", e))?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

        while let Some(match_) = matches.next() {
            for capture in match_.captures {
                let node = capture.node;
                let Some(text) = source.get(node.byte_range()) else {
                    continue;
                };
                let start_pos = node.start_position();

                let line_start = source
                    .get(..node.start_byte())
                    .and_then(|s: &str| s.rfind('\n'))
                    .map(|i| i + 1)
                    .unwrap_or(0);
                let line_end = source
                    .get(node.end_byte()..)
                    .and_then(|s: &str| s.find('\n'))
                    .map(|i| node.end_byte() + i)
                    .unwrap_or(source.len());
                let context = source
                    .get(line_start..line_end)
                    .map(|s: &str| s.trim().to_string())
                    .unwrap_or_default();

                let capture_name = query.capture_names()[capture.index as usize];

                let (ref_type, symbol, associated_type) = match capture_name {
                    "method.receiver" => {
                        let method_name = Self::find_method_name_for_receiver(
                            &node,
                            source,
                            language,
                            ast_recursion_limit,
                        );
                        if let Some(method_name) = method_name {
                            let type_name = Self::find_receiver_type(&node, source, language)
                                .or_else(|| Some(text.to_string()));

                            if let Some(type_name) = type_name {
                                (
                                    ReferenceType::MethodDefinition,
                                    method_name,
                                    Some(type_name),
                                )
                            } else {
                                continue;
                            }
                        } else {
                            continue;
                        }
                    }
                    "struct.literal" => (ReferenceType::TypeInstantiation, text.to_string(), None),
                    "field.type" => (ReferenceType::FieldType, text.to_string(), None),
                    "param.type" => (ReferenceType::ParameterType, text.to_string(), None),
                    "var.type" | "shortvar.type" => {
                        (ReferenceType::VariableType, text.to_string(), None)
                    }
                    "type.assertion" | "type.conversion" => {
                        (ReferenceType::Call, text.to_string(), None)
                    }
                    _ => continue,
                };

                references.push(ReferenceInfo {
                    symbol,
                    ref_type,
                    line: start_pos.row + 1,
                    context,
                    associated_type,
                });
            }
        }

        Ok(references)
    }

    fn find_method_name_for_receiver(
        receiver_node: &tree_sitter::Node,
        source: &str,
        language: &str,
        ast_recursion_limit: Option<usize>,
    ) -> Option<String> {
        use super::languages;

        languages::get_language_info(language)
            .and_then(|info| info.find_method_for_receiver_handler)
            .and_then(|handler| handler(receiver_node, source, ast_recursion_limit))
    }

    fn find_receiver_type(
        receiver_node: &tree_sitter::Node,
        source: &str,
        language: &str,
    ) -> Option<String> {
        use super::languages;

        languages::get_language_info(language)
            .and_then(|info| info.find_receiver_type_handler)
            .and_then(|handler| handler(receiver_node, source))
    }

    fn find_containing_function(
        node: &tree_sitter::Node,
        source: &str,
        language: &str,
    ) -> Option<String> {
        use super::languages;

        let info = languages::get_language_info(language)?;

        let mut current = *node;

        while let Some(parent) = current.parent() {
            let kind = parent.kind();

            if info.function_node_kinds.contains(&kind) {
                if let Some(handler) = info.extract_function_name_handler
                    && let Some(name) = handler(&parent, source, kind)
                {
                    return Some(name);
                }

                if let Some(name) =
                    Self::extract_text_from_child(&parent, source, info.function_name_kinds)
                {
                    return Some(name);
                }
            }

            current = parent;
        }

        None
    }

    fn empty_analysis_result() -> AnalysisResult {
        AnalysisResult {
            functions: vec![],
            classes: vec![],
            imports: vec![],
            calls: vec![],
            references: vec![],
            function_count: 0,
            class_count: 0,
            line_count: 0,
            import_count: 0,
            main_line: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_manager_creates_for_supported_languages() {
        let pm = ParserManager::new();
        for lang in &[
            "python",
            "rust",
            "javascript",
            "typescript",
            "go",
            "java",
            "kotlin",
            "swift",
            "ruby",
        ] {
            assert!(pm.get_or_create_parser(lang).is_ok(), "failed for {}", lang);
        }
    }

    #[test]
    fn parser_manager_rejects_unsupported() {
        let pm = ParserManager::new();
        assert!(pm.get_or_create_parser("brainfuck").is_err());
    }

    #[test]
    fn parser_manager_caches_parser() {
        let pm = ParserManager::new();
        let p1 = pm.get_or_create_parser("rust").unwrap();
        let p2 = pm.get_or_create_parser("rust").unwrap();
        assert!(std::sync::Arc::ptr_eq(&p1, &p2));
    }

    #[test]
    fn parse_rust_code() {
        let pm = ParserManager::new();
        let tree = pm.parse("fn main() {}", "rust");
        assert!(tree.is_ok());
    }

    #[test]
    fn parse_python_code() {
        let pm = ParserManager::new();
        let tree = pm.parse("def hello():\n    pass\n", "python");
        assert!(tree.is_ok());
    }

    #[test]
    fn parse_go_code() {
        let pm = ParserManager::new();
        let tree = pm.parse("package main\nfunc main() {}\n", "go");
        assert!(tree.is_ok());
    }

    #[test]
    fn extract_elements_rust() {
        let pm = ParserManager::new();
        let code = "use std::io;\n\nstruct Foo;\n\nfn bar() {}\nfn main() {}\n";
        let tree = pm.parse(code, "rust").unwrap();
        let result = ElementExtractor::extract_elements(&tree, code, "rust").unwrap();
        assert!(result.functions.iter().any(|f| f.name == "bar"));
        assert!(result.functions.iter().any(|f| f.name == "main"));
        assert!(result.classes.iter().any(|c| c.name == "Foo"));
        assert!(result.main_line.is_some());
    }

    #[test]
    fn extract_elements_python() {
        let pm = ParserManager::new();
        let code = "import os\n\nclass Foo:\n    pass\n\ndef bar():\n    pass\n";
        let tree = pm.parse(code, "python").unwrap();
        let result = ElementExtractor::extract_elements(&tree, code, "python").unwrap();
        assert!(result.functions.iter().any(|f| f.name == "bar"));
        assert!(result.classes.iter().any(|c| c.name == "Foo"));
    }

    #[test]
    fn extract_with_depth_structure() {
        let pm = ParserManager::new();
        let code = "fn foo() {}\nfn main() {}\n";
        let tree = pm.parse(code, "rust").unwrap();
        let result =
            ElementExtractor::extract_with_depth(&tree, code, "rust", "structure", None).unwrap();
        // Structure mode clears functions/classes/imports
        assert!(result.functions.is_empty());
        assert!(result.classes.is_empty());
        assert!(result.imports.is_empty());
    }

    #[test]
    fn extract_with_depth_semantic() {
        let pm = ParserManager::new();
        let code = "fn foo() { bar(); }\nfn bar() {}\n";
        let tree = pm.parse(code, "rust").unwrap();
        let result =
            ElementExtractor::extract_with_depth(&tree, code, "rust", "semantic", None).unwrap();
        assert!(!result.functions.is_empty());
        // Should have calls extracted
        assert!(!result.calls.is_empty() || !result.references.is_empty());
    }

    #[test]
    fn extract_elements_unsupported_returns_empty() {
        let pm = ParserManager::new();
        // Parse as JS but ask for elements of "haskell" which has no language info
        let code = "function f() {}";
        let tree = pm.parse(code, "javascript").unwrap();
        let result = ElementExtractor::extract_elements(&tree, code, "haskell").unwrap();
        assert!(result.functions.is_empty());
    }

    #[test]
    fn parser_manager_default_works() {
        let _pm = ParserManager::default();
    }
}
