use super::mark_invalidated;

#[test]
fn earlier_deferred_offsets_replace_later_ones_without_rewriting_equal_offsets() {
    let mut offset = None;
    assert!(mark_invalidated(&mut offset, 80));
    assert!(!mark_invalidated(&mut offset, 120));
    assert_eq!(offset, Some(80));
    // A deferred projection may arrive after a physically later source write.
    assert!(mark_invalidated(&mut offset, 40));
    assert!(!mark_invalidated(&mut offset, 40));
    assert_eq!(offset, Some(40));
}
