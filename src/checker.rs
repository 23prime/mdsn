use std::collections::{HashMap, HashSet};

use crate::extractor::HeadingLine;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    TrailingDot,
    Spacing,
    DepthMismatch,
    MissingParent,
    Order,
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCode::TrailingDot => write!(f, "TRAILING_DOT"),
            ErrorCode::Spacing => write!(f, "SPACING"),
            ErrorCode::DepthMismatch => write!(f, "DEPTH_MISMATCH"),
            ErrorCode::MissingParent => write!(f, "MISSING_PARENT"),
            ErrorCode::Order => write!(f, "ORDER"),
        }
    }
}

#[derive(Serialize)]
pub struct CheckError {
    pub line_no: usize,
    pub code: ErrorCode,
    pub message: String,
}

pub fn check(headings: &[HeadingLine]) -> Vec<CheckError> {
    let mut errors = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut last_counter: HashMap<String, u32> = HashMap::new();

    for h in headings {
        let has_trailing_dot = h.raw_number.ends_with('.');

        if !has_trailing_dot {
            errors.push(CheckError {
                line_no: h.line_no,
                code: ErrorCode::TrailingDot,
                message: format!(
                    "section number {} requires a trailing dot (e.g., {}.)",
                    h.raw_number, h.raw_number
                ),
            });
        }

        if h.spacing != " " {
            errors.push(CheckError {
                line_no: h.line_no,
                code: ErrorCode::Spacing,
                message: format!(
                    "section number {} must be followed by exactly one space",
                    h.raw_number
                ),
            });
        }

        if h.segments.is_empty() {
            continue;
        }

        let expected_depth = h.level - 1;
        let depth_ok = h.segments.len() == expected_depth;

        if !depth_ok {
            errors.push(CheckError {
                line_no: h.line_no,
                code: ErrorCode::DepthMismatch,
                message: format!(
                    "heading level (h{}) does not match section number depth {}",
                    h.level, h.raw_number
                ),
            });
            // Skip ORDER/MISSING_PARENT to avoid noise from wrong depth
            let full_key = segments_key(&h.segments);
            seen.insert(full_key);
            continue;
        }

        if h.segments.len() > 1 {
            let parent_key = segments_key(&h.segments[..h.segments.len() - 1]);
            if !seen.contains(&parent_key) {
                let child_num = format!("{}.", segments_key(&h.segments));
                let parent_num = format!("{}.", parent_key);
                errors.push(CheckError {
                    line_no: h.line_no,
                    code: ErrorCode::MissingParent,
                    message: format!(
                        "child section {child_num} appears before parent section {parent_num} is defined"
                    ),
                });
            }
        }

        let current = *h.segments.last().unwrap();
        let parent_key = if h.segments.len() > 1 {
            segments_key(&h.segments[..h.segments.len() - 1])
        } else {
            String::new()
        };

        let scope = if parent_key.is_empty() {
            "at top-level".to_string()
        } else {
            format!("under section {}.", parent_key)
        };

        let prev = last_counter.get(&parent_key).copied().unwrap_or(0);
        if current != prev + 1 {
            errors.push(CheckError {
                line_no: h.line_no,
                code: ErrorCode::Order,
                message: format!(
                    "headings {scope} are not in ascending order (previous: {prev}, current: {current})"
                ),
            });
        }

        last_counter.insert(parent_key, current);
        seen.insert(segments_key(&h.segments));
    }

    errors
}

fn segments_key(segments: &[u32]) -> String {
    segments
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(".")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extractor::extract_headings;

    fn check_str(md: &str) -> Vec<CheckError> {
        check(&extract_headings(md))
    }

    fn codes(errors: &[CheckError]) -> Vec<String> {
        errors.iter().map(|e| e.code.to_string()).collect()
    }

    #[test]
    fn test_valid() {
        let md = "## 1. A\n### 1.1. B\n### 1.2. C\n## 2. D\n";
        assert!(check_str(md).is_empty());
    }

    #[test]
    fn test_trailing_dot() {
        let errors = check_str("## 1 Title\n");
        assert!(codes(&errors).contains(&"TRAILING_DOT".to_string()));
    }

    #[test]
    fn test_spacing() {
        let errors = check_str("## 1.Title\n");
        assert!(codes(&errors).contains(&"SPACING".to_string()));

        let errors = check_str("## 1.  Title\n");
        assert!(codes(&errors).contains(&"SPACING".to_string()));
    }

    #[test]
    fn test_depth_mismatch() {
        let errors = check_str("## 1.1. Title\n");
        assert!(codes(&errors).contains(&"DEPTH_MISMATCH".to_string()));
    }

    #[test]
    fn test_missing_parent() {
        let errors = check_str("### 1.1. Child\n## 1. Parent\n");
        assert!(codes(&errors).contains(&"MISSING_PARENT".to_string()));
    }

    #[test]
    fn test_order_must_start_at_1() {
        let errors = check_str("## 2. Second\n");
        assert!(codes(&errors).contains(&"ORDER".to_string()));
    }

    #[test]
    fn test_order_not_consecutive() {
        let errors = check_str("## 1. First\n## 3. Third\n");
        assert!(codes(&errors).contains(&"ORDER".to_string()));
    }

    #[test]
    fn test_order_not_ascending() {
        let errors = check_str("## 2. B\n## 1. A\n");
        assert!(codes(&errors).contains(&"ORDER".to_string()));
    }

    #[test]
    fn test_child_order_resets_per_parent() {
        let md = "## 1. A\n### 1.1. B\n## 2. C\n### 2.1. D\n";
        assert!(check_str(md).is_empty());
    }
}
