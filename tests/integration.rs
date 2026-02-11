use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn fixture(name: &str) -> String {
    fixtures_dir().join(name).to_string_lossy().to_string()
}

fn cwd() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .to_string_lossy()
        .to_string()
}

// ── File analysis ──────────────────────────────────────────────────────

#[test]
fn analyze_rust_file() {
    let out = code_analyze::analyze(&fixture("sample.rs"), None, 2, 3, None, &cwd());
    assert!(out.contains("FILE:"), "expected FILE: header:\n{out}");
    assert!(
        out.contains("sample.rs"),
        "expected filename in output:\n{out}"
    );
    assert!(
        out.contains("F:"),
        "expected F: (functions) section:\n{out}"
    );
    assert!(out.contains("main"), "expected 'main' function:\n{out}");
    assert!(out.contains("helper"), "expected 'helper' function:\n{out}");
}

#[test]
fn analyze_python_file() {
    let out = code_analyze::analyze(&fixture("sample.py"), None, 2, 3, None, &cwd());
    assert!(out.contains("FILE:"), "expected FILE: header:\n{out}");
    assert!(out.contains("F:"), "expected F: section:\n{out}");
    assert!(
        out.contains("process"),
        "expected 'process' function:\n{out}"
    );
    assert!(out.contains("main"), "expected 'main' function:\n{out}");
    assert!(out.contains("C:"), "expected C: (classes) section:\n{out}");
    assert!(
        out.contains("FileReader"),
        "expected 'FileReader' class:\n{out}"
    );
    assert!(out.contains("I:"), "expected I: (imports) section:\n{out}");
}

#[test]
fn analyze_javascript_file() {
    let out = code_analyze::analyze(&fixture("sample.js"), None, 2, 3, None, &cwd());
    assert!(out.contains("FILE:"), "expected FILE: header:\n{out}");
    assert!(out.contains("F:"), "expected F: section:\n{out}");
    assert!(out.contains("greet"), "expected 'greet' function:\n{out}");
    assert!(out.contains("main"), "expected 'main' function:\n{out}");
    assert!(out.contains("C:"), "expected C: section:\n{out}");
    assert!(out.contains("Logger"), "expected 'Logger' class:\n{out}");
}

#[test]
fn analyze_go_file() {
    let out = code_analyze::analyze(&fixture("sample.go"), None, 2, 3, None, &cwd());
    assert!(out.contains("FILE:"), "expected FILE: header:\n{out}");
    assert!(out.contains("F:"), "expected F: section:\n{out}");
    assert!(out.contains("main"), "expected 'main' function:\n{out}");
    assert!(out.contains("helper"), "expected 'helper' function:\n{out}");
    assert!(out.contains("C:"), "expected C: (structs) section:\n{out}");
    assert!(out.contains("Greeter"), "expected 'Greeter' struct:\n{out}");
}

// ── Directory analysis ─────────────────────────────────────────────────

#[test]
fn analyze_fixtures_directory() {
    let out = code_analyze::analyze(&fixtures_dir().to_string_lossy(), None, 2, 3, None, &cwd());
    assert!(out.contains("SUMMARY:"), "expected SUMMARY: header:\n{out}");
    assert!(
        out.contains("sample.rs"),
        "expected sample.rs in tree:\n{out}"
    );
    assert!(
        out.contains("sample.py"),
        "expected sample.py in tree:\n{out}"
    );
    assert!(
        out.contains("sample.js"),
        "expected sample.js in tree:\n{out}"
    );
    assert!(
        out.contains("sample.go"),
        "expected sample.go in tree:\n{out}"
    );
    // Should contain file count
    assert!(
        out.contains("4 files"),
        "expected '4 files' in summary:\n{out}"
    );
}

#[test]
fn analyze_directory_has_language_stats() {
    let out = code_analyze::analyze(&fixtures_dir().to_string_lossy(), None, 2, 3, None, &cwd());
    assert!(
        out.contains("Languages:"),
        "expected Languages: line:\n{out}"
    );
}

#[test]
fn analyze_directory_tree_format() {
    let out = code_analyze::analyze(&fixtures_dir().to_string_lossy(), None, 2, 3, None, &cwd());
    // Should have LOC annotations like [NL]
    assert!(out.contains("L"), "expected line count annotations:\n{out}");
    // Should have the column header
    assert!(
        out.contains("PATH [LOC, FUNCTIONS, CLASSES]"),
        "expected column header:\n{out}"
    );
}

// ── Focused analysis ───────────────────────────────────────────────────

#[test]
fn focused_analysis_finds_symbol() {
    let out = code_analyze::analyze(
        &fixtures_dir().to_string_lossy(),
        Some("main"),
        2,
        3,
        None,
        &cwd(),
    );
    assert!(
        out.contains("FOCUSED ANALYSIS: main"),
        "expected focused header:\n{out}"
    );
    assert!(
        out.contains("DEFINITIONS:"),
        "expected DEFINITIONS section:\n{out}"
    );
    assert!(
        out.contains("STATISTICS:"),
        "expected STATISTICS section:\n{out}"
    );
}

#[test]
fn focused_analysis_nonexistent_symbol() {
    let out = code_analyze::analyze(
        &fixtures_dir().to_string_lossy(),
        Some("nonexistent_symbol_xyz"),
        2,
        3,
        None,
        &cwd(),
    );
    assert!(
        out.contains("not found"),
        "expected 'not found' for missing symbol:\n{out}"
    );
}

#[test]
fn focused_analysis_on_single_file_has_hint() {
    let out = code_analyze::analyze(&fixture("sample.rs"), Some("main"), 2, 3, None, &cwd());
    assert!(
        out.contains("NOTE:"),
        "expected NOTE about directory path:\n{out}"
    );
}

// ── Edge cases ─────────────────────────────────────────────────────────

#[test]
fn nonexistent_path() {
    let out = code_analyze::analyze("/tmp/does_not_exist_xyz_123", None, 2, 3, None, &cwd());
    assert!(
        out.contains("does not exist"),
        "expected error for nonexistent path:\n{out}"
    );
}

#[test]
fn analyze_empty_directory() {
    let dir = tempfile::tempdir().unwrap();
    let out = code_analyze::analyze(&dir.path().to_string_lossy(), None, 2, 3, None, &cwd());
    assert!(
        out.contains("SUMMARY:"),
        "expected SUMMARY for empty dir:\n{out}"
    );
    assert!(out.contains("0 files"), "expected 0 files:\n{out}");
}

#[test]
fn analyze_binary_file_no_crash() {
    let dir = tempfile::tempdir().unwrap();
    let bin_path = dir.path().join("data.rs");
    std::fs::write(&bin_path, &[0u8, 1, 2, 0xFF, 0xFE, 0xFD]).unwrap();
    // Should not panic; may return empty result
    let out = code_analyze::analyze(&bin_path.to_string_lossy(), None, 2, 3, None, &cwd());
    assert!(
        out.contains("FILE:"),
        "expected FILE: even for binary:\n{out}"
    );
}

#[test]
fn analyze_with_ast_recursion_limit() {
    let out = code_analyze::analyze(&fixture("sample.rs"), None, 2, 3, Some(50), &cwd());
    assert!(
        out.contains("FILE:"),
        "expected FILE: with recursion limit:\n{out}"
    );
}

// ── Analyze project's own source ───────────────────────────────────────

#[test]
fn analyze_own_lib_rs() {
    let lib_rs = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/lib.rs")
        .to_string_lossy()
        .to_string();
    let out = code_analyze::analyze(&lib_rs, None, 2, 3, None, &cwd());
    // lib.rs is very small, but should still produce FILE: header
    assert!(out.contains("FILE:"), "expected FILE: for lib.rs:\n{out}");
}

#[test]
fn analyze_own_src_directory() {
    let src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .to_string_lossy()
        .to_string();
    let out = code_analyze::analyze(&src_dir, None, 2, 3, None, &cwd());
    assert!(
        out.contains("SUMMARY:"),
        "expected SUMMARY for src/:\n{out}"
    );
    assert!(
        out.contains("rust"),
        "expected 'rust' language in stats:\n{out}"
    );
}

// ── Relative path handling ─────────────────────────────────────────────

#[test]
fn analyze_relative_path() {
    let out = code_analyze::analyze("tests/fixtures/sample.rs", None, 2, 3, None, &cwd());
    assert!(
        out.contains("FILE:"),
        "expected FILE: for relative path:\n{out}"
    );
    assert!(out.contains("main"), "expected 'main' function:\n{out}");
}
