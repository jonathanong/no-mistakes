use super::*;

#[test]
fn test_empty_content() {
    let sections = parse_markdown_sections("");
    assert!(sections.is_empty());
}

#[test]
fn test_single_h1() {
    let content = "# Hello\n\nsome text\n";
    let sections = parse_markdown_sections(content);
    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].level, 1);
    assert_eq!(sections[0].heading, "Hello");
    assert_eq!(sections[0].line, 1);
}

#[test]
fn test_multiple_headings() {
    let content = "# Title\n\n## Section A\n\n### Sub\n\n## Section B\n";
    let sections = parse_markdown_sections(content);
    assert_eq!(sections.len(), 4);
    assert_eq!(sections[0].level, 1);
    assert_eq!(sections[0].heading, "Title");
    assert_eq!(sections[1].level, 2);
    assert_eq!(sections[1].heading, "Section A");
    assert_eq!(sections[2].level, 3);
    assert_eq!(sections[2].heading, "Sub");
    assert_eq!(sections[3].level, 2);
    assert_eq!(sections[3].heading, "Section B");
}

#[test]
fn test_line_numbers() {
    let content = "line1\nline2\n## Heading\nline4\n";
    let sections = parse_markdown_sections(content);
    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].line, 3);
}

#[test]
fn test_has_section_found() {
    let content = "# Title\n\n## Performance\n\ntext\n";
    assert!(has_section(content, "Performance"));
    assert!(has_section(content, "Title"));
}

#[test]
fn test_has_section_not_found() {
    let content = "# Title\n\n## Performance\n";
    assert!(!has_section(content, "performance")); // case-sensitive
    assert!(!has_section(content, "Missing"));
}

#[test]
fn test_no_headings_in_plain_text() {
    let content = "Just some plain text.\nNo headings here.\n";
    let sections = parse_markdown_sections(content);
    assert!(sections.is_empty());
}

#[test]
fn test_h4_h5_h6_levels() {
    // Exercises heading_level_to_u32 for H4, H5, H6 (lines 63-65).
    let content = "#### H4\n##### H5\n###### H6\n";
    let sections = parse_markdown_sections(content);
    assert_eq!(sections.len(), 3);
    assert_eq!(sections[0].level, 4);
    assert_eq!(sections[1].level, 5);
    assert_eq!(sections[2].level, 6);
}
