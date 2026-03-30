mod checker;
mod extractor;

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about = "Check section number consistency in Markdown files")]
struct Args {
    /// File paths or glob patterns (e.g. '**/*.md')
    patterns: Vec<String>,
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

fn run() -> Result<bool> {
    let args = Args::parse();

    if args.patterns.is_empty() {
        eprintln!("Usage: mdsn <patterns...>");
        return Ok(false);
    }

    let files = collect_files(&args.patterns)?;
    let mut has_errors = false;

    for path in &files {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let headings = extractor::extract_headings(&content);
        let errors = checker::check(&headings);

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
