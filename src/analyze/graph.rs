// Copyright 2024 Block, Inc. (original code from https://github.com/block/goose)
// Copyright 2025 utapyngo (modifications)
// SPDX-License-Identifier: Apache-2.0

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;

use super::types::{AnalysisResult, CallChain, ReferenceType};

/// Sentinel value used to represent type references as callers in the call graph
const REFERENCE_CALLER: &str = "<reference>";

#[derive(Debug, Clone, Default)]
pub struct CallGraph {
    callers: HashMap<String, Vec<(PathBuf, usize, String)>>,
    callees: HashMap<String, Vec<(PathBuf, usize, String)>>,
    pub definitions: HashMap<String, Vec<(PathBuf, usize)>>,
}

impl CallGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build_from_results(results: &[(PathBuf, AnalysisResult)]) -> Self {
        let mut graph = Self::new();

        for (file_path, result) in results {
            for func in &result.functions {
                graph
                    .definitions
                    .entry(func.name.clone())
                    .or_default()
                    .push((file_path.clone(), func.line));
            }

            for class in &result.classes {
                graph
                    .definitions
                    .entry(class.name.clone())
                    .or_default()
                    .push((file_path.clone(), class.line));
            }

            for call in &result.calls {
                let caller = call
                    .caller_name
                    .clone()
                    .unwrap_or_else(|| "<module>".to_string());

                graph
                    .callers
                    .entry(call.callee_name.clone())
                    .or_default()
                    .push((file_path.clone(), call.line, caller.clone()));

                if caller != "<module>" {
                    graph.callees.entry(caller).or_default().push((
                        file_path.clone(),
                        call.line,
                        call.callee_name.clone(),
                    ));
                }
            }

            for reference in &result.references {
                match &reference.ref_type {
                    ReferenceType::MethodDefinition => {
                        if let Some(type_name) = &reference.associated_type {
                            graph.callees.entry(type_name.clone()).or_default().push((
                                file_path.clone(),
                                reference.line,
                                reference.symbol.clone(),
                            ));
                        }
                    }
                    ReferenceType::TypeInstantiation
                    | ReferenceType::FieldType
                    | ReferenceType::VariableType
                    | ReferenceType::ParameterType => {
                        graph
                            .callers
                            .entry(reference.symbol.clone())
                            .or_default()
                            .push((
                                file_path.clone(),
                                reference.line,
                                REFERENCE_CALLER.to_string(),
                            ));
                    }
                    ReferenceType::Definition | ReferenceType::Call | ReferenceType::Import => {}
                }
            }
        }

        graph
    }

    pub fn find_incoming_chains(&self, symbol: &str, max_depth: u32) -> Vec<CallChain> {
        if max_depth == 0 {
            return vec![];
        }

        let mut chains = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        if let Some(direct_callers) = self.callers.get(symbol) {
            for (file, line, caller) in direct_callers {
                let initial_path = vec![(file.clone(), *line, caller.clone(), symbol.to_string())];

                if max_depth == 1 {
                    chains.push(CallChain { path: initial_path });
                } else {
                    queue.push_back((caller.clone(), initial_path, 1));
                }
            }
        }

        while let Some((current_symbol, path, depth)) = queue.pop_front() {
            if depth >= max_depth {
                chains.push(CallChain { path });
                continue;
            }

            if visited.contains(&current_symbol) {
                chains.push(CallChain { path });
                continue;
            }
            visited.insert(current_symbol.clone());

            if let Some(callers) = self.callers.get(&current_symbol) {
                for (file, line, caller) in callers {
                    let mut new_path =
                        vec![(file.clone(), *line, caller.clone(), current_symbol.clone())];
                    new_path.extend(path.clone());

                    if depth + 1 >= max_depth {
                        chains.push(CallChain { path: new_path });
                    } else {
                        queue.push_back((caller.clone(), new_path, depth + 1));
                    }
                }
            } else {
                chains.push(CallChain { path });
            }
        }

        chains
    }

    pub fn find_outgoing_chains(&self, symbol: &str, max_depth: u32) -> Vec<CallChain> {
        if max_depth == 0 {
            return vec![];
        }

        let mut chains = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        if let Some(direct_callees) = self.callees.get(symbol) {
            for (file, line, callee) in direct_callees {
                let initial_path = vec![(file.clone(), *line, symbol.to_string(), callee.clone())];

                if max_depth == 1 {
                    chains.push(CallChain { path: initial_path });
                } else {
                    queue.push_back((callee.clone(), initial_path, 1));
                }
            }
        }

        while let Some((current_symbol, path, depth)) = queue.pop_front() {
            if depth >= max_depth {
                chains.push(CallChain { path });
                continue;
            }

            if visited.contains(&current_symbol) {
                chains.push(CallChain { path });
                continue;
            }
            visited.insert(current_symbol.clone());

            if let Some(callees) = self.callees.get(&current_symbol) {
                for (file, line, callee) in callees {
                    let mut new_path = path.clone();
                    new_path.push((file.clone(), *line, current_symbol.clone(), callee.clone()));

                    if depth + 1 >= max_depth {
                        chains.push(CallChain { path: new_path });
                    } else {
                        queue.push_back((callee.clone(), new_path, depth + 1));
                    }
                }
            } else {
                chains.push(CallChain { path });
            }
        }

        chains
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyze::types::{AnalysisResult, CallInfo, ClassInfo, FunctionInfo};

    fn make_result(funcs: &[&str], calls: &[(&str, &str)]) -> AnalysisResult {
        let functions: Vec<FunctionInfo> = funcs
            .iter()
            .enumerate()
            .map(|(i, name)| FunctionInfo {
                name: name.to_string(),
                line: i + 1,
                params: vec![],
            })
            .collect();

        let call_infos: Vec<CallInfo> = calls
            .iter()
            .enumerate()
            .map(|(i, (caller, callee))| CallInfo {
                caller_name: Some(caller.to_string()),
                callee_name: callee.to_string(),
                line: i + 10,
                column: 0,
                context: String::new(),
            })
            .collect();

        AnalysisResult {
            function_count: functions.len(),
            class_count: 0,
            import_count: 0,
            line_count: 50,
            functions,
            classes: vec![],
            imports: vec![],
            calls: call_infos,
            references: vec![],
            main_line: None,
        }
    }

    #[test]
    fn empty_graph() {
        let graph = CallGraph::new();
        assert!(graph.definitions.is_empty());
        assert!(graph.find_incoming_chains("x", 2).is_empty());
        assert!(graph.find_outgoing_chains("x", 2).is_empty());
    }

    #[test]
    fn build_from_results_records_definitions() {
        let results = vec![(PathBuf::from("test.rs"), make_result(&["foo", "bar"], &[]))];
        let graph = CallGraph::build_from_results(&results);
        assert!(graph.definitions.contains_key("foo"));
        assert!(graph.definitions.contains_key("bar"));
        assert_eq!(graph.definitions["foo"].len(), 1);
    }

    #[test]
    fn build_from_results_records_calls() {
        let results = vec![(
            PathBuf::from("test.rs"),
            make_result(&["main", "helper"], &[("main", "helper")]),
        )];
        let graph = CallGraph::build_from_results(&results);

        // main calls helper → helper has incoming from main
        let incoming = graph.find_incoming_chains("helper", 1);
        assert!(
            !incoming.is_empty(),
            "expected incoming chains for 'helper'"
        );

        // main calls helper → main has outgoing to helper
        let outgoing = graph.find_outgoing_chains("main", 1);
        assert!(!outgoing.is_empty(), "expected outgoing chains from 'main'");
    }

    #[test]
    fn find_incoming_zero_depth() {
        let results = vec![(
            PathBuf::from("test.rs"),
            make_result(&["a", "b"], &[("a", "b")]),
        )];
        let graph = CallGraph::build_from_results(&results);
        assert!(graph.find_incoming_chains("b", 0).is_empty());
    }

    #[test]
    fn find_outgoing_zero_depth() {
        let results = vec![(
            PathBuf::from("test.rs"),
            make_result(&["a", "b"], &[("a", "b")]),
        )];
        let graph = CallGraph::build_from_results(&results);
        assert!(graph.find_outgoing_chains("a", 0).is_empty());
    }

    #[test]
    fn transitive_chains() {
        // a -> b -> c
        let results = vec![(
            PathBuf::from("test.rs"),
            make_result(&["a", "b", "c"], &[("a", "b"), ("b", "c")]),
        )];
        let graph = CallGraph::build_from_results(&results);

        // At depth 2, c should see chain from a through b
        let incoming = graph.find_incoming_chains("c", 2);
        assert!(
            !incoming.is_empty(),
            "expected transitive incoming chains for 'c'"
        );
    }

    #[test]
    fn classes_added_to_definitions() {
        let mut result = make_result(&[], &[]);
        result.classes.push(ClassInfo {
            name: "MyStruct".into(),
            line: 5,
            methods: vec![],
        });
        result.class_count = 1;
        let results = vec![(PathBuf::from("test.rs"), result)];
        let graph = CallGraph::build_from_results(&results);
        assert!(graph.definitions.contains_key("MyStruct"));
    }
}
