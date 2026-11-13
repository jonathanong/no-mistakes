use super::super::support::{assert_markers, fixture_with_effects_from};

#[test]
fn cov_p2_finite_helper_names_require_one_exact_selector_edge() {
    let (source, _, heads) = fixture_with_effects_from("cov-p2-import-gate", "domains.mts");
    assert_eq!(assert_markers(&source, &heads), 12);
}

#[test]
fn cov_p2_records_require_private_results_and_fixed_local_text() {
    let (source, _, heads) = fixture_with_effects_from("cov-p2-import-gate", "record-matrix.mts");
    assert_eq!(assert_markers(&source, &heads), 13);
}
