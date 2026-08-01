use super::*;

#[test]
fn closing_fence_rejects_excess_indent_and_trailing_content() {
    assert!(!is_closing_fence_line(b"    ```", b'`', 3, 0));
    assert!(!is_closing_fence_line(b"\t```", b'`', 3, 0));
    assert!(!is_closing_fence_line(b" \t```", b'`', 3, 0));
    assert!(!is_closing_fence_line(b"``` trailing", b'`', 3, 0));
    assert!(is_closing_fence_line(b">```", b'`', 3, 0));
}

#[test]
fn blockquote_and_line_end_helpers_cover_boundary_forms() {
    assert_eq!(strip_blockquote_prefix(b"    > quote"), b"    > quote");
    assert_eq!(strip_blockquote_prefix(b">quote"), b"quote");
    assert_eq!(line_end_with_ending("line\rnext", 4), 5);
    assert_eq!(line_end_with_ending("line", 4), 4);
}
