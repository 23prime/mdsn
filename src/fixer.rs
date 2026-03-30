use crate::extractor::HeadingLine;
use std::collections::HashMap;

pub fn fix(content: &str, headings: &[HeadingLine]) -> String {
    let by_line: HashMap<usize, &HeadingLine> = headings.iter().map(|h| (h.line_no, h)).collect();

    let lines: Vec<&str> = content.split('\n').collect();
    let fixed: Vec<String> = lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            if let Some(h) = by_line.get(&(i + 1)) {
                apply_fix(line, h)
            } else {
                (*line).to_string()
            }
        })
        .collect();

    fixed.join("\n")
}

fn apply_fix(line: &str, h: &HeadingLine) -> String {
    let (base, crlf) = line.strip_suffix('\r').map_or((line, ""), |s| (s, "\r"));

    // Structure: '#' * level + ' ' + raw_number + spacing + title
    let prefix_len = h.level + 1;
    let num_spacing_end = prefix_len + h.raw_number.len() + h.spacing.len();
    let title = &base[num_spacing_end..];

    let fixed_num = if h.raw_number.ends_with('.') {
        h.raw_number.clone()
    } else {
        format!("{}.", h.raw_number)
    };

    format!("{}{} {}{}", &base[..prefix_len], fixed_num, title, crlf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extractor::extract_headings;

    fn fix_str(md: &str) -> String {
        let headings = extract_headings(md);
        fix(md, &headings)
    }

    #[test]
    fn test_fix_trailing_dot() {
        assert_eq!(fix_str("## 1 Title\n"), "## 1. Title\n");
    }

    #[test]
    fn test_fix_spacing_extra() {
        assert_eq!(fix_str("## 1.  Title\n"), "## 1. Title\n");
    }

    #[test]
    fn test_fix_spacing_missing() {
        assert_eq!(fix_str("## 1.Title\n"), "## 1. Title\n");
    }

    #[test]
    fn test_fix_both() {
        assert_eq!(fix_str("## 1  Title\n"), "## 1. Title\n");
    }

    #[test]
    fn test_no_change_valid() {
        let md = "## 1. Title\n";
        assert_eq!(fix_str(md), md);
    }

    #[test]
    fn test_crlf_preserved() {
        assert_eq!(fix_str("## 1 Title\r\n"), "## 1. Title\r\n");
    }

    #[test]
    fn test_nested_heading_fixed() {
        assert_eq!(fix_str("### 1.2 Sub\n"), "### 1.2. Sub\n");
    }

    #[test]
    fn test_multiple_lines() {
        let md = "## 1 A\n### 1.1.  B\n## 2. C\n";
        let expected = "## 1. A\n### 1.1. B\n## 2. C\n";
        assert_eq!(fix_str(md), expected);
    }

    #[test]
    fn test_non_heading_lines_unchanged() {
        let md = "# Title\n\nSome text.\n\n## 1. Section\n";
        assert_eq!(fix_str(md), md);
    }
}
