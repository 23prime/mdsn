# Implementation Patterns and Gotchas

## Source layout

```text
src/
├── main.rs       # CLI (clap), file discovery, output formatting
├── extractor.rs  # Markdown heading parser → HeadingLine
└── checker.rs    # Validation logic → CheckError
```

Each layer has a single responsibility. Do not put validation logic in `main.rs`
or output logic in `checker.rs`.

## Adding a new validation rule

### 1. Add the error code to `checker.rs`

```rust
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    TrailingDot,
    Spacing,
    DepthMismatch,
    MissingParent,
    Order,
    MyNewRule,  // add here
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // ...
            ErrorCode::MyNewRule => write!(f, "MY_NEW_RULE"),
        }
    }
}
```

### 2. Emit the error in `check()`

Push a `CheckError` with the new code at the appropriate point in the loop:

```rust
errors.push(CheckError {
    line_no: h.line_no,
    code: ErrorCode::MyNewRule,
    message: format!("..."),
});
```

### 3. Add tests

Add a `#[test]` in `checker::tests` that asserts the new code appears in the
output for a triggering input (see Testing Guide).

## clap patterns

### Bool flags

`bool` fields in clap derive become presence flags — no value needed.
`default_value = "false"` is redundant:

```rust
// correct
#[arg(long)]
json: bool,

// redundant — clap already defaults bool to false
#[arg(long, default_value = "false")]
json: bool,
```

### Mutually exclusive flags — use `conflicts_with`

When two flags cannot be used together, declare the constraint in clap rather than
silently ignoring one of them at runtime:

```rust
// --verbose is meaningless alongside --json; make it an error at parse time
#[arg(long, short, conflicts_with = "json")]
verbose: bool,
```

This surfaces the conflict in `--help` and produces a clear error message instead
of silently swallowing the flag.

### Constrained string options — use `ValueEnum`

When a flag accepts a fixed set of values, use `clap::ValueEnum` instead of `String`:

```rust
#[derive(clap::ValueEnum, Clone)]
enum OutputFormat {
    Text,
    Json,
}
```

This gives automatic `--help` documentation and compile-time exhaustiveness.

## Output modes

`main.rs` controls output. `checker.rs` only returns `Vec<CheckError>` — it never prints.

Text errors go to **stderr**; success messages and JSON go to **stdout**. This allows
`mdsn '**/*.md' | jq ...` to work correctly in JSON mode.

When adding a new output mode, keep the branching in `run()` and keep `checker.rs` unchanged.

## File discovery

`collect_files` expands glob patterns, deduplicates, and respects `.gitignore`.
It returns a sorted `Vec<PathBuf>` so output is deterministic across runs.

Do not change the sort order — tests and downstream tools may depend on it.

## Error propagation

Use `anyhow::Context` to attach file path context to I/O errors:

```rust
std::fs::read_to_string(path)
    .with_context(|| format!("failed to read {}", path.display()))?;
```

Fatal errors (exit code 2) propagate via `Err` from `run()`. Validation errors
(exit code 1) are signalled by returning `Ok(true)`.

## HeadingLine fields do not map to raw line byte positions

`extractor.rs` trims leading spaces after the `#` run before extracting
`raw_number` and `spacing`. As a result, `HeadingLine` fields cannot be used
directly as byte offsets into the original line.

When you need to locate the section number inside the original line (e.g. in
`fixer.rs`), derive the position from the line itself:

```rust
// Correct: count actual spaces after '#' run in the raw line
let after_hashes = &line[h.level..];
let leading_spaces = after_hashes.len() - after_hashes.trim_start_matches(' ').len();
let num_start = h.level + leading_spaces;

// Wrong: assumes exactly one space after '#' run
let num_start = h.level + 1;
```

Also, `spacing` only captures ASCII space characters — not tabs or other
whitespace. When normalizing the separator between number and title, skip all
ASCII whitespace:

```rust
let title_offset = after_num
    .find(|c: char| !c.is_ascii_whitespace())
    .unwrap_or(after_num.len());
```
