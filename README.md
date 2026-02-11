# code-analyze

[![CI](https://github.com/utapyngo/code-analyze/actions/workflows/ci.yml/badge.svg)](https://github.com/utapyngo/code-analyze/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/code-analyze)](https://crates.io/crates/code-analyze)
[![codecov](https://codecov.io/gh/utapyngo/code-analyze/graph/badge.svg)](https://codecov.io/gh/utapyngo/code-analyze)
[![License](https://img.shields.io/crates/l/code-analyze)](LICENSE)

Tree-sitter based code structure analyzer for AI agents.

Analyzes code structure and relationships — file overviews, call graphs, and symbol tracking across codebases. Designed as an [Agent Skill](https://agentskills.io/specification) for use with AI coding assistants.

## Supported Languages

- Python
- Rust
- JavaScript / TypeScript
- Go
- Java
- Kotlin
- Swift
- Ruby

## Build

```bash
cargo build --release
```

The binary is at `target/release/analyze`.

## Package as Agent Skill

```bash
mise run skill
```

This creates `dist/code-analyze/` with:
- `SKILL.md` — skill metadata and usage instructions
- `scripts/analyze` — the binary

## Install

Symlink the packaged skill directory to where your agent discovers skills:

```bash
ln -s "$(pwd)/dist/code-analyze" ~/.claude/skills/code-analyze
```

Or symlink just the binary for direct CLI use:

```bash
ln -s "$(pwd)/target/release/analyze" ~/.local/bin/analyze
```

## Usage

```bash
analyze src/                        # directory overview
analyze src/main.rs                 # single file analysis
analyze -f handle_request .         # track symbol across codebase
analyze -f process -d 4 src/        # deep call chain tracking
analyze -m 1 .                      # shallow directory overview
```

See `analyze --help` or [SKILL.md](SKILL.md) for full documentation.

## License

Apache-2.0 — see [LICENSE](LICENSE) and [NOTICE](NOTICE) for details.

Based on code from [block/goose](https://github.com/block/goose).
