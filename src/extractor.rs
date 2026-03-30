pub struct HeadingLine {
    pub line_no: usize,
    pub level: usize,
    pub raw_number: String,
    pub segments: Vec<u32>,
    pub spacing: String,
}

pub fn extract_headings(content: &str) -> Vec<HeadingLine> {
    let mut in_code_block = false;
    let mut headings = Vec::new();

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim_end_matches('\r');
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }
        if let Some(h) = parse_heading_line(trimmed, i + 1) {
            headings.push(h);
        }
    }

    headings
}

fn parse_heading_line(line: &str, line_no: usize) -> Option<HeadingLine> {
    let level = line.chars().take_while(|&c| c == '#').count();
    if !(2..=6).contains(&level) {
        return None;
    }

    let rest = &line[level..];
    if !rest.starts_with(' ') {
        return None;
    }

    let content = rest.trim_start_matches(' ');
    if !content.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }

    let num_end = content
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(content.len());
    let raw_number = &content[..num_end];
    let after_num = &content[num_end..];

    let space_end = after_num
        .find(|c: char| c != ' ')
        .unwrap_or(after_num.len());
    let spacing = &after_num[..space_end];

    let segments = raw_number
        .trim_end_matches('.')
        .split('.')
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse::<u32>().ok())
        .collect();

    Some(HeadingLine {
        line_no,
        level,
        raw_number: raw_number.to_string(),
        segments,
        spacing: spacing.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn heading(content: &str) -> Option<HeadingLine> {
        parse_heading_line(content, 1)
    }

    #[test]
    fn test_h1_ignored() {
        assert!(heading("# 1. Title").is_none());
    }

    #[test]
    fn test_non_numbered_ignored() {
        assert!(heading("## Introduction").is_none());
        assert!(heading("## ").is_none());
    }

    #[test]
    fn test_valid_heading() {
        let h = heading("## 1. Title").unwrap();
        assert_eq!(h.level, 2);
        assert_eq!(h.raw_number, "1.");
        assert_eq!(h.segments, vec![1]);
        assert_eq!(h.spacing, " ");
    }

    #[test]
    fn test_nested_heading() {
        let h = heading("### 1.2. Title").unwrap();
        assert_eq!(h.level, 3);
        assert_eq!(h.raw_number, "1.2.");
        assert_eq!(h.segments, vec![1, 2]);
    }

    #[test]
    fn test_missing_trailing_dot() {
        let h = heading("## 1 Title").unwrap();
        assert_eq!(h.raw_number, "1");
        assert_eq!(h.spacing, " ");
    }

    #[test]
    fn test_no_space_after_number() {
        let h = heading("## 1.Title").unwrap();
        assert_eq!(h.raw_number, "1.");
        assert_eq!(h.spacing, "");
    }

    #[test]
    fn test_double_space_after_number() {
        let h = heading("## 1.  Title").unwrap();
        assert_eq!(h.raw_number, "1.");
        assert_eq!(h.spacing, "  ");
    }

    #[test]
    fn test_crlf() {
        let h = heading("## 1. Title\r").unwrap();
        assert_eq!(h.raw_number, "1.");
    }
}
