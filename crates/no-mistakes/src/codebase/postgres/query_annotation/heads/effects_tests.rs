use super::support::{assert_markers, fixture_with_effects};

#[test]
fn discarded_local_append_only_helpers_preserve_trusted_prefixes() {
    let (source, _, heads) = fixture_with_effects("append-effects.mts");
    assert_eq!(assert_markers(&source, &heads), 16);
}
