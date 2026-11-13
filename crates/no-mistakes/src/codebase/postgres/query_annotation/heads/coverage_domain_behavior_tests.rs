use super::support::{assert_markers, fixture_with_effects};

#[test]
fn finite_helper_domains_require_exact_stable_calls_in_the_selector_context() {
    let (source, _, heads) = fixture_with_effects("coverage-finite-domains.mts");
    assert_eq!(assert_markers(&source, &heads), 18);
}

#[test]
fn finite_return_contracts_keep_totality_same_heads_and_stable_local_builders() {
    let (source, _, heads) = fixture_with_effects("coverage-return-contracts.mts");
    assert_eq!(assert_markers(&source, &heads), 18);
}
