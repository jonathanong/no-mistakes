use super::support::{assert_markers, fixture};

#[test]
fn captured_returned_builder_keeps_only_closed_prefix_uses() {
    let (source, _, heads) = fixture("returned-captured-prefix.mts");
    assert_eq!(assert_markers(&source, &heads), 10);
    let (source, _, heads) = fixture("returned-captured-dynamic.mts");
    assert_eq!(assert_markers(&source, &heads), 2);
}
