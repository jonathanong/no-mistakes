use super::*;

#[test]
fn replacement_cutoffs_keep_possible_and_guaranteed_offsets_separate() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/query-reached-per-item/provenance-fixture/src/nested-receiver-cutoffs.mts");
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
    let histories = facts.value_flow.receiver_mutations.as_ref().unwrap();
    let boxes = facts
        .value_flow
        .bindings
        .iter()
        .filter(|binding| binding.name == "box")
        .collect::<Vec<_>>();
    assert_eq!(boxes.len(), 9);
    for (index, binding) in boxes.iter().enumerate() {
        let write = &histories[&binding.scope]["box"]
            .assigned
            .as_ref()
            .unwrap()
            .members["client"];
        assert_eq!(
            write.unconditional_offset.is_some(),
            index == 8,
            "box {index}: {write:?}"
        );
        if let Some(guaranteed) = write.unconditional_offset {
            assert!(guaranteed.get() > write.offset);
        }
    }
    let restored = histories[&0]["restoredBox"]
        .assigned
        .as_ref()
        .unwrap()
        .replacements
        .as_ref()
        .unwrap();
    assert_eq!(restored.members["client"].len(), 2);
    assert_eq!(restored.dynamic.len(), 0);
    let dynamic = histories[&0]["dynamicBox"]
        .assigned
        .as_ref()
        .unwrap()
        .replacements
        .as_ref()
        .unwrap();
    assert_eq!(dynamic.members["client"].len(), 1);
    assert_eq!(dynamic.dynamic.len(), 1);
    let parameter = &histories[&0]["parameterBox"]
        .assigned
        .as_ref()
        .unwrap()
        .members["client"];
    assert!(parameter.unconditional_offset.is_none());
    for name in [
        "forBox",
        "whileBox",
        "doBox",
        "arrayBox",
        "tryBox",
        "labelBox",
        "logicalAndBox",
        "logicalOrBox",
        "logicalNullBox",
    ] {
        let write = &histories[&0][name].assigned.as_ref().unwrap().members["client"];
        assert!(write.unconditional_offset.is_none(), "{name}: {write:?}");
    }
    let acquired = &histories[&0]["acquiredBox"]
        .assigned
        .as_ref()
        .unwrap()
        .members["client"];
    assert!(acquired.unconditional_offset.unwrap().get() > acquired.offset);
}

#[test]
fn converging_default_paths_join_cutoffs_instead_of_multiplying_states() {
    let source =
        std::fs::read_to_string(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-cases/codebase-analysis/import-facts/fixture/nested-receiver-branching.mts",
        ))
        .unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    );
    assert!(facts.value_flow.reference_path_visits > 12);
    assert!(
        facts.value_flow.reference_path_visits <= 12 * 8 + 8,
        "converging defaults must not create exponential route identities: {}",
        facts.value_flow.reference_path_visits
    );
}

#[test]
fn deferred_receiver_certainty_belongs_to_the_callable() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/import-facts/fixture/nested-receiver-deferred.mts",
    );
    let source = std::fs::read_to_string(fixture).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    );
    let histories = facts.value_flow.receiver_mutations.as_ref().unwrap();
    for (name, guaranteed) in [
        ("labeledBox", true),
        ("conditionalBox", true),
        ("caughtBox", true),
        ("logicalBox", true),
        ("guardedBox", false),
        ("defaultBox", false),
    ] {
        let write = &histories[&0][name].assigned.as_ref().unwrap().members["client"];
        assert_eq!(
            write.unconditional_offset.is_some(),
            guaranteed,
            "{name}: {write:?}"
        );
    }
}
