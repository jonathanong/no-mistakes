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

#[test]
fn malformed_loop_rows_and_comment_interpolations_keep_names_unknown() {
    let (source, _, heads) = fixture_with_effects("finite-comment-grammar.mts");
    assert_eq!(assert_markers(&source, &heads), 8);
}

#[test]
fn finite_helper_name_calls_require_closed_stable_literal_provenance() {
    let (source, _, heads) = fixture_with_effects("finite-comment-domain.mts");
    assert_eq!(assert_markers(&source, &heads), 7);
}

#[path = "finite_comment_index_tests.rs"]
mod finite_comment_index_tests;
