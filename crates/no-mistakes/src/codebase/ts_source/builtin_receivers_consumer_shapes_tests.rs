use super::*;

#[test]
fn leaf_fallback_excludes_known_custom_and_wrong_receivers() {
    let source = proof_source("consumer-shapes.ts");
    let facts = proven_facts(&source);
    for call in [
        "textValue.toFixed",
        "sinkValue.toFixed",
        "text.toFixed",
        "casted.join",
        "(base as Sink).join",
        "(<Sink>base).join",
        "({ join: (_: string) => '' } satisfies Sink).join",
    ] {
        let start = source.find(call).unwrap() as u32;
        assert!(
            facts
                .known_nonbuiltin
                .iter()
                .any(|(offset, _)| *offset == start),
            "{call}"
        );
    }
    for call in [
        "numberLike.toFixed",
        "joinable.join",
        "hasValue.has",
        "input.toFixed",
        "inputAny.toFixed",
        "computed.toFixed",
        "candidate.find",
        "(base as unknown).join",
        "(base as any).join",
        "castedUnknown.join",
    ] {
        let start = source.find(call).unwrap() as u32;
        assert!(
            !facts
                .known_nonbuiltin
                .iter()
                .any(|(offset, _)| *offset == start),
            "{call}"
        );
    }
    assert_eq!(
        callback_values(&source, "ownerRows.find", &facts),
        vec![Some(vec![0])]
    );
    assert_eq!(
        callback_values(&source, "plainRows.find", &facts),
        vec![Some(vec![0])]
    );
    assert_eq!(
        callback_values(&source, "membership.has", &facts),
        vec![None]
    );
}

#[test]
fn const_array_assertion_preserves_native_callback_and_leaf_proofs() {
    let source = proof_source("review-generic-shapes.mts");
    let facts = proven_facts(&source);
    assert_eq!(
        callback_values(&source, "PARTITIONS.find", &facts),
        vec![Some(vec![0])]
    );
    assert_eq!(
        callback_values(&source, "PARTITIONS.join", &facts),
        vec![None]
    );
}
