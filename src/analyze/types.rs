// Copyright 2024 Block, Inc. (original code from https://github.com/block/goose)
// Copyright 2025 utapyngo (modifications)
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub functions: Vec<FunctionInfo>,
    pub classes: Vec<ClassInfo>,
    pub imports: Vec<String>,
    pub calls: Vec<CallInfo>,
    pub references: Vec<ReferenceInfo>,
    pub function_count: usize,
    pub class_count: usize,
    pub line_count: usize,
    pub import_count: usize,
    pub main_line: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub line: usize,
    pub params: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassInfo {
    pub name: String,
    pub line: usize,
    pub methods: Vec<FunctionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallInfo {
    pub caller_name: Option<String>,
    pub callee_name: String,
    pub line: usize,
    pub column: usize,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceInfo {
    pub symbol: String,
    pub ref_type: ReferenceType,
    pub line: usize,
    pub context: String,
    pub associated_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReferenceType {
    Definition,
    MethodDefinition,
    Call,
    TypeInstantiation,
    FieldType,
    VariableType,
    ParameterType,
    Import,
}

#[derive(Debug, Clone)]
pub enum EntryType {
    File(AnalysisResult),
}

pub type ElementQueryResult = (Vec<FunctionInfo>, Vec<ClassInfo>, Vec<String>);

#[derive(Debug, Clone)]
pub struct CallChain {
    pub path: Vec<(PathBuf, usize, String, String)>, // (file, line, from, to)
}

pub struct FocusedAnalysisData<'a> {
    pub focus_symbol: &'a str,
    pub follow_depth: u32,
    pub files_analyzed: &'a [PathBuf],
    pub definitions: &'a [(PathBuf, usize)],
    pub incoming_chains: &'a [CallChain],
    pub outgoing_chains: &'a [CallChain],
}

/// Analysis modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnalysisMode {
    Structure,
    Semantic,
    Focused,
}

impl AnalysisMode {
    pub fn as_str(&self) -> &str {
        match self {
            AnalysisMode::Structure => "structure",
            AnalysisMode::Semantic => "semantic",
            AnalysisMode::Focused => "focused",
        }
    }
}

impl AnalysisResult {
    pub fn empty(line_count: usize) -> Self {
        Self {
            functions: vec![],
            classes: vec![],
            imports: vec![],
            calls: vec![],
            references: vec![],
            function_count: 0,
            class_count: 0,
            line_count,
            import_count: 0,
            main_line: None,
        }
    }
}
