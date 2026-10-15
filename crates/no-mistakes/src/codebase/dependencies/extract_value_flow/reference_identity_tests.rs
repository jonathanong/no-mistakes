use super::*;

fn saved_history(case: &str, name: &str) -> (String, ValueMutationHistory) {
    let source = std::fs::read_to_string(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/query-reached-per-item/provenance-fixture/src/nested-receiver-containers")
        .join(format!("{case}.mts"))).unwrap();
    let allocator = Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    );
    let history = (*facts.value_flow.receiver_mutations.as_ref().unwrap()[&0][name]).clone();
    (source, history)
}

#[test]
fn deep_saved_capture_proves_detachment_with_two_borrowed_path_walks() {
    let (source, history) = saved_history("global-capture-chain", "a0");
    let mut path = vec![Some("x".into()); 32];
    path.push(Some("client".into()));
    let reads = vec![Some((source.len() as u32, None)); path.len()];
    let effects = fresh_receiver_effects::FreshReceiverEffects::default();
    assert!(reference_replaced(
        &history,
        &path,
        &reads,
        None,
        Some(&effects),
        None
    ));
    #[cfg(feature = "test-instrumentation")]
    assert_eq!(effects.identity_visits(), 64);
    let mut uncertain = reads.clone();
    uncertain[0] = None;
    assert!(!reference_replaced(
        &history, &path, &uncertain, None, None, None
    ));
    let mut uncertain = reads.clone();
    *uncertain.last_mut().unwrap() = None;
    assert!(!reference_replaced(
        &history, &path, &uncertain, None, None, None
    ));
    let mut dynamic = path.clone();
    dynamic[0] = None;
    assert!(!reference_replaced(
        &history, &dynamic, &reads, None, None, None
    ));
    let mut absent = path.clone();
    absent[0] = Some("absent".into());
    assert!(!reference_replaced(
        &history, &absent, &reads, None, None, None
    ));
    assert!(!reference_replaced(
        &history,
        &path,
        &reads[1..],
        None,
        None,
        None
    ));
    let mut other_epoch = fresh_receiver_effects::FreshReceiverEffects::default();
    other_epoch.epoch = 1;
    assert!(!reference_replaced(
        &history,
        &path,
        &reads,
        None,
        Some(&other_epoch),
        None
    ));
}

#[test]
fn canonical_history_adoption_keeps_captures_but_conflicting_lineage_cannot_sever() {
    let (source, history) = saved_history("replaced-terminal", "box");
    let path = [Some("wrapper".into()), Some("client".into())];
    let reads = [Some((source.len() as u32, None)); 2];
    let query = |history: &ValueMutationHistory| {
        reference_replaced(history, &path, &reads, None, None, None)
    };
    assert!(query(&history));
    let mut adopted = ValueMutationHistory::default();
    adopted.merge(history.clone());
    assert!(query(&adopted));
    // Merge into existing assignment storage as canonical receiver unions do.
    let mut empty_assigned = ValueMutationHistory::default();
    empty_assigned
        .members
        .entry("wrapper".into())
        .or_default()
        .assigned = Some(Default::default());
    empty_assigned.merge(history.clone());
    assert!(query(&empty_assigned));
    let mut conflicting = history.clone();
    let assigned = conflicting
        .members
        .get_mut("wrapper")
        .unwrap()
        .assigned
        .as_mut()
        .unwrap();
    let identity = assigned
        .replacements
        .as_mut()
        .unwrap()
        .identity
        .as_mut()
        .unwrap();
    let capture = identity
        .captures
        .values_mut()
        .find_map(Option::as_mut)
        .unwrap();
    capture.epoch += 1;
    adopted.merge(conflicting);
    assert!(!query(&adopted));
    // A foreign dynamic writer cannot inherit a same-context exact fresh proof.
    let mut dynamic = history.clone();
    dynamic
        .members
        .get_mut("wrapper")
        .unwrap()
        .assigned
        .as_mut()
        .unwrap()
        .dynamic = Some(mutation_history::ValueAssignedMutation {
        offset: 1,
        unconditional_offset: None,
        writer: ValueAssignedWriter::Callable(CallableId(77)),
    });
    assert!(!query(&dynamic));
    assert!(query(&history));
}

#[test]
fn missing_ancestor_timeline_never_invents_order_for_coarse_or_foreign_writes() {
    let mut assigned = ValueAssignedMutations::default();
    let read = |offset| Some((offset, None));
    assert!(!assigned.identity_changed("box", read(1), read(2)));
    assert!(assigned.identity_changed("box", None, read(2)));
    assigned.record(
        Some("box".into()),
        mutation_history::ValueAssignedMutation {
            offset: 5,
            unconditional_offset: None,
            writer: ValueAssignedWriter::Module,
        },
    );
    assert!(assigned.identity_changed("box", read(1), read(2)));
    assigned.members.get_mut("box").unwrap().writer = ValueAssignedWriter::Mixed;
    assert!(assigned.identity_changed("box", read(10), read(11)));
    assigned.replacements = Some(Default::default());
    assigned
        .replacements
        .as_mut()
        .unwrap()
        .identity
        .get_or_insert_with(Default::default)
        .unknown
        .record(20, ValueAssignedWriter::Module);
    assert!(assigned.identity_changed("box", read(19), read(20)));
}
