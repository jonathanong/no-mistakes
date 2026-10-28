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

#[test]
fn discarded_local_second_argument_effects_keep_factory_heads() {
    let (source, _, heads) = fixture_with_effects("second-argument-effects.mts");
    assert_eq!(assert_markers(&source, &heads), 15);
    let (source, _, heads) = fixture_with_effects("second-argument-unsafe.mts");
    assert_eq!(assert_markers(&source, &heads), 8);
    let (source, _, heads) = fixture_with_effects("second-argument-dynamic.mts");
    assert_eq!(assert_markers(&source, &heads), 1);
    let (source, _, heads) = fixture_with_effects("second-argument-duplicate.mts");
    assert_eq!(assert_markers(&source, &heads), 1);
}
