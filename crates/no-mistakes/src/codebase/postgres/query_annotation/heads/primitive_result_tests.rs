use super::support::{assert_markers, fixture_with_effects};

#[test]
fn closed_primitive_helper_result_keeps_only_total_fixed_heads() {
    let (source, _, heads) = fixture_with_effects("primitive-result.mts");
    assert_eq!(assert_markers(&source, &heads), 15);
    let (source, _, heads) = fixture_with_effects("primitive-result-dynamic.mts");
    assert_eq!(assert_markers(&source, &heads), 1);
}
