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
