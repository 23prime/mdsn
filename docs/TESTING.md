# Testing Guide

## Overview

Tests live inline in each source file under `#[cfg(test)]` modules.
No external test server or credentials are required.

## Extractor tests (`extractor.rs`)

Test `parse_heading_line` directly via a local helper:

```rust
fn heading(content: &str) -> Option<HeadingLine> {
    parse_heading_line(content, 1)
}

#[test]
fn test_valid_heading() {
    let h = heading("## 1. Title").unwrap();
    assert_eq!(h.level, 2);
    assert_eq!(h.raw_number, "1.");
    assert_eq!(h.segments, vec![1]);
    assert_eq!(h.spacing, " ");
}
```

Test both inputs that should parse (assert fields) and inputs that should not (assert `None`).

## Checker tests (`checker.rs`)

Use two helpers to keep tests concise:

```rust
fn check_str(md: &str) -> Vec<CheckError> {
    check(&extract_headings(md))
}

fn codes(errors: &[CheckError]) -> Vec<String> {
    errors.iter().map(|e| e.code.to_string()).collect()
}
```

Assert that specific error codes appear (or don't appear) for a given input:

```rust
#[test]
fn test_trailing_dot() {
    let errors = check_str("## 1 Title\n");
    assert!(codes(&errors).contains(&"TRAILING_DOT".to_string()));
}

#[test]
fn test_valid() {
    let md = "## 1. A\n### 1.1. B\n## 2. C\n";
    assert!(check_str(md).is_empty());
}
```

Avoid asserting on exact error messages — assert on error codes, which are the stable API.

## Testing output branches

Any code path that behaves differently based on `--json` must have two tests: one for text output and one for JSON output. Omitting the JSON path leaves the serialization logic untested.

This applies to `run()` itself once integration tests are added, and to any future helper that formats results.

## Tests that change the working directory

When a test calls `std::env::set_current_dir`, always restore the original directory with a Drop guard so it is recovered even on panic:

```rust
struct DirGuard(std::path::PathBuf);

impl Drop for DirGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.0);
    }
}

let original = std::env::current_dir().unwrap();
std::env::set_current_dir(&tmp_dir).unwrap();
let _guard = DirGuard(original);
```

Without this, a panicking test leaves the process in the wrong directory and breaks subsequent tests that use relative paths. This pattern is required when testing config file loading (global vs project-local lookup).

## `#[cfg_attr(test, derive(Debug))]`

When a struct is the `T` in `Result<T, _>` and tests call `.unwrap_err()`, the compiler requires `T: Debug`. Add a conditional derive to avoid a compile error without bloating release builds:

```rust
#[cfg_attr(test, derive(Debug))]
pub struct MyArgs { ... }
```

## Rules

- Do not read from the filesystem in checker or extractor tests — pass strings directly.
- Place shared test fixtures (e.g. `sample_md()`) at module level with `#[cfg(test)]`, not inside `mod tests { ... }`, so sibling test modules can reuse them.
- Each test should cover a single behavior. Prefer many small tests over one large test.
