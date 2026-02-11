// Copyright 2024 Block, Inc. (original code from https://github.com/block/goose)
// Copyright 2025 utapyngo (modifications)
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;

/// Analyze code structure and relationships using tree-sitter parsing.
///
/// Modes (auto-selected):
///
///   • Directory → Structure overview: file tree with LOC/function/class counts
///
///   • File → Semantic analysis: functions, classes, imports, call relationships
///
///   • With focus → Focused analysis: track a symbol across files with call chains
///
/// Supports: Python, Rust, JavaScript/TypeScript, Go, Java, Kotlin, Swift, Ruby
#[derive(Parser)]
#[command(
    name = "analyze",
    override_usage = "analyze [-f SYMBOL] [-d DEPTH] [-m DEPTH] [--ast-recursion-limit N] <PATH>"
)]
struct Args {
    /// File or directory path to analyze
    path: String,

    /// Symbol name to focus on
    #[arg(short, long)]
    focus: Option<String>,

    /// Call graph depth. 0=where defined, 1=direct callers/callees, 2+=transitive chains
    #[arg(short = 'd', long, default_value_t = 2)]
    follow_depth: u32,

    /// Directory recursion limit. 0=unlimited
    #[arg(short = 'm', long, default_value_t = 3)]
    max_depth: u32,

    /// Maximum depth for recursive AST traversal (prevents stack overflow in deeply nested code)
    #[arg(long)]
    ast_recursion_limit: Option<usize>,
}

fn main() {
    let args = match Args::try_parse() {
        Ok(args) => args,
        Err(e) => {
            if e.kind() == clap::error::ErrorKind::DisplayHelp
                || e.kind() == clap::error::ErrorKind::DisplayVersion
            {
                print!("{e}");
                std::process::exit(0);
            }
            eprintln!(
                "Usage: analyze [-f SYMBOL] [-d DEPTH] [-m DEPTH] [--ast-recursion-limit N] <PATH>"
            );
            eprintln!("Try 'analyze --help' for more information.");
            std::process::exit(1);
        }
    };
    let cwd = std::env::current_dir()
        .expect("Failed to get current directory")
        .to_string_lossy()
        .to_string();

    let result = code_analyze::analyze(
        &args.path,
        args.focus.as_deref(),
        args.follow_depth,
        args.max_depth,
        args.ast_recursion_limit,
        &cwd,
    );

    print!("{}", result);
}
