use super::support::{assert_markers, fixture};

#[test]
fn finite_conditional_consumer_heads_prove_every_literal_branch() {
    let (source, _, heads) = fixture("finite-conditional-consumer.mts");
    assert_eq!(assert_markers(&source, &heads), 13);
    for section in [
        "listSecurityPage",
        "deleteSecurityPage",
        "deleteOAuthPage",
        "listTopicPage",
        "atBudgetChain",
        "oldBalancedBoundary",
    ] {
        assert_proof(&source, &heads, section, super::super::HeadProof::Literal);
    }
}

#[test]
fn finite_conditional_consumer_unknown_and_transformed_heads_abstain() {
    let (source, _, heads) = fixture("finite-conditional-consumer.mts");
    for section in [
        "unknownBranch",
        "missingBranch",
        "suffixOnly",
        "binaryTransformation",
        "branchBinaryTransformation",
        "untrustedTags",
        "overBudgetChain",
    ] {
        assert_proof(&source, &heads, section, super::super::HeadProof::Unproven);
    }
}

#[test]
fn finite_conditional_consumer_legacy_candidates_share_the_same_bounded_heads() {
    let (source, ts, low) = fixture("finite-conditional-consumer.mts");
    let candidates = super::support::decisions(&source, &low)
        .into_iter()
        .map(|(span, _)| span)
        .collect::<crate::fx::FxHashSet<_>>();
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-require-query-annotation/fixture/low-heads")
        .join("finite-conditional-consumer.mts");
    let legacy = crate::ast::with_program(&path, &source, |program, _| {
        super::collect_with_helpers(
            program,
            &ts,
            candidates.iter().copied(),
            &candidates,
            false,
            false,
        )
    })
    .unwrap();
    assert_eq!(assert_markers(&source, &legacy), 13);
}

fn assert_proof(
    source: &str,
    heads: &super::super::AnnotationHeads,
    section: &str,
    expected: super::super::HeadProof,
) {
    let start = source.find(&format!("function {section}(")).unwrap();
    let call = start + source[start..].find("read(statement)").unwrap();
    assert_eq!(
        heads
            .calls
            .iter()
            .find(|(span, _)| span.0 == call as u32)
            .map(|(_, proof)| *proof),
        Some(expected),
        "{section} must preserve its exact original-AST proof"
    );
}
