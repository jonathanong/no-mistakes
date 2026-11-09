use super::super::proven;
use super::{callback_values, proof_source, proven_facts};

#[test]
fn receiver_gap_shapes_stay_nonbuiltin_while_const_arrays_remain_native() {
    let source = proof_source("proof-receiver-gaps.mts");
    let facts = proven_facts(&source);
    for call in [
        "(true).toFixed",
        "constructed.toFixed",
        "sink.join",
        "qualified.toFixed",
        "mixed.toFixed",
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
    assert_eq!(
        callback_values(&source, "rows.forEach", &facts),
        vec![Some(vec![0])]
    );
    assert_eq!(callback_values(&source, "alias.join", &facts), vec![None]);
}

#[test]
fn contextual_primitive_proofs_need_explicit_source_types() {
    let source = proof_source("proof-primitives.mts");
    let found = proven(&source);
    assert_eq!(
        found
            .iter()
            .filter(|call| call.as_str() == "r.toFixed")
            .count(),
        2,
        "{found:?}"
    );
    assert!(
        found.iter().any(|call| call == "expectation.function.trim"),
        "{found:?}"
    );
    assert!(
        found.iter().any(|call| call == "dayAnchorMs.toString"),
        "{found:?}"
    );
    assert!(
        found.iter().any(|call| call == "dayAnchorText.padStart"),
        "{found:?}"
    );
    assert!(!found.iter().any(|call| call == "r.trim"), "{found:?}");
}
