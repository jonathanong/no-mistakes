use super::*;

#[test]
fn promotion_preserves_scoped_membership_and_moves_the_first_set() {
    let mut affected = Affected::default();
    assert_eq!(affected.iter().count(), 0);
    assert!(affected.get(&0).is_none());
    affected.insert(4, "first".into());
    affected.insert(4, "second".into());
    affected.insert(4, "first".into());
    assert_eq!(affected.get(&4).unwrap().len(), 2);
    assert!(affected.get(&9).is_none());
    let original = affected.get(&4).unwrap().get("first").unwrap().as_ptr();
    affected.insert(9, "first".into());
    assert_eq!(
        affected.get(&4).unwrap().get("first").unwrap().as_ptr(),
        original
    );
    affected.insert(12, "third".into());
    affected.insert(9, "fourth".into());
    assert_eq!(affected.get(&4).unwrap().len(), 2);
    assert_eq!(affected.get(&9).unwrap().len(), 2);
    assert!(affected.get(&12).unwrap().contains("third"));
    assert!(affected.get(&99).is_none());
    let mut scopes: Vec<_> = affected.iter().map(|(scope, _)| *scope).collect();
    scopes.sort_unstable();
    assert_eq!(scopes, [4, 9, 12]);
    eprintln!(
        "Affected layout {} bytes; old scope map {} bytes",
        std::mem::size_of::<Affected>(),
        std::mem::size_of::<FxHashMap<usize, FxHashSet<String>>>()
    );
}
#[cfg(feature = "test-instrumentation")]
fn flow(source: &str) -> ValueFlowFacts {
    let allocator = Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        source,
        false,
        false,
        true,
    )
    .value_flow
}
#[cfg(feature = "test-instrumentation")]
fn receiver_source(name: &str) -> String {
    std::fs::read_to_string(
        crate::test_support::rule_fixture_root("query-reached-per-item")
            .with_file_name("provenance-fixture")
            .join("src/nested-receiver-containers")
            .join(name),
    )
    .unwrap()
}
#[test]
#[cfg(feature = "test-instrumentation")]
fn one_scope_physical_writers_need_no_outer_hash_buffers() {
    let source = receiver_source("mutable-root-scale.mts");
    let facts = flow(&source);
    assert_eq!(facts.reference_affected_scope_buffers, 0);
    let history = &facts.receiver_mutations.as_ref().unwrap()[&0]["client"];
    assert!(history.assignment_invalidated(&["get"], source.len() as u32, None, false));
    let source = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/import-facts/fixture/many-member-writes.mts"),
    )
    .unwrap();
    let facts = flow(&source);
    assert_eq!(facts.reference_affected_scope_buffers, 0);
    assert!(
        facts.receiver_mutations.as_ref().unwrap()[&0]["primary"].assignment_invalidated(
            &["field255"],
            source.len() as u32,
            None,
            false
        )
    );
}
#[test]
#[cfg(feature = "test-instrumentation")]
fn cross_scope_receiver_projection_retains_its_required_promotion() {
    let source = receiver_source("mutable-affected-cross-scope.mts");
    let facts = flow(&source);
    assert!(facts.reference_affected_scope_buffers > 0);
    let history = &facts.receiver_mutations.as_ref().unwrap()[&0]["client"];
    assert!(history.assignment_invalidated(&["get"], source.len() as u32, None, true));
    assert!(!history.assignment_invalidated(&["set"], source.len() as u32, None, true));
}
