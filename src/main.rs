mod checker;
mod extractor;

use anyhow::{Context, Result};
use clap::Parser;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about = "Check section number consistency in Markdown files")]
struct Args {
    /// File paths or glob patterns (e.g. '**/*.md')
    patterns: Vec<String>,

    /// Output results as JSON
    #[arg(long)]
    json: bool,

    /// Print per-file status and a summary
    #[arg(long, short)]
    verbose: bool,
}

fn collect_files(patterns: &[String]) -> Result<Vec<PathBuf>> {
    use ignore::gitignore::GitignoreBuilder;

    let mut builder = GitignoreBuilder::new(".");
    builder.add(".gitignore");
    let gitignore = builder.build()?;

    let mut files = Vec::new();
    for pattern in patterns {
        let matches =
            glob::glob(pattern).with_context(|| format!("invalid glob pattern: {pattern}"))?;
        for entry in matches {
            let path = entry?;
            if path.is_file() && !gitignore.matched(&path, false).is_ignore() {
                files.push(path);
            }
        }
    }

    files.sort();
    files.dedup();
    Ok(files)
}

#[derive(Serialize)]
struct JsonError {
    file: String,
    line: usize,
    code: checker::ErrorCode,
    message: String,
}

#[derive(Serialize)]
struct JsonOutput {
    valid: bool,
    errors: Vec<JsonError>,
}

fn run() -> Result<bool> {
    let args = Args::parse();

    if args.patterns.is_empty() {
        eprintln!("Usage: mdsn <patterns...>");
        return Ok(false);
    }

    let files = collect_files(&args.patterns)?;

    if args.json {
        let mut json_errors: Vec<JsonError> = Vec::new();

        for path in &files {
            let content = std::fs::read_to_string(path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            let headings = extractor::extract_headings(&content);
            let errors = checker::check(&headings);

            for e in errors {
                json_errors.push(JsonError {
                    file: path.display().to_string(),
                    line: e.line_no,
                    code: e.code,
                    message: e.message,
                });
            }
        }

        let output = JsonOutput {
            valid: json_errors.is_empty(),
            errors: json_errors,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(!output.valid);
    }

    let mut has_errors = false;
    let mut total_errors: usize = 0;

    for path in &files {
        if args.verbose {
            eprintln!("Checking {}...", path.display());
        }

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let headings = extractor::extract_headings(&content);
        let errors = checker::check(&headings);

        let error_count = errors.len();
        for e in &errors {
            has_errors = true;
            eprintln!(
                "{}:{}: [{}] {}",
                path.display(),
                e.line_no,
                e.code,
                e.message
            );
        }

        if args.verbose {
            if error_count == 0 {
                eprintln!("  {}: OK", path.display());
            } else {
                eprintln!("  {}: {} error(s)", path.display(), error_count);
            }
            total_errors += error_count;
        }
    }

    if args.verbose {
        eprintln!("---");
        eprintln!("Checked {} file(s), {} error(s)", files.len(), total_errors);
    }

    if !has_errors {
        println!("All section numbers are valid.");
    }

    Ok(has_errors)
}

fn main() {
    match run() {
        Ok(true) => std::process::exit(1),
        Ok(false) => {}
        Err(e) => {
            eprintln!("[ERROR] {e:#}");
            std::process::exit(2);
        }
    }
}
