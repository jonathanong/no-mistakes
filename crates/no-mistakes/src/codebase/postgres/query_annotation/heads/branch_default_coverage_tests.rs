use super::super::super::{helper_heads, HeadProof};
use super::super::support::{assert_markers, fixture_with_effects};

#[test]
fn branch_owner_and_effect_default_boundaries_remain_unproven() {
    let (source, _, heads) = fixture_with_effects("branch-default-coverage.mts");
    assert_eq!(assert_markers(&source, &heads), 2);
    assert!(heads
        .calls
        .values()
        .all(|proof| *proof != HeadProof::Literal));
}
#[test]
fn imported_paired_options_require_the_builder_projection_kind() {
    let (source, _, heads) = fixture_with_effects("imported-options-coverage.mts");
    assert_eq!(assert_markers(&source, &heads), 1);
    assert!(heads
        .pending_imports
        .iter()
        .any(|pending| matches!(pending.kind, helper_heads::Kind::Builder)));
}
