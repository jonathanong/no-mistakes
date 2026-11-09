use super::*;

#[test]
fn json_callbacks_require_global_json_receiver() {
    let source = proof_source("proof-json-reviver.mts");
    let facts = proven_facts(&source);
    let positions = |callee: &str| {
        source
            .match_indices(callee)
            .map(|(offset, _)| {
                facts
                    .callbacks
                    .iter()
                    .find(|((start, _), _)| *start == offset as u32)
                    .map(|(_, positions)| positions.map(|positions| positions.to_vec()))
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        positions("JSON.parse"),
        [Some(Some(vec![1])), Some(Some(vec![])), Some(Some(vec![1]))]
    );
    assert_eq!(
        positions("JSON.stringify"),
        [Some(Some(vec![1])), Some(Some(vec![])), Some(Some(vec![1]))]
    );
    let shadowed = proof_source("proof-shadowed-json.mts");
    assert!(proven_facts(&shadowed).callbacks.is_empty());
}
