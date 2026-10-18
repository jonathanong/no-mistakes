use super::*;

fn collector() -> ImportCollector {
    let source = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/value-flow-mutation-alias-index/fixture/src/aliases.mts"),
    ).unwrap();
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
    let mut collector = ImportCollector {
        callable_aliases: facts
            .callable_aliases
            .into_iter()
            .map(|alias| CallableAliasBinding {
                alias,
                lexical_scope_depth: 0,
            })
            .collect(),
        value_flow: facts.value_flow,
        ..Default::default()
    };
    for (slot, binding) in collector.value_flow.bindings.iter().enumerate() {
        collector
            .value_binding_indexes
            .entry(binding.scope)
            .or_default()
            .entry(binding.name.clone())
            .or_default()
            .push(slot);
    }
    collector
}

#[test]
fn indexed_alias_invalidation_preserves_scopes_member_heads_and_earliest_cutoffs() {
    let mut full = collector();
    let mut indexed = collector();
    let index = indexed.prepare_mutation_alias_index();
    let mut affected = mutation_projection::Affected::default();
    affected.insert(0, "first");
    affected.insert(0, "namespace");
    for offset in [80, 120, 40, 40] {
        full.invalidate_mutation_bindings(&affected, None, offset, true);
        indexed.invalidate_mutation_bindings_indexed(&affected, None, offset, true, Some(&index));
    }
    assert_eq!(indexed.callable_aliases, full.callable_aliases);
    assert_eq!(indexed.value_flow.bindings, full.value_flow.bindings);
    assert_eq!(
        indexed.reassigned_callable_binding_ids,
        full.reassigned_callable_binding_ids
    );
    let changed: Vec<_> = indexed
        .callable_aliases
        .iter()
        .filter(|binding| binding.alias.invalidated_at.is_some())
        .collect();
    assert_eq!(changed.len(), 3, "{changed:#?}");
    assert!(changed.iter().all(
        |binding| binding.alias.binding_scope == 0 && binding.alias.invalidated_at == Some(40)
    ));
    assert!(indexed
        .callable_aliases
        .iter()
        .any(|binding| binding.alias.local == "first"
            && binding.alias.binding_scope != 0
            && binding.alias.invalidated_at.is_none()));
    assert!(indexed
        .callable_aliases
        .iter()
        .any(|binding| binding.alias.local == "namespaceExtra"
            && binding.alias.invalidated_at.is_none()));
    assert_eq!(indexed.value_flow.reference_callable_alias_visits, 12);
    assert_eq!(
        full.value_flow.reference_callable_alias_visits,
        4 * full.callable_aliases.len()
    );
    assert_eq!(indexed.value_flow.reference_callable_alias_index_builds, 1);
}

#[test]
fn indexed_alias_invalidation_keeps_nonmembers_and_missing_names_cold() {
    for (name, member) in [("first", false), ("missing", true)] {
        let mut full = collector();
        let mut indexed = collector();
        let index = indexed.prepare_mutation_alias_index();
        let mut affected = mutation_projection::Affected::default();
        affected.insert(0, name);
        full.invalidate_mutation_bindings(&affected, None, 40, member);
        indexed.invalidate_mutation_bindings_indexed(&affected, None, 40, member, Some(&index));
        assert_eq!(indexed.callable_aliases, full.callable_aliases);
        assert_eq!(
            indexed.reassigned_callable_binding_ids,
            full.reassigned_callable_binding_ids
        );
        assert_eq!(indexed.value_flow.reference_callable_alias_visits, 0);
        if !member {
            assert_eq!(full.value_flow.reference_callable_alias_visits, 0);
            let binding = indexed
                .value_flow
                .bindings
                .iter()
                .find(|binding| binding.scope == 0 && binding.name == "first")
                .unwrap();
            assert_eq!(binding.invalidated_at, Some(40));
            assert!(binding.mutation_contexts.contains(&None));
        }
    }
    let mut empty = ImportCollector::default();
    let index = empty.prepare_mutation_alias_index();
    assert!(index.matching(0, "missing").is_empty());
    assert_eq!(empty.value_flow.reference_callable_alias_index_builds, 0);
    assert_eq!(empty.value_flow.reference_callable_alias_index_entries, 0);
}

#[test]
fn indexed_alias_invalidation_selects_the_shadowed_lexical_head() {
    let mut full = collector();
    let mut indexed = collector();
    let scope = indexed
        .callable_aliases
        .iter()
        .find(|binding| binding.alias.local == "first" && binding.alias.binding_scope != 0)
        .unwrap()
        .alias
        .binding_scope;
    let index = indexed.prepare_mutation_alias_index();
    let mut affected = mutation_projection::Affected::default();
    affected.insert(scope, "first");
    full.invalidate_mutation_bindings(&affected, None, 40, true);
    indexed.invalidate_mutation_bindings_indexed(&affected, None, 40, true, Some(&index));
    assert_eq!(indexed.callable_aliases, full.callable_aliases);
    assert_eq!(indexed.value_flow.bindings, full.value_flow.bindings);
    let changed: Vec<_> = indexed
        .callable_aliases
        .iter()
        .filter(|binding| binding.alias.invalidated_at.is_some())
        .collect();
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].alias.binding_scope, scope);
    assert_eq!(indexed.value_flow.reference_callable_alias_visits, 1);
}
