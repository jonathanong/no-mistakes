use super::*;

#[test]
fn module_provider_facts_retain_both_targets_and_physical_writer_coordinates() {
    let path = crate::test_support::rule_fixture_root("query-reached-per-item")
        .with_file_name("provenance-fixture")
        .join("src/module-writer-rebind-before.mts");
    let source = std::fs::read_to_string(path).unwrap();
    let allocator = Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, &source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    );
    let histories = &facts.value_flow.receiver_mutations.as_ref().unwrap()[&0];
    let phase = source.find("write();").unwrap() as u32;
    for (name, expected) in [("primary", false), ("replica", true)] {
        let history = &histories[name];
        assert!(history.has_assignments, "{name}: {history:#?}");
        assert_eq!(
            history.assignment_invalidated_guarded_reads(
                &["get"],
                &|_| None,
                &|| None,
                &|guards| Arc::new(guards.active_summary(|_, provider| provider.contains(phase)))
            ),
            expected,
            "{name}: {history:#?}"
        );
        for write in history
            .assigned
            .as_ref()
            .unwrap()
            .module
            .as_ref()
            .unwrap()
            .members
            .values()
            .flatten()
        {
            assert!(write.timing.single_context().flatten().is_some());
            assert!(
                write.timing.offset < phase,
                "physical writer declaration precedes the invocation"
            );
        }
    }
}
