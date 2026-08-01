use super::*;

#[test]
fn masks_only_confirmed_exact_multiline_code_spans() {
    let source = b"`example\n```mermaid\n```\n` tail";
    let mut scanner = CodeSpanScanner::default();

    scanner.observe_source(source, 0, 8);
    assert!(scanner.is_masking_markdown());
    scanner.observe_source(source, 9, 19);
    assert!(scanner.is_masking_markdown());
    scanner.observe_source(source, 20, 23);
    assert!(scanner.is_masking_markdown());
    scanner.observe_source(source, 24, source.len());
    assert!(!scanner.is_masking_markdown());
}

#[test]
fn handles_unmatched_escaped_and_crlf_backticks() {
    for source in [
        b"`unmatched\n\ntext".as_slice(),
        b"\\`escaped`",
        b"`crlf\r\ncontent\r\n`",
    ] {
        let mut scanner = CodeSpanScanner::default();
        scanner.observe_source(source, 0, source.len());
        assert!(!scanner.is_masking_markdown(), "{source:?}");
    }
}
