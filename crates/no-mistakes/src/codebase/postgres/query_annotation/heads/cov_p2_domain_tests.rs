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

#[test]
fn cov_p2_finite_domain_owners_and_index_demands_remain_exact() {
    for (name, count) in [
        ("finite-extra.mts", 12),
        ("finite-unused-root.mts", 1),
        ("finite-unrelated-read.mts", 2),
        ("finite-no-demand.mts", 1),
        ("fresh-budget.mts", 2),
    ] {
        let (source, _, heads) = fixture_with_effects_from("cov-p2-import-gate", name);
        assert_eq!(assert_markers(&source, &heads), count, "{name}");
    }
}
