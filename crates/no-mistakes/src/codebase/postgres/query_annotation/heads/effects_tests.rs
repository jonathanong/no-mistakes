use super::support::{assert_markers, fixture_with_effects};

#[test]
fn discarded_local_append_only_helpers_preserve_trusted_prefixes() {
    let (source, _, heads) = fixture_with_effects("append-effects.mts");
    assert_eq!(assert_markers(&source, &heads), 16);
}

#[test]
fn local_effect_composition_and_discarded_returned_param_keep_prefix() {
    let (source, _, heads) = fixture_with_effects("append-effect-composition.mts");
    assert_eq!(assert_markers(&source, &heads), 12);
}

#[test]
fn captured_parameter_appends_keep_only_the_fixed_prefix() {
    let (source, _, heads) = fixture_with_effects("captured-append-effects.mts");
    assert_eq!(assert_markers(&source, &heads), 13);
}

#[path = "branch_default_coverage_tests.rs"]
mod coverage_tests;
