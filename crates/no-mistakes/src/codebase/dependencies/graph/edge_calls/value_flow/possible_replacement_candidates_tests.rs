use super::*;

fn target() -> FlowValue {
    FlowValue::Target(ResolvedCallTarget::Global {
        name: "fetch".into(),
    })
}
fn candidate() -> ResolvedCallCandidate {
    ResolvedCallCandidate {
        target: KnownCallTarget::Global {
            name: "fetch".into(),
        },
        node: None,
    }
}

#[test]
fn sparse_candidate_collection_preserves_wrappers_choices_and_existing_sets() {
    let wrapped = [
        FlowValue::Possible(Box::new(target())),
        FlowValue::Provenance {
            executor: Box::new(target()),
            typed: vec![],
            members: vec![],
            receiver_factories: vec![],
        },
        FlowValue::TypedObject {
            executor: Box::new(target()),
            properties: fx_map(),
        },
        FlowValue::Returned {
            factory: vec![],
            executor: Box::new(target()),
            receiver: None,
        },
        FlowValue::Bound {
            target: Box::new(target()),
            arguments: vec![],
        },
    ];
    let mut leaf = |value| matches!(value, FlowValue::Target(_)).then(candidate);
    for value in wrapped {
        assert_eq!(collect_possible(value, &mut leaf), [Some(candidate())]);
    }
    let choice = FlowValue::Choice(vec![target(), FlowValue::Unknown]);
    assert_eq!(
        collect_possible(choice, &mut leaf),
        [Some(candidate()), None]
    );
    for has_unknown in [false, true] {
        let set = FlowValue::Target(ResolvedCallTarget::Alternatives(std::sync::Arc::new(
            ResolvedCallAlternatives::new(vec![candidate()], has_unknown),
        )));
        let values = collect_possible(set, &mut leaf);
        assert_eq!(values[0], Some(candidate()));
        assert_eq!(values.len(), 1 + usize::from(has_unknown));
    }
}

#[test]
fn sparse_candidate_storage_keeps_the_single_leaf_and_mixed_set_contracts() {
    for values in [
        ReplacementCandidates::Single(Some(candidate())),
        ReplacementCandidates::Possible(vec![Some(candidate()), None]),
    ] {
        let observed = values.into_iter().collect::<Vec<_>>();
        assert_eq!(observed[0], Some(candidate()));
        assert!(observed.len() == 1 || observed == [Some(candidate()), None]);
    }
    assert_eq!(
        ReplacementCandidates::Single(None)
            .into_iter()
            .collect::<Vec<_>>(),
        [None]
    );
}
