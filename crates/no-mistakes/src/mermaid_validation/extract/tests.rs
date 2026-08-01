use super::*;

#[test]
fn delimiter_and_line_helpers_cover_non_fence_and_crlf_edges() {
    assert!(opening_delimiter("plain text", usize::MAX).is_none());
    assert!(opening_delimiter("`` mermaid", 0).is_none());
    assert_eq!(line_number("one\r\ntwo\rthree", usize::MAX), 3);
}
