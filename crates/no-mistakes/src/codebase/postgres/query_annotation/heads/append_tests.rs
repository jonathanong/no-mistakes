use super::support::{assert_markers, fixture};

#[test]
fn append_chains_keep_heads_only_before_local_escapes() {
    let (source, _, heads) = fixture("append.mts");
    assert_eq!(assert_markers(&source, &heads), 22);
}

#[test]
fn repeated_execution_never_uses_textual_order_to_hide_mutation() {
    let (source, _, heads) = fixture("append-loops.mts");
    assert_eq!(assert_markers(&source, &heads), 18);
}

#[test]
fn append_suffixes_are_checked_without_composing_sql() {
    let (source, _, heads) = fixture("append-literal-arguments.mts");
    assert_eq!(assert_markers(&source, &heads), 14);
}
