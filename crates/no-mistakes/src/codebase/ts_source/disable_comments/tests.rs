use super::LineDirectives;

#[test]
fn scan_through_leaves_a_source_without_line_directives_unscanned() {
    let source = "const value = 1;\n";
    let mut lines = LineDirectives::new(source, true);
    assert!(!lines.scan_inline);
    assert!(!lines.scan_next);
    lines.scan_through(source, 1, None);
    assert_eq!(lines.scanned_line, 0);
    assert_eq!(lines.offset, 0);
}
