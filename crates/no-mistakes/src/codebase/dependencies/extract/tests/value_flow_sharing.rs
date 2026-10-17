use super::*;
use oxc_parser::Parser;

#[test]
fn destructured_value_bindings_share_one_initializer() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/import-facts/fixture/shared-destructuring.mts");
    let source = std::fs::read_to_string(fixture).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    let import_only = extract_import_facts_from_program(&parsed.program);
    assert!(import_only.value_flow.bindings.is_empty());
    assert!(import_only.value_flow.parameters.is_empty());
    assert!(import_only.value_flow.calls.is_empty());
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    );
    #[cfg(feature = "test-instrumentation")]
    assert_eq!(facts.value_flow.reference_link_constructions, 0);
    assert!(facts.value_flow.receiver_mutations.is_none());
    let bases = facts
        .value_flow
        .bindings
        .iter()
        .filter_map(|binding| match &binding.value {
            ValueExpression::Member(base, _) => Some(base),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(bases.len(), 128);
    assert!(bases
        .iter()
        .all(|base| std::sync::Arc::ptr_eq(bases[0], base)));
    assert!(
        matches!(bases[0].as_ref(), ValueExpression::Object(fields, false) if fields.len() == 128)
    );
}

#[cfg(feature = "test-instrumentation")]
#[test]
fn member_writes_do_not_rebuild_reference_links() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/import-facts/fixture/many-member-writes.mts");
    let source = std::fs::read_to_string(fixture).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    );
    assert_eq!(facts.value_flow.reference_link_constructions, 515);
    assert_no_mutable_root_work(&facts.value_flow);
    assert_eq!(facts.value_flow.reference_component_links, 3);
    assert_eq!(facts.value_flow.reference_link_visits, 2 * 2 * 256 + 2 * 3);
    assert_eq!(facts.value_flow.reference_rhs_barriers, 0);
    assert_eq!(facts.value_flow.reference_rhs_destinations, 0);
    assert!(facts.value_flow.reference_component_root_steps <= 2 * 257 + 3 * 3 + 4 * 2);
    let assigned = facts.value_flow.receiver_mutations.as_ref().unwrap()[&0]["primary"]
        .assigned
        .as_ref()
        .unwrap();
    assert_eq!(assigned.members.len(), 257);
    assert!(assigned.propagated.is_none());
    assert!(
        assigned.replacements.is_none(),
        "ordinary direct writes retain no reference event index"
    );
    assert_eq!(assigned.merge_reinsertions, 0);
    for name in ["client", "alias", "late"] {
        let binding = facts
            .value_flow
            .bindings
            .iter()
            .find(|binding| binding.name == name)
            .unwrap();
        assert!(binding.invalidated_at.is_some());
    }
    assert!(facts
        .value_flow
        .bindings
        .iter()
        .filter(|binding| binding.name.starts_with("unused"))
        .all(|binding| binding.invalidated_at.is_none()));
}

#[test]
fn default_exports_extend_an_already_initialized_reference_index() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/import-facts/fixture/mutated-default-export.mts");
    let source = std::fs::read_to_string(fixture).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    );
    let exported = facts
        .value_flow
        .bindings
        .iter()
        .find(|binding| binding.name == "default")
        .unwrap();
    assert!(exported
        .invalidated_at
        .is_some_and(|offset| offset > exported.declared_at));
    assert!(exported.mutation_contexts.contains(&None));
}

#[test]
fn late_defaults_share_one_indexed_mutation_history() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/import-facts/fixture/many-late-defaults.mts");
    let source = std::fs::read_to_string(fixture).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    );
    assert_eq!(facts.value_flow.parameters.len(), 512);
    // Later writes retain the same earliest offset instead of rewriting every alias.
    #[cfg(feature = "test-instrumentation")]
    assert_eq!(facts.value_flow.reference_invalidation_stores, 512);
    // Direct aliases have no projected value; eligibility must not cause a second scan.
    #[cfg(feature = "test-instrumentation")]
    assert_eq!(facts.value_flow.reference_binding_visits, 256 * 512);
    #[cfg(feature = "test-instrumentation")]
    assert_no_mutable_root_work(&facts.value_flow);
    #[cfg(feature = "test-instrumentation")]
    {
        assert_eq!(facts.value_flow.reference_rhs_barriers, 0);
        assert_eq!(facts.value_flow.reference_rhs_destinations, 0);
    }

    let histories = facts
        .value_flow
        .parameters
        .iter()
        .map(|parameter| {
            let sources = parameter
                .default_mutations
                .as_ref()
                .unwrap()
                .get(&0)
                .unwrap();
            assert_eq!(sources.len(), 1);
            assert!(sources[0].path.is_empty());
            &sources[0].history
        })
        .collect::<Vec<_>>();
    assert!(histories
        .iter()
        .all(|history| std::sync::Arc::ptr_eq(histories[0], history)));
    let history = histories[0];
    let receivers = facts.value_flow.receiver_mutations.as_ref().unwrap();
    let receiver_histories = receivers
        .values()
        .flat_map(|names| names.values())
        .collect::<Vec<_>>();
    // Inline-call analysis retains every alias, but all names share one history.
    assert_eq!(receiver_histories.len(), 513);
    assert!(receiver_histories
        .iter()
        .all(|receiver| std::sync::Arc::ptr_eq(history, receiver)));
    let cloned = facts.value_flow.clone();
    assert!(std::sync::Arc::ptr_eq(
        receivers,
        cloned.receiver_mutations.as_ref().unwrap()
    ));
    assert_eq!(history.members.len(), 512);
    assert!(history
        .members
        .values()
        .all(|child| child.members.is_empty()));
    assert!(history.invalidated(&[], &[], u32::MAX, None, false));
    assert!(history.invalidated(&[], &["p0".into()], u32::MAX, None, false));
    assert!(!history.invalidated(&[], &["missing".into()], u32::MAX, None, false));
    // The method value copied before its receiver was written stays valid.
    assert!(!history.invalidated(
        &[Some("p0".into())],
        &["method".into()],
        u32::MAX,
        None,
        false
    ));
}

#[test]
fn late_default_exports_capture_an_already_mutated_receiver() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/import-facts/fixture/late-mutated-default-export.mts",
    );
    let source = std::fs::read_to_string(fixture).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    );
    let export = facts
        .value_flow
        .bindings
        .iter()
        .find(|binding| binding.name == "default")
        .unwrap();
    assert!(export
        .invalidated_at
        .is_some_and(|offset| offset < export.declared_at));
    assert!(export.mutation_contexts.contains(&None));
}

#[path = "assigned_writer_contexts.rs"]
mod assigned_writer_contexts;

#[test]
fn nested_writes_retain_exact_assignments_in_shared_receiver_histories() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/import-facts/fixture/nested-receiver-writes.mts");
    let source = std::fs::read_to_string(fixture).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    );
    let receivers = facts.value_flow.receiver_mutations.as_ref().unwrap();
    let primary = &receivers[&0]["primary"];
    assert!(std::sync::Arc::ptr_eq(primary, &receivers[&0]["receiver"]));
    assert!(std::sync::Arc::ptr_eq(
        &receivers[&0]["box"],
        &receivers[&0]["alias"]
    ));
    let assigned = &primary
        .assigned
        .as_ref()
        .unwrap()
        .propagated
        .as_ref()
        .unwrap()
        .aggregate();
    assert_eq!(assigned.members.len(), 2);
    assert!(assigned.members.contains_key("get"));
    assert!(assigned.members.contains_key("info"));
    let contexts = &primary
        .assigned
        .as_ref()
        .unwrap()
        .propagated
        .as_ref()
        .unwrap()
        .context(None)
        .unwrap();
    assert_eq!(contexts.members.len(), 2);
    assert_eq!(contexts.members, assigned.members);
    assert!(primary.members["nested"]
        .assigned
        .as_ref()
        .unwrap()
        .propagated
        .as_ref()
        .unwrap()
        .aggregate()
        .members
        .contains_key("get"));
    let defaults = facts.value_flow.parameters[0]
        .default_mutations
        .as_ref()
        .unwrap();
    assert!(std::sync::Arc::ptr_eq(primary, &defaults[&0][0].history));
    assert!(!receivers[&0].contains_key("session"));
    #[cfg(feature = "test-instrumentation")]
    {
        assert_eq!(
            facts.value_flow.reference_link_constructions,
            facts.value_flow.bindings.len()
        );
        assert_eq!(facts.value_flow.reference_component_links, 2);
        assert!(facts.value_flow.reference_component_root_steps <= 64);
    }
}

#[path = "receiver_cutoffs.rs"]
mod receiver_cutoffs;

#[cfg(feature = "test-instrumentation")]
#[test]
fn nested_writes_bound_existing_traversal_and_canonical_lookups() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/import-facts/fixture/nested-receiver-write-sharing.mts",
    );
    let source = std::fs::read_to_string(fixture).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    );
    assert_eq!(facts.value_flow.reference_link_constructions, 257);
    assert_eq!(facts.value_flow.reference_component_links, 128);
    // The existing propagation still visits each undirected alias edge per write.
    // Ancestor checks add only root lookups, not a second component traversal.
    assert_eq!(facts.value_flow.reference_link_visits, 128 * 2 * 128);
    assert!(facts.value_flow.reference_component_root_steps <= 128 * 3 + 128 * 5 + 130 * 2);
    let receivers = facts.value_flow.receiver_mutations.as_ref().unwrap();
    let primary = &receivers[&0]["primary"];
    assert_eq!(
        primary
            .assigned
            .as_ref()
            .unwrap()
            .propagated
            .as_ref()
            .unwrap()
            .aggregate()
            .members
            .len(),
        128
    );
    for name in (0..128).map(|i| format!("alias{i}")) {
        assert!(std::sync::Arc::ptr_eq(
            &receivers[&0]["box"],
            &receivers[&0][&name]
        ));
    }
    let defaults = facts.value_flow.parameters[0]
        .default_mutations
        .as_ref()
        .unwrap();
    assert!(std::sync::Arc::ptr_eq(primary, &defaults[&0][0].history));
}

#[cfg(feature = "test-instrumentation")]
fn assert_no_mutable_root_work(facts: &ValueFlowFacts) {
    // Ordinary aliases retain receiver-read timing without mutable-root coordinates.
    type Receiver = ((usize, String), Vec<Option<String>>);
    type Read = Option<(u32, Option<CallableId>)>;
    let entry_bytes = std::mem::size_of::<((Receiver, Vec<Read>, ()), bool, bool, bool)>();
    assert!(facts.reference_pending_capacity > 0);
    assert_eq!(facts.reference_version_dispatches, 0);
    assert_eq!(facts.receiver_only_pending_spills, 0);
    assert_eq!(
        (facts.reference_root_probes, facts.reference_pending_bytes),
        (0, facts.reference_pending_capacity * entry_bytes),
    );
}

// These counters exist only with test-instrumentation; an ungated module does not compile.
#[cfg(feature = "test-instrumentation")]
#[path = "replacement_demand.rs"]
mod replacement_demand;
