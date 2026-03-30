# Validation Guide

## Layer boundaries

Each layer validates only what it owns. Do not leak logic across layers.

| Layer | Responsibility |
| ----- | -------------- |
| `main.rs` (clap) | Syntactic/type-level checks: required args, pattern syntax |
| `extractor.rs` | Structural parsing: identifies numbered headings, extracts fields |
| `checker.rs` | Semantic validation: checks rules against the parsed heading list |

## `main.rs` (clap)

Handle only what clap can check from the argument definition: required flags,
type coercion (e.g. "cannot parse as integer"), mutex groups.

Do **not** add domain-level constraints here. For example, "at least one pattern
must be provided" is currently checked with a manual guard in `run()` — keep it
there, not in an `#[arg]` attribute.

## `extractor.rs`

`extract_headings` parses every line and returns a `Vec<HeadingLine>`. It does not
validate correctness — it just extracts structure. A `HeadingLine` may represent
an invalid section number; that is the checker's responsibility.

What the extractor does:

- Skips h1 and non-numbered headings silently (returns nothing for them)
- Skips headings inside fenced code blocks
- Captures `raw_number`, `segments`, `spacing` as-is — even if malformed

What the extractor does **not** do:

- Report errors
- Skip headings based on whether they are valid

## `checker.rs`

`check` receives the full list of `HeadingLine`s and returns `Vec<CheckError>`.
It applies all semantic rules in a single pass.

Rules are checked in this order for each heading:

1. Trailing dot (`TRAILING_DOT`)
2. Spacing (`SPACING`)
3. Depth match (`DEPTH_MISMATCH`) — if wrong depth, skip ORDER/MISSING_PARENT to avoid noise
4. Parent existence (`MISSING_PARENT`)
5. Ascending order (`ORDER`)

Do **not** add I/O, file reading, or output formatting to `checker.rs`.

## Args validation with `try_new`

Use a fallible constructor when construction can fail due to a domain invariant
that cannot be expressed in the clap definition:

```rust
impl MyArgs {
    pub fn try_new(patterns: Vec<String>, json: bool) -> anyhow::Result<Self> {
        if patterns.is_empty() {
            anyhow::bail!("at least one pattern is required");
        }
        Ok(Self { patterns, json })
    }
}
```

Use plain `new` only when all arguments are always valid.

Errors from `try_new` propagate naturally to `main` via `?` and exit with code 2.
