use super::*;

fn outer_positions(
    source: &str,
    call: &str,
    facts: &super::super::super::ProvenCalls,
) -> Option<Option<Vec<usize>>> {
    let start = source.find(call)? as u32;
    facts
        .callbacks
        .iter()
        .filter(|((offset, _), _)| *offset == start)
        .max_by_key(|((_, end), _)| *end)
        .map(|(_, positions)| positions.map(|positions| positions.to_vec()))
}

#[test]
fn sparse_holes_and_mutated_empty_arrays_keep_callback_proof_sound() {
    let source = proof_source("proof-array-cardinality.mts");
    let facts = proven_facts(&source);
    for call in [
        "stableEmpty.forEach",
        "Array.from(stableEmpty",
        "stableAlias.findLastIndex",
        "stableCopy.find",
        "sparse.map",
        "emptySparseSlice.find",
        "emptySparseSplice.find",
        "emptyNoArgSplice.find",
        "distantSlice.find",
        "farNegativeEnd.find",
        "unaryPlusSlice.find",
        "negativeDelete.find",
        "zeroUnknownSlice.find",
        "zeroUnknownSplice.find",
    ] {
        assert_eq!(
            outer_positions(&source, call, &facts),
            Some(Some(vec![])),
            "{call}"
        );
    }
    for call in [
        "sparse.find(",
        "sparse.findIndex",
        "sparse.findLast(",
        "sparse.findLastIndex",
        "Array.from(sparse",
        "[,,].find(",
        "[,,].findLastIndex",
        "Array.from([,,]",
        "holeSlice.find",
        "negativeSlice.find",
        "farNegativeSlice.find",
        "fractionalSlice.find",
        "omittedSlice.find",
        "omittedDelete.find",
        "largeDelete.find",
        "typedFromCall.find",
        "(produce() as number[]).map",
    ] {
        let expected = if call.starts_with("Array.from") { 1 } else { 0 };
        assert_eq!(
            outer_positions(&source, call, &facts),
            Some(Some(vec![expected])),
            "{call}"
        );
    }
    for call in [
        "changed.forEach",
        "(changed as number[]).forEach",
        "changed!.find",
        "Array.from(changed",
        "aliasSource.forEach",
        "escaped.forEach",
        "derived.forEach",
        "mutatedHoles.find",
        "unknownStart.find",
        "unknownCount.find",
        "infiniteStart.find",
        "unknownUnary.find",
        "unsupportedUnary.find",
        "Array.from(uncertainSparse",
    ] {
        assert_eq!(outer_positions(&source, call, &facts), None, "{call}");
    }
}

#[test]
fn conditional_array_aliases_and_callback_receivers_propagate_instability() {
    let source = proof_source("review-array-aliases.mts");
    let facts = proven_facts(&source);
    for call in [
        "selected.forEach",
        "sourceTarget.forEach",
        "holes.map",
        "namedHoles.map",
        "escapingHoles.map",
        "writtenLengthHoles.map",
        "updatedLengthHoles.map",
        "capturedHoles.map",
        "restHoles.map",
        "argumentsHoles.map",
        "destructuredHoles.map",
    ] {
        assert_eq!(outer_positions(&source, call, &facts), None, "{call}");
    }
    for call in [
        "fresh.forEach",
        "stableChoice.forEach",
        "zero.find",
        "zero.map",
        "skipped.map",
        "pureHoles.map",
        "pureNamedHoles.map",
        "sharedPure.map",
        "readOnlyHoles.map",
        "computedLengthHoles.map",
        "shadowedHoles.map",
        "shadowedArrowHoles.map",
        "unusedRest.map",
        "unusedDestructured.map",
    ] {
        assert_eq!(
            outer_positions(&source, call, &facts),
            Some(Some(vec![])),
            "{call}"
        );
    }
    for call in [
        "pureHoles.find",
        "pureNamedHoles.find",
        "sharedPure.find",
        "readOnlyHoles.find",
        "computedLengthHoles.find",
        "shadowedHoles.find",
        "shadowedArrowHoles.find",
        "unusedRest.find",
        "unusedDestructured.find",
        "importedHoles.find",
        "unknownHoles.find",
    ] {
        assert_eq!(
            outer_positions(&source, call, &facts),
            Some(Some(vec![0])),
            "{call}"
        );
    }
}

#[cfg(feature = "test-instrumentation")]
#[test]
fn array_callback_summaries_skip_unrelated_bodies_and_empty_candidates() {
    let source = proof_source("review-array-aliases.mts");
    let _ = proven_facts(&source);
    assert_eq!(
        super::super::super::array_escapes::tests::callback_summaries(),
        3
    );
    let source = proof_source("review-no-array-candidates.mts");
    let _ = proven_facts(&source);
    assert_eq!(
        super::super::super::array_escapes::tests::callback_summaries(),
        0
    );
}
