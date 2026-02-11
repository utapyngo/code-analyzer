---
name: code-analyzer
description: Analyze code structure and relationships using tree-sitter. Use when you need to understand a codebase, find function definitions, track symbol usage across files, or get a structural overview of a directory.
---

# Code Analyzer

Static code analysis CLI using tree-sitter parsing.

## Usage

```bash
scripts/analyze [-f SYMBOL] [-d DEPTH] [-m DEPTH] <PATH>
```

## Modes

The tool auto-selects its mode based on arguments:

- **Directory** → file tree with LOC, function, and class counts
- **File** → functions, classes, imports, call relationships
- **Focused** (`-f`) → track a symbol across files with call chains

## Output Format

### File header
```
FILE: path/to/file.rs [151L, 1F, 1C]
```
`L` = lines, `F` = functions, `C` = classes/structs

### Sections

| Prefix | Meaning | Example |
|--------|---------|---------|
| `C:` | Classes/structs with line numbers | `Args:34` |
| `F:` | Functions with line numbers | `main:60`, `parse:42•4` (•N = cyclomatic complexity) |
| `I:` | Imports, `(N)` = grouped count | `use std(2)` |
| `R:` | References | `methods[name(Type)]`, `types[Name]`, `fields[Type]` |

### Directory output
```
SUMMARY:
Shown: 19 files, 2678L, 69F, 30C (max_depth=3)
Languages: rust (100%)

PATH [LOC, FUNCTIONS, CLASSES]
  mod.rs [306L, 9F, 3C]
  parser.rs [450L, 14F, 5C]
```

### Focused output (`-f SYMBOL`)
```
FOCUSED ANALYSIS: analyze

DEFINITIONS:
F1:221 - analyze

OUTGOING CALL CHAINS (depth=2):
F1:228 (analyze -> Path::new)
F1:246 (analyze -> analyze_focused) -> F1:174 (analyze_focused -> CallGraph::build_from_results)

FILES:
  F1: src/analyze/mod.rs
  F2: src/analyze/traversal.rs
```

Format: `F<file>:<line> (caller -> callee)`. File index maps to `FILES:` section.

## Options

| Flag | Default | Description |
|------|---------|-------------|
| `-f SYMBOL` | — | Track symbol across files |
| `-d DEPTH` | 2 | Call graph depth (0 = definition only) |
| `-m DEPTH` | 3 | Directory recursion limit (0 = unlimited) |
| `--ast-recursion-limit N` | unlimited | Prevent stack overflow in deeply nested code |

## Examples

```bash
scripts/analyze src/main.rs             # single file analysis
scripts/analyze src/                    # directory overview
scripts/analyze -f handle_request .     # track symbol across codebase
scripts/analyze -m 1 .                  # shallow overview
scripts/analyze -f process -d 4 src/    # deep call chain tracking
```

## Supported Languages

- Python
- Rust
- JavaScript / TypeScript
- Go
- Java
- Kotlin
- Swift
- Ruby
