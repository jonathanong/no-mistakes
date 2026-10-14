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
