# Specification: Markdown Section Numbers Checker

## Overview

This tool validates the consistency of section numbers in Markdown files.

## Target Headings

- Headings from h2 (`##`) to h6 (`######`) with numbered sections
- Pattern: `## 1.2.3. Title`
- h1 (`#`) is not checked (used for document title)

## Section Number Format

```md
## {number}. {title}
```

- `{number}`: Dot-separated digits (e.g., `1`, `1.2`, `1.2.3`)
- Must end with a trailing dot (e.g., `1.` not `1`)
- Exactly one space after the trailing dot

## Validation Rules

### Trailing Dot Required

Section numbers must end with a dot.

| Input | Valid |
| ------- | ------- |
| `## 1. Title` | Yes |
| `## 1 Title` | No |
| `## 1.2. Title` | Yes |
| `## 1.2 Title` | No |

### Single Space After Number

Exactly one space is required after the section number.

| Input | Valid |
| ------- | ------- |
| `## 1. Title` | Yes |
| `## 1.Title` | No |
| `## 1.  Title` | No |

### Level-Depth Consistency

The heading level must match the depth of section number.

| Heading Level | Expected Depth | Example |
| --------------- | ---------------- | --------- |
| h2 (`##`) | 1 | `## 1.` |
| h3 (`###`) | 2 | `### 1.1.` |
| h4 (`####`) | 3 | `#### 1.1.1.` |
| h5 (`#####`) | 4 | `##### 1.1.1.1.` |
| h6 (`######`) | 5 | `###### 1.1.1.1.1.` |

### Parent Before Child

A parent section must be defined before its child sections.

**Valid:**

```markdown
## 1. Parent
### 1.1. Child
```

**Invalid:**

```markdown
### 1.1. Child
## 1. Parent
```

### Ascending Order

Section numbers under the same parent must be in ascending order.

**Valid:**

```markdown
## 1. First
## 2. Second
## 3. Third
```

**Invalid:**

```markdown
## 2. Second
## 1. First
```

## Output

### Text (default)

Errors are written to stderr in the following format:

```text
<file>:<line>: [<CODE>] <message>
```

If no errors are found, `All section numbers are valid.` is printed to stdout.

### Verbose (`--verbose` / `-v`)

Pass `--verbose` (or `-v`) to print per-file processing status and a final summary to stderr:

```text
Checking path/to/file1.md...
  path/to/file1.md: OK
Checking path/to/file2.md...
path/to/file2.md:3: [TRAILING_DOT] section number 1 requires a trailing dot (e.g., 1.)
  path/to/file2.md: 1 error(s)
---
Checked 2 file(s), 1 error(s)
```

Per-file summaries (e.g., `docs/spec.md: OK`) are indented by two spaces to visually separate them from error lines. Normal error output and exit codes are unchanged. Verbose output is always written to stderr.

### Fix (`--fix`)

Pass `--fix` to automatically correct `TRAILING_DOT` and `SPACING` errors in place. After
fixing, any remaining errors are reported to stderr and the exit code reflects them.

Errors that require human judgment (`DEPTH_MISMATCH`, `MISSING_PARENT`, `ORDER`) are not
modified.

`--fix` cannot be combined with `--json`.

### JSON (`--json`)

Pass `--json` to output results as JSON to stdout:

```json
{
  "valid": false,
  "errors": [
    {
      "file": "path/to/file.md",
      "line": 5,
      "code": "TRAILING_DOT",
      "message": "section number 1 requires a trailing dot (e.g., 1.)"
    }
  ]
}
```

- `valid`: `true` if no errors were found, `false` otherwise
- `errors`: array of error objects; empty when valid

## Error Messages

| Code | Error Message |
| ------ | --------------- |
| `TRAILING_DOT` | `section number {number} requires a trailing dot (e.g., {number}.)` |
| `SPACING` | `section number {number} must be followed by exactly one space` |
| `DEPTH_MISMATCH` | `heading level (h{level}) does not match section number depth {number}` |
| `MISSING_PARENT` | `child section {child} appears before parent section {parent} is defined` |
| `ORDER` | `headings {scope} are not in ascending order (previous: {prev}, current: {curr})` |
