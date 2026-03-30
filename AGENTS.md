# AGENTS.md

This file provides guidance to AI coding agents when working with code in this repository.

## General agent rules

- When users ask questions, answer them instead of doing the work.

### Shell Rules

- Always use `rm -f` (never bare `rm`)
- Before running a series of `git` commands, confirm you are in the project root; if not, `cd` there first. Then run all subsequent `git` commands from that directory without the `-C` option.

## Project Overview

`mdsn` is a CLI tool that validates section number consistency in Markdown files.

### What it checks

- Headings h2–h6 with numbered sections (e.g. `## 1.2.3. Title`); h1 is ignored
- Trailing dot required (`1.2.` not `1.2`)
- Exactly one space after the trailing dot
- Heading level must match section number depth (h3 → depth 2)
- Parent section must be defined before child sections
- Section numbers must be consecutive and ascending (starts at 1, no gaps)

Error output format: `<file>:<line>: [<CODE>] <message>`

Exit code 0 if valid, 1 if errors, 2 on fatal error.

### Source layout

```txt
src/
├── main.rs       # CLI (clap), file discovery (glob + ignore), output
├── extractor.rs  # Markdown heading parser → HeadingLine
└── checker.rs    # Validation logic → CheckError
```

See `docs/SPEC.md` for the full specification.
See `docs/PATTERNS.md` for implementation patterns and gotchas.
See `docs/TESTING.md` for the testing guide.
See `docs/VALIDATION.md` for validation layer boundaries.

## Development Commands

Use `mise run <task>` for all development operations. Do not invoke `cargo` or other tools directly.

| Task | Alias | Description |
| --- | --- | --- |
| `mise run rs-test` | `rt` | Run tests |
| `mise run rs-lint` | `rl` | Lint (clippy + fmt check) |
| `mise run rs-fix` | `rf` | Auto-fix lint and format |
| `mise run rs-check` | `rc` | Lint + test |
| `mise run rs-build` | | Debug build |
| `mise run rs-build-release` | | Release build |
| `mise run rs-run` | `rr` | Run the application |
| `mise run check` | `c` | Check everything (Markdown, GH Actions, spell, Rust) |
| `mise run fix` | `f` | Fix everything |
