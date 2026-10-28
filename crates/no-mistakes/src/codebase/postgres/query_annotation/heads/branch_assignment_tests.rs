use super::super::HeadProof;
use super::support::{assert_markers, fixture};

const FIXTURE: &str = "two-arm-assignment.mts";

fn proof_at(
    source: &str,
    heads: &super::super::AnnotationHeads,
    marker: &str,
    occurrence: usize,
) -> HeadProof {
    let start = source.match_indices(marker).nth(occurrence).unwrap().0 as u32;
    *heads
        .calls
        .get(&(start, start + marker.len() as u32))
        .unwrap()
}

#[test]
fn exhaustive_local_assignment_arms_preserve_only_proven_named_heads() {
    let (source, _, heads) = fixture(FIXTURE);
    assert_eq!(assert_markers(&source, &heads), 16);
    assert_eq!(proof_at(&source, &heads, "read(q)", 0), HeadProof::Literal);
    assert_eq!(
        proof_at(&source, &heads, "read(different)", 0),
        HeadProof::Literal
    );
    assert_eq!(
        proof_at(&source, &heads, "read(mixed)", 0),
        HeadProof::Unproven
    );
}

#[test]
fn eval_reference_keeps_the_assignment_census_open() {
    let (source, ts, heads) = fixture("two-arm-eval.mts");
    assert_eq!(assert_markers(&source, &heads), 1);
    assert_eq!(proof_at(&source, &heads, "read(q)", 0), HeadProof::Unproven);
    assert!(ts
        .module_bindings
        .as_ref()
        .unwrap()
        .diagnostics
        .iter()
        .any(|diagnostic| { diagnostic.message == "Dynamic eval may change binding semantics" }));
}

#[test]
fn a_missing_head_in_every_arm_is_distinct_from_mixed_or_incomplete_arms() {
    let (source, _, heads) = fixture(FIXTURE);
    assert_eq!(proof_at(&source, &heads, "read(q)", 1), HeadProof::Missing);
    assert_eq!(
        proof_at(&source, &heads, "read(mixed)", 0),
        HeadProof::Unproven
    );
    assert_eq!(
        proof_at(&source, &heads, "read(noElse)", 0),
        HeadProof::Unproven
    );
}

#[test]
fn branch_assignment_shape_and_lifetime_controls_remain_unproven() {
    let (source, _, heads) = fixture("two-arm-rejections.mts");
    assert_eq!(assert_markers(&source, &heads), 16);
    assert_eq!(
        proof_at(&source, &heads, "read(q)", 12),
        HeadProof::Unproven,
        "arguments access must invalidate the function-local lifetime proof"
    );
}
