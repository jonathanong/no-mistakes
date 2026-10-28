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

#[test]
fn trusted_tagged_prefix_survives_opaque_append_suffixes() {
    let (source, _, heads) =
        super::support::fixture_with_append_candidates("append-opaque-prefix.mts", true);
    assert_eq!(assert_markers(&source, &heads), 30);
    let calls = super::support::decisions(&source, &heads);
    assert!(heads.proves_missing(calls[4].0), "known missing SQL head");
    assert!(heads.proves_missing(calls[5].0), "empty annotation head");
    for call in &calls[9..15] {
        assert!(!heads.proves_missing(call.0), "untrusted or partial head");
    }
    assert!(heads.accepts(calls[15].0), "complete transaction boundary");
}
