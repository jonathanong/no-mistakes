use super::*;

fn replacement_work(facts: &ImportFacts) -> [usize; 5] {
    let flow = &facts.value_flow;
    [
        flow.replacement_statement_checks,
        flow.replacement_expression_checks,
        flow.replacement_region_frames,
        flow.replacement_loop_classifications,
        flow.callable_assignment_name_clones,
    ]
}

#[test]
fn imports_only_skip_replacement_bookkeeping_but_keep_import_and_scope_facts() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/import-facts/fixture/replacement-demand.mts");
    let source = std::fs::read_to_string(fixture).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    let extract = |value_flow| {
        extract_import_facts_from_program_with_source_and_demands(
            &parsed.program,
            &source,
            false,
            false,
            value_flow,
        )
    };
    let imports_only = extract(false);
    let values = extract(true);
    // These work counters observe the actual hot-path prologues, not output size.
    assert_eq!(replacement_work(&imports_only), [0; 5]);
    assert!(replacement_work(&values)[..4]
        .iter()
        .all(|count| *count > 0));
    assert_eq!(replacement_work(&values)[4], 0);
    assert_eq!(imports_only.imports, values.imports);
    assert_eq!(imports_only.function_calls, values.function_calls);
    assert_eq!(imports_only.callable_scope_ids, values.callable_scope_ids);
    assert_eq!(imports_only.exported_functions, values.exported_functions);
    assert!(imports_only.value_flow.bindings.is_empty());
    assert!(!values.value_flow.bindings.is_empty());
    assert!(imports_only
        .imports
        .iter()
        .any(|import| import.specifier == "./lazy.mts"));
}

fn saved_assignment_facts(name: &str, value_flow: bool) -> ImportFacts {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/import-facts/fixture")
        .join(name);
    let source = std::fs::read_to_string(fixture).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    let mut collector = ImportCollector {
        collect_value_flow: value_flow,
        ..Default::default()
    };
    collector.local_stack.push(fx_set());
    collector.lexical_scope_ids.push(0);
    predeclare_program_value_bindings(&mut collector, &parsed.program);
    assert_eq!(collector.callable_assignment_state.is_some(), value_flow);
    extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        value_flow,
    )
}

#[test]
fn discarded_noncallable_staging_keeps_capsule_absent() {
    for values in [false, true] {
        let facts = saved_assignment_facts("discarded-callable-staging.mts", values);
        assert!(facts.value_flow.replacements.is_none());
        assert!(facts.value_flow.callable_assignments().is_empty());
    }
}

#[test]
fn receiver_and_callable_facts_share_conservative_capsule() {
    let facts = saved_assignment_facts("mixed-callable-receiver.mts", true);
    let capsule = facts.value_flow.replacements.as_ref().unwrap();
    assert!(!capsule.callable_only);
    assert!(capsule.requires_receiver_proof());
    assert!(!capsule.writes.is_empty());
    assert_eq!(facts.value_flow.callable_assignments().len(), 1);
    assert!(facts.value_flow.callable_assignment_is_used(0, "callback"));
}

#[test]
fn for_of_callable_proofs_keep_opaque_and_noncallable_fallbacks_conservative() {
    let facts = saved_assignment_facts("replacement-loop-fallbacks.mts", true);
    let assignments = facts.value_flow.callable_assignments();
    for (name, guaranteed) in [("fromNonliteral", false), ("fromHole", false), ("accepted", true)] {
        let assignment = assignments.iter().find(|assignment| assignment.name == name).unwrap();
        assert_eq!(assignment.guaranteed, guaranteed, "{name}");
    }
    assert!(!assignments.iter().any(|assignment| assignment.name == "scalar"));
    let capsule = facts.value_flow.replacements.as_ref().unwrap();
    assert!(capsule.requires_receiver_proof());
    assert!(!capsule.writes.is_empty());
    assert_eq!(facts.value_flow.replacement_loop_classifications, 5);
}
