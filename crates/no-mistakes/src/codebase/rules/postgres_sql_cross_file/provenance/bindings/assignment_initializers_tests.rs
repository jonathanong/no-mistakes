use super::assignment_scalar_tests::prepared;
use super::*;

#[test]
fn reassigned_scalar_origins_survive_conditional_writes_and_rhs_self_reads() {
    let (facts, files) = prepared(&["assignment-open.tsx", "assignment-closed.tsx"]);
    let bindings = Bindings::new(&facts).with_numeric_parameter_bounds();
    for (index, name, open, initial_open) in [
        (0, "conditionalWrite", true, true),
        (1, "conditionalWrite", false, false),
        (0, "selfValue", true, true),
        (1, "selfValue", false, false),
        (1, "restoredValue", false, false),
        (1, "replacedValue", false, true),
    ] {
        let file = &files[index];
        let flow = &facts.get_ts_facts(file).unwrap().value_flow;
        assert!(flow.bindings.iter().all(|binding| binding.name != name));
        let mut writes: Vec<_> = flow
            .identifier_writes
            .iter()
            .filter(|write| write.name == name)
            .filter_map(|write| {
                write
                    .assigned
                    .as_deref()
                    .map(|assigned| (write.scope, assigned))
            })
            .collect();
        writes.sort_by_key(|(_, assigned)| assigned.completed_at);
        let (scope, first) = writes.first().copied().unwrap();
        let (_, last) = writes.last().copied().unwrap();
        assert_eq!(
            writes
                .iter()
                .filter(|(_, assigned)| assigned.initial.is_some())
                .count(),
            1
        );
        let initial = first.initial.as_ref().unwrap();
        assert_eq!(initial.owner, first.owner);
        assert!(initial.declared_at < first.completed_at);
        assert!(writes
            .iter()
            .skip(1)
            .all(|(_, assigned)| assigned.initial.is_none()));
        let call = flow
            .calls
            .iter()
            .filter(|call| {
                call.context == last.owner
                    && call.source_callee == "read"
                    && call.offset >= last.completed_at
            })
            .min_by_key(|call| call.offset)
            .unwrap();
        assert!(bindings.value(file, scope, name, call.end).is_none());
        assert_eq!(
            bindings.flows_unbounded(file, scope, name, call.end),
            open,
            "{name}"
        );
        assert_eq!(
            bindings.flows_unbounded(file, scope, name, first.completed_at - 1),
            initial_open,
            "initial {name}"
        );
        if name == "restoredValue" {
            assert!(bindings.flows_unbounded(file, scope, name, first.completed_at));
        }
    }
}
