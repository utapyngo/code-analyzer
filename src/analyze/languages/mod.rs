// Copyright 2024 Block, Inc. (original code from https://github.com/block/goose)
// Copyright 2025 utapyngo (modifications)
// SPDX-License-Identifier: Apache-2.0

pub mod go;
pub mod java;
pub mod javascript;
pub mod kotlin;
pub mod python;
pub mod ruby;
pub mod rust;
pub mod swift;

/// Handler for extracting function names from special node kinds
type ExtractFunctionNameHandler = fn(&tree_sitter::Node, &str, &str) -> Option<String>;

/// Handler for finding method names from receiver nodes
type FindMethodForReceiverHandler = fn(&tree_sitter::Node, &str, Option<usize>) -> Option<String>;

/// Handler for finding the receiver type from a receiver node
type FindReceiverTypeHandler = fn(&tree_sitter::Node, &str) -> Option<String>;

/// Language configuration containing all language-specific information
#[derive(Copy, Clone)]
pub struct LanguageInfo {
    pub element_query: &'static str,
    pub call_query: &'static str,
    pub reference_query: &'static str,
    pub function_node_kinds: &'static [&'static str],
    pub function_name_kinds: &'static [&'static str],
    pub extract_function_name_handler: Option<ExtractFunctionNameHandler>,
    pub find_method_for_receiver_handler: Option<FindMethodForReceiverHandler>,
    pub find_receiver_type_handler: Option<FindReceiverTypeHandler>,
}

/// Get language configuration for a given language
pub fn get_language_info(language: &str) -> Option<LanguageInfo> {
    match language {
        "python" => Some(LanguageInfo {
            element_query: python::ELEMENT_QUERY,
            call_query: python::CALL_QUERY,
            reference_query: "",
            function_node_kinds: &["function_definition"],
            function_name_kinds: &["identifier", "field_identifier", "property_identifier"],
            extract_function_name_handler: None,
            find_method_for_receiver_handler: None,
            find_receiver_type_handler: None,
        }),
        "rust" => Some(LanguageInfo {
            element_query: rust::ELEMENT_QUERY,
            call_query: rust::CALL_QUERY,
            reference_query: rust::REFERENCE_QUERY,
            function_node_kinds: &["function_item", "impl_item"],
            function_name_kinds: &["identifier", "field_identifier", "property_identifier"],
            extract_function_name_handler: Some(rust::extract_function_name_for_kind),
            find_method_for_receiver_handler: Some(rust::find_method_for_receiver),
            find_receiver_type_handler: Some(rust::find_receiver_type),
        }),
        "javascript" | "typescript" => Some(LanguageInfo {
            element_query: javascript::ELEMENT_QUERY,
            call_query: javascript::CALL_QUERY,
            reference_query: "",
            function_node_kinds: &[
                "function_declaration",
                "method_definition",
                "arrow_function",
            ],
            function_name_kinds: &["identifier", "field_identifier", "property_identifier"],
            extract_function_name_handler: None,
            find_method_for_receiver_handler: None,
            find_receiver_type_handler: None,
        }),
        "go" => Some(LanguageInfo {
            element_query: go::ELEMENT_QUERY,
            call_query: go::CALL_QUERY,
            reference_query: go::REFERENCE_QUERY,
            function_node_kinds: &["function_declaration", "method_declaration"],
            function_name_kinds: &["identifier", "field_identifier", "property_identifier"],
            extract_function_name_handler: None,
            find_method_for_receiver_handler: Some(go::find_method_for_receiver),
            find_receiver_type_handler: None,
        }),
        "java" => Some(LanguageInfo {
            element_query: java::ELEMENT_QUERY,
            call_query: java::CALL_QUERY,
            reference_query: "",
            function_node_kinds: &["method_declaration", "constructor_declaration"],
            function_name_kinds: &["identifier", "field_identifier", "property_identifier"],
            extract_function_name_handler: None,
            find_method_for_receiver_handler: None,
            find_receiver_type_handler: None,
        }),
        "kotlin" => Some(LanguageInfo {
            element_query: kotlin::ELEMENT_QUERY,
            call_query: kotlin::CALL_QUERY,
            reference_query: "",
            function_node_kinds: &["function_declaration", "class_body"],
            function_name_kinds: &["identifier", "field_identifier", "property_identifier"],
            extract_function_name_handler: None,
            find_method_for_receiver_handler: None,
            find_receiver_type_handler: None,
        }),
        "swift" => Some(LanguageInfo {
            element_query: swift::ELEMENT_QUERY,
            call_query: swift::CALL_QUERY,
            reference_query: "",
            function_node_kinds: &[
                "function_declaration",
                "init_declaration",
                "deinit_declaration",
                "subscript_declaration",
            ],
            function_name_kinds: &["simple_identifier"],
            extract_function_name_handler: Some(swift::extract_function_name_for_kind),
            find_method_for_receiver_handler: None,
            find_receiver_type_handler: None,
        }),
        "ruby" => Some(LanguageInfo {
            element_query: ruby::ELEMENT_QUERY,
            call_query: ruby::CALL_QUERY,
            reference_query: ruby::REFERENCE_QUERY,
            function_node_kinds: &["method", "singleton_method"],
            function_name_kinds: &["identifier", "field_identifier", "property_identifier"],
            extract_function_name_handler: None,
            find_method_for_receiver_handler: Some(ruby::find_method_for_receiver),
            find_receiver_type_handler: None,
        }),
        _ => None,
    }
}
