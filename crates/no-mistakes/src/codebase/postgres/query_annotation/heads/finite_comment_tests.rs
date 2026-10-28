use super::support::{assert_markers, fixture_with_effects};

#[test]
fn finite_names_require_exact_runtime_domains_and_trusted_builder_lifetimes() {
    let (source, _, heads) = fixture_with_effects("finite-comment-names.mts");
    assert_eq!(assert_markers(&source, &heads), 4);
    let (source, _, heads) = fixture_with_effects("finite-comment-unknown.mts");
    assert_eq!(assert_markers(&source, &heads), 16);
    let (source, _, heads) = fixture_with_effects("finite-comment-overbudget.mts");
    assert_eq!(assert_markers(&source, &heads), 1);
}
