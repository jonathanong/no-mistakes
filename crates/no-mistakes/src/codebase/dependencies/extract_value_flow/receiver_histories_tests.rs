use super::*;

#[test]
fn late_proof_demand_and_untracked_union_cannot_supply_missing_events() {
    let mut late = ReceiverHistories::default();
    let key = (0, "late".to_string());
    late.history_mut(&key)
        .0
        .record(&[Some("client".into())], 20, None);
    late.require_reference_proof(0, "late");
    assert!(!late.history(&key).unwrap().1);

    let mut linked = ReceiverHistories::default();
    let tracked = (0, "tracked".to_string());
    linked.require_reference_proof(0, "tracked");
    linked.history_mut(&tracked).0.record(&[], 10, None);
    assert!(linked.history(&tracked).unwrap().1);
    linked.link((0, "tracked"), (0, "empty"));
    assert!(linked.history(&tracked).unwrap().1);
    let unknown = (0, "unknown".to_string());
    linked.history_mut(&unknown).0.record(&[], 30, None);
    linked.link((0, "tracked"), (0, "unknown"));
    assert!(!linked.history(&tracked).unwrap().1);
    assert!(!linked.history(&unknown).unwrap().1);
}

#[test]
fn populated_late_union_preserves_exact_restoration_and_phase_evidence() {
    let source = std::fs::read_to_string(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/import-facts/fixture/nested-receiver-late-union.mts")).unwrap();
    let allocator = Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    );
    let extracted = &facts.value_flow.receiver_mutations.as_ref().unwrap()[&0];
    let mut histories = ReceiverHistories::default();
    for name in ["left", "right", "primary", "secondary"] {
        *histories.history_mut(&(0, name.to_string())).0 = (*extracted[name]).clone();
    }
    histories.link((0, "left"), (0, "right"));
    histories.link((0, "primary"), (0, "secondary"));
    let mut joined = histories.finish();
    assert_eq!(joined.len(), 2);
    for (names, history) in &mut joined {
        history.finish_early_assignments();
        if names.iter().any(|(_, name)| name == "left") {
            assert_eq!(names.len(), 2);
            let assigned = history.assigned.as_ref().unwrap();
            assert!(assigned.members.contains_key("extra"));
            assert!(assigned.members.contains_key("other"));
            assert!(assigned.dynamic.is_some());
            assert!(
                !assigned.definitely_replaced("client", source.len() as u32, None),
                "a later restoration/dynamic write must survive the union"
            );
            assert!(assigned.early.is_some());
            for callee in ["left.get)", "right.set)"] {
                let offset = source.find(callee).unwrap() as u32;
                let member = if callee.starts_with("left") {
                    "get"
                } else {
                    "set"
                };
                assert!(
                    history.assignment_invalidated(&[member], offset, None, false),
                    "the union must preserve pre-completion destructuring writes"
                );
            }
        } else {
            let assigned = history.assigned.as_ref().unwrap();
            assert!(assigned.members.contains_key("info"));
            let propagated = assigned.propagated.as_ref().unwrap();
            for member in ["get", "set"] {
                assert!(propagated.aggregate.members.contains_key(member));
                assert!(propagated.contexts[&None].members.contains_key(member));
                assert!(history.assignment_invalidated(
                    &[member],
                    source.len() as u32,
                    None,
                    false
                ));
            }
        }
    }
}
