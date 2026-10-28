use super::support::{assert_markers, fixture_with_effects};

#[test]
fn closed_captured_builder_keeps_its_named_head_in_any_closure_order() {
    let (source, _, heads) = fixture_with_effects("../captured-prefix/query.mts");
    assert_eq!(assert_markers(&source, &heads), 3);
    let (source, _, heads) = fixture_with_effects("../captured-prefix/unsafe.mts");
    assert_eq!(assert_markers(&source, &heads), 6);
    let (source, _, heads) = fixture_with_effects("../captured-prefix/shadow.mts");
    assert_eq!(assert_markers(&source, &heads), 2);
}

#[test]
fn dynamic_eval_gap_prevents_closed_capture_proof() {
    let (source, _, heads) = fixture_with_effects("../captured-prefix/gap.mts");
    assert_eq!(assert_markers(&source, &heads), 1);
}
