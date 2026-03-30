# mdsn - Markdown section numbers checker

Markdown section numbers checker.

## Features

- 🌐 **Cross-platform** — Runs on Linux, macOS, and Windows (x86\_64 / aarch64 / Apple Silicon)
- 🔧 **JSON output** — All primary commands support `--json` for machine-readable output
- 📦 **Single binary** — Just download and run; no extra setup required
- ⚡ **Easy install** — Single-command installation via shell script or PowerShell

## Installation

### Linux / macOS

```bash
curl -fsSL https://raw.githubusercontent.com/23prime/mdsn/latest/install.sh | sh
```

### Windows

```powershell
irm https://raw.githubusercontent.com/23prime/mdsn/latest/install.ps1 | iex
```

## Usage

```bash
mdsn <file.md>
```

Check all `.md` files:

```bash
mdsn '**/*.md'
```

## Error Codes

| Code | Description |
| ------ | ------------- |
| `TRAILING_DOT` | Section number requires trailing dot (e.g., `1.` not `1`) |
| `SPACING` | Exactly one space required after number |
| `DEPTH_MISMATCH` | Heading level doesn't match number depth |
| `MISSING_PARENT` | Parent section not defined before child |
| `ORDER` | Section numbers not in ascending order |

## Development

### Pre-requirements

- [mise](https://mise.jdx.dev)
- [rustup](https://rustup.rs)

### Commands

```bash
mise run setup   # Install tools
mise run check   # Lint / format / test
mise run fix     # Auto fix
```

### Release

```bash
mise run release -- patch   # Bump version (patch / minor / major) -> create tag and push to trigger CI release
```
