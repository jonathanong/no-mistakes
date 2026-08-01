use super::*;

#[test]
fn closing_fence_rejects_excess_indent_and_trailing_content() {
    assert!(!is_closing_fence_line(b"    ```", b'`', 3, 0, 0));
    assert!(!is_closing_fence_line(b"\t```", b'`', 3, 0, 0));
    assert!(!is_closing_fence_line(b" \t```", b'`', 3, 0, 0));
    assert!(!is_closing_fence_line(b"``` trailing", b'`', 3, 0, 0));
}

#[test]
fn closing_fence_requires_the_opening_blockquote_depth() {
    assert!(!is_closing_fence_line(b"> ```", b'`', 3, 0, 0));
    assert!(!is_closing_fence_line(b"```", b'`', 3, 1, 0));
    assert!(!is_closing_fence_line(b"> > ```", b'`', 3, 1, 0));
    assert!(is_closing_fence_line(b"> ```", b'`', 3, 1, 0));
    assert!(is_closing_fence_line(b"> > ```", b'`', 3, 2, 0));
}

#[test]
fn closing_fence_scan_accepts_a_matching_quoted_delimiter() {
    let source = "> ```\n";
    let matching = FenceDelimiter {
        marker: b'`',
        length: 3,
        blockquote_depth: 1,
        container_indent: 0,
        content_start: 0,
    };

    assert!(has_closing_fence(source, matching, source.len()));
    assert!(!has_closing_fence(
        source,
        FenceDelimiter {
            blockquote_depth: 0,
            ..matching
        },
        source.len()
    ));
}

#[test]
fn blockquote_and_line_end_helpers_cover_boundary_forms() {
    assert_eq!(
        strip_blockquote_prefix(b"    > quote"),
        (&b"    > quote"[..], 0)
    );
    assert_eq!(strip_blockquote_prefix(b">quote"), (&b"quote"[..], 1));
    assert_eq!(strip_blockquote_prefix(b" > > quote"), (&b"quote"[..], 2));
    assert_eq!(line_end_with_ending("line\r\nnext", 4), 6);
    assert_eq!(line_end_with_ending("line\rnext", 4), 5);
    assert_eq!(line_end_with_ending("line", 4), 4);
}
