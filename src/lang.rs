// Copyright 2024 Block, Inc. (original code from https://github.com/block/goose)
// Copyright 2025 utapyngo (modifications)
// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

/// Get the markdown language identifier for a file extension
pub fn get_language_identifier(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("rs") => "rust",
        Some("hs") => "haskell",
        Some("rkt") | Some("scm") => "scheme",
        Some("py") => "python",
        Some("js") => "javascript",
        Some("ts") => "typescript",
        Some("json") => "json",
        Some("toml") => "toml",
        Some("yaml") | Some("yml") => "yaml",
        Some("sh") => "bash",
        Some("ps1") => "powershell",
        Some("bat") | Some("cmd") => "batch",
        Some("vbs") => "vbscript",
        Some("go") => "go",
        Some("md") => "markdown",
        Some("html") => "html",
        Some("css") => "css",
        Some("sql") => "sql",
        Some("java") => "java",
        Some("cpp") | Some("cc") | Some("cxx") => "cpp",
        Some("c") => "c",
        Some("h") | Some("hpp") => "cpp",
        Some("rb") => "ruby",
        Some("php") => "php",
        Some("swift") => "swift",
        Some("kt") | Some("kts") => "kotlin",
        Some("scala") => "scala",
        Some("r") => "r",
        Some("m") => "matlab",
        Some("pl") => "perl",
        Some("dockerfile") => "dockerfile",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_rust() {
        assert_eq!(get_language_identifier(Path::new("main.rs")), "rust");
    }

    #[test]
    fn detect_python() {
        assert_eq!(get_language_identifier(Path::new("script.py")), "python");
    }

    #[test]
    fn detect_javascript() {
        assert_eq!(get_language_identifier(Path::new("app.js")), "javascript");
    }

    #[test]
    fn detect_typescript() {
        assert_eq!(get_language_identifier(Path::new("app.ts")), "typescript");
    }

    #[test]
    fn detect_go() {
        assert_eq!(get_language_identifier(Path::new("main.go")), "go");
    }

    #[test]
    fn detect_java() {
        assert_eq!(get_language_identifier(Path::new("App.java")), "java");
    }

    #[test]
    fn detect_kotlin() {
        assert_eq!(get_language_identifier(Path::new("App.kt")), "kotlin");
        assert_eq!(get_language_identifier(Path::new("build.kts")), "kotlin");
    }

    #[test]
    fn detect_swift() {
        assert_eq!(get_language_identifier(Path::new("App.swift")), "swift");
    }

    #[test]
    fn detect_ruby() {
        assert_eq!(get_language_identifier(Path::new("app.rb")), "ruby");
    }

    #[test]
    fn detect_c_cpp() {
        assert_eq!(get_language_identifier(Path::new("main.c")), "c");
        assert_eq!(get_language_identifier(Path::new("main.cpp")), "cpp");
        assert_eq!(get_language_identifier(Path::new("main.cc")), "cpp");
        assert_eq!(get_language_identifier(Path::new("main.h")), "cpp");
    }

    #[test]
    fn detect_misc_languages() {
        assert_eq!(get_language_identifier(Path::new("a.json")), "json");
        assert_eq!(get_language_identifier(Path::new("a.toml")), "toml");
        assert_eq!(get_language_identifier(Path::new("a.yaml")), "yaml");
        assert_eq!(get_language_identifier(Path::new("a.yml")), "yaml");
        assert_eq!(get_language_identifier(Path::new("a.sh")), "bash");
        assert_eq!(get_language_identifier(Path::new("a.md")), "markdown");
        assert_eq!(get_language_identifier(Path::new("a.html")), "html");
        assert_eq!(get_language_identifier(Path::new("a.css")), "css");
        assert_eq!(get_language_identifier(Path::new("a.sql")), "sql");
    }

    #[test]
    fn unknown_extension_returns_empty() {
        assert_eq!(get_language_identifier(Path::new("file.xyz")), "");
    }

    #[test]
    fn no_extension_returns_empty() {
        assert_eq!(get_language_identifier(Path::new("Makefile")), "");
    }
}
