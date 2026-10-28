use super::support::{assert_markers, fixture};

#[test]
fn finite_named_conditional_heads_keep_tag_lifetimes_and_unknown_arms() {
    let (source, _, heads) = fixture("conditional-return-local.mts");
    assert_eq!(assert_markers(&source, &heads), 19);
    let source_after_escape = source.find("read(statement); // low: false").unwrap() as u32;
    assert!(!heads.calls.iter().any(|(span, proof)| {
        span.0 == source_after_escape && *proof == super::super::HeadProof::Literal
    }));
    for section in [
        "missingBranch",
        "partialBranch",
        "unknownBranch",
        "transactionBoundary",
    ] {
        let start = source.find(&format!("function {section}")).unwrap();
        let call = start + source[start..].find("read(statement)").unwrap();
        assert_eq!(
            heads
                .calls
                .iter()
                .find(|(span, _)| span.0 == call as u32)
                .map(|(_, proof)| *proof),
            Some(super::super::HeadProof::Unproven),
            "{section} must retain advisory uncertainty"
        );
    }
}
