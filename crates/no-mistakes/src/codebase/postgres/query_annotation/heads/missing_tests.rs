use super::support::fixture;
use crate::codebase::ts_source::byte_offset_to_line;

#[test]
fn direct_unannotated_heads_are_distinct_from_unknown_shapes() {
    let (source, _, heads) = fixture("low-missing.mts");
    let mut missing = Vec::new();
    let mut unproven = Vec::new();
    for span in heads.calls.keys() {
        let tail = &source[span.0 as usize..];
        if !tail.starts_with("read(") {
            continue;
        }
        let line = byte_offset_to_line(&source, span.0 as usize) as usize;
        if heads.proves_missing(*span) {
            missing.push(line);
        } else if !heads.accepts(*span) {
            unproven.push(line);
        }
    }
    missing.sort_unstable();
    unproven.sort_unstable();
    assert_eq!(
        missing,
        [3, 4, 5, 7, 8, 11, 19, 21, 27, 29, 33, 34, 35, 36, 37, 38, 39, 42, 43, 44, 47]
    );
    assert_eq!(
        unproven,
        [12, 13, 14, 15, 16, 17, 18, 22, 23, 24, 25, 32, 40, 41]
    );
}

#[test]
fn repeated_leading_comments_keep_a_partial_transaction_prefix_unknown() {
    let (source, _, heads) = fixture("low-comment-prefix.mts");
    let span = heads
        .calls
        .keys()
        .find(|span| source[span.0 as usize..].starts_with("read("))
        .unwrap();
    assert!(!heads.accepts(*span));
    assert!(!heads.proves_missing(*span));
}
