use super::*;

#[test]
fn default_and_each_receiver_evidence_keep_conservative_demand() {
    assert!(ValueReplacementFacts::default().requires_receiver_proof());
    let callable_only = || ValueReplacementFacts {
        callable_only: true,
        ..Default::default()
    };
    assert!(!callable_only().requires_receiver_proof());
    // Even a caller-supplied callable-only marker cannot hide receiver evidence.
    for evidence in 0..5 {
        let mut facts = callable_only();
        match evidence {
            0 => facts.writes.push(ValueReplacementWrite {
                receiver: "client".into(),
                property: None,
                receiver_reset: true,
                owner: None,
                region: None,
                completed_at: 0,
                value: ValueExpression::Unknown,
                unconditional: false,
            }),
            1 => facts.regions.push(ValueRepeatRegion {
                start: 0,
                end: 1,
                owner: None,
                repeated: false,
                may_repeat: false,
            }),
            2 => {
                facts.calls.insert(
                    0,
                    ValueRepeatCall {
                        owner: None,
                        region: None,
                        unconditional: false,
                    },
                );
            }
            3 => facts.needs_call_proofs = true,
            _ => {
                facts.unsafe_owners.insert(CallableId(0));
            }
        }
        assert!(facts.requires_receiver_proof(), "evidence {evidence}");
    }
    let empty = ValueFlowFacts::default();
    assert!(empty.callable_assignments().is_empty());
    assert!(!empty.callable_assignment_is_used(0, "absent"));
    assert!(!empty.has_receiver_replacement_facts());
}
