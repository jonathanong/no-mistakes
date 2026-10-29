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
fn captured_paired_text_reads_need_a_closed_native_root() {
    let (source, _, heads) = super::support::fixture("captured-text-field.mts");
    assert_eq!(super::support::assert_markers(&source, &heads), 8);
}

#[test]
fn dynamic_eval_gap_prevents_closed_capture_proof() {
    let (source, _, heads) = fixture_with_effects("../captured-prefix/gap.mts");
    assert_eq!(assert_markers(&source, &heads), 1);
}
