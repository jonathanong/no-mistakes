use super::*;

#[test]
fn forward_receiver_paths_group_physical_writers_and_skip_missing_initializers() {
    for (name, sources, replays) in [("grouped", 2, 1), ("uninitialized", 1, 0)] {
        let source = std::fs::read_to_string(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../test-cases/codebase-analysis/value-flow-forward-receiver-alias/fixture/src")
                .join(format!("{name}.mts")),
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
        )
        .value_flow;
        assert_eq!(facts.reference_forward_sources, sources, "{name}");
        assert_eq!(facts.reference_forward_writer_replays, replays, "{name}");
    }
}

#[test]
fn forward_receiver_writer_work_is_indexed_and_diamond_paths_share_projection() {
    for count in [16, 64] {
        let source = std::fs::read_to_string(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../test-cases/codebase-analysis/value-flow-forward-receiver-alias/fixture/src")
                .join(format!("scale-{count}.mts")),
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
        )
        .value_flow;
        assert_eq!(facts.reference_forward_sources, count);
        assert_eq!(facts.reference_forward_writer_replays, count);
        assert!(
            facts.reference_binding_visits <= 8 * count,
            "{count}: {}",
            facts.reference_binding_visits
        );
        assert!(
            facts.reference_path_visits <= 8 * count,
            "{count}: {}",
            facts.reference_path_visits
        );
        let histories = &facts.receiver_mutations.as_ref().unwrap()[&0];
        assert!(histories["primary"].has_assignments);
    }
}
