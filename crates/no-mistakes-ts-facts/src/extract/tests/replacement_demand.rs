use super::*;

#[cfg(feature = "test-instrumentation")]
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
    #[cfg(feature = "test-instrumentation")]
    {
        assert_eq!(replacement_work(&imports_only), [0; 5]);
        assert!(replacement_work(&values)[..4]
            .iter()
            .all(|count| *count > 0));
        assert_eq!(replacement_work(&values)[4], 0);
    }
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
    #[cfg(feature = "test-instrumentation")]
    assert_eq!(facts.value_flow.replacement_loop_classifications, 5);
}

#[test]
fn ordinary_import_fixtures_skip_per_expression_replacement_guards() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    #[cfg(feature = "test-instrumentation")]
    let mut entries = Vec::new();
    for relative in [
        "test-cases/codebase-analysis/import-facts/fixture/shared-destructuring.mts",
        "fixtures/performance/react-traits/many-components.tsx",
    ] {
        let path = root.join(relative);
        let source = std::fs::read_to_string(&path).unwrap();
        let allocator = Allocator::default();
        let parsed =
            Parser::new(&allocator, &source, SourceType::from_path(&path).unwrap()).parse();
        assert!(parsed.diagnostics.is_empty(), "{relative}");
        let facts = extract_import_facts_from_program_with_source_and_demands(
            &parsed.program,
            &source,
            false,
            false,
            false,
        );
        let values = extract_import_facts_from_program_with_source_and_demands(
            &parsed.program,
            &source,
            false,
            false,
            true,
        );
        assert_eq!(facts.imports, values.imports, "{relative}");
        assert_eq!(facts.function_calls, values.function_calls, "{relative}");
        assert_eq!(
            facts.callable_scope_ids, values.callable_scope_ids,
            "{relative}"
        );
        // Count entry before the demand guard: post-guard counters hide ordinary-path work.
        #[cfg(feature = "test-instrumentation")]
        entries.push((
            relative,
            facts.value_flow.replacement_expression_guard_entries,
        ));
    }
    #[cfg(feature = "test-instrumentation")]
    assert!(entries.iter().all(|(_, count)| *count == 0), "{entries:?}");
}

#[test]
fn typed_invalidations_preserve_owner_semantics_and_ordinary_unary_walks() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/import-facts/fixture/replacement-typed-invalidations.mts");
    let source = std::fs::read_to_string(fixture).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    let extract = |demand| extract_import_facts_from_program_with_source_and_demands(
        &parsed.program, &source, false, false, demand,
    );
    let imports = extract(false);
    let values = extract(true);
    assert_eq!(imports.imports, values.imports);
    assert_eq!(imports.function_calls, values.function_calls);
    assert_eq!(imports.callable_scope_ids, values.callable_scope_ids);
    #[cfg(feature = "test-instrumentation")]
    assert_eq!(values.value_flow.replacement_expression_checks, 3);
    let capsule = values.value_flow.replacements.as_ref().unwrap();
    assert_eq!(capsule.writes.len(), 4);
    for (name, invalidated) in [
        ("awaited", true),
        ("yielded", true),
        ("deleted", true),
        ("ordinary", false),
    ] {
        let (owner, _) = values
            .callable_scope_ids
            .iter()
            .find(|(_, scope)| scope == name)
            .unwrap();
        let write = capsule.writes.iter().find(|write| write.owner == Some(*owner)).unwrap();
        assert_eq!(write.unconditional, !invalidated, "{name}");
        // Cold await precedes capsule creation; warm yield/delete also record unsafe ownership.
        assert_eq!(capsule.unsafe_owners.contains(owner), invalidated && name != "awaited", "{name}");
    }
}

#[path = "callable_assignment_targets.rs"]
mod callable_assignment_targets;

#[cfg(feature = "test-instrumentation")]
#[path = "statement_outline.rs"]
mod statement_outline;

// Default-feature compilation is what this suite protects. The module is absent
// when counters exist, so it cannot name instrumentation-only fields.
#[cfg(not(feature = "test-instrumentation"))]
#[path = "default_feature_instrumentation.rs"]
mod default_feature_instrumentation;
