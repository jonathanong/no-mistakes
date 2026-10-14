use super::*;

#[test]
fn ancestor_assignments_preserve_rhs_phase_cutoff_and_writer_boundaries() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/import-facts/fixture/assigned-ancestor-boundaries.mts",
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
    let histories = &facts.value_flow.receiver_mutations.as_ref().unwrap()[&0];
    let reads = |name: &str| {
        source
            .match_indices(&format!("read({name})"))
            .map(|(offset, _)| offset as u32)
            .collect::<Vec<_>>()
    };
    let phased = reads("phased");
    // Source syntax precedes the RHS physically, but that default executes
    // after the nested receiver is replaced; the RHS still sees the old one.
    for (offset, expected) in [(phased[0], true), (phased[1], false), (phased[2], true)] {
        assert_eq!(
            histories["phased"].assignment_invalidated(&["get", "nested"], offset, None, false),
            expected
        );
        assert!(!histories["phased"].assignment_invalidated(
            &["get", "other"],
            offset,
            None,
            false
        ));
    }
    let ordinary = reads("ordinary");
    assert!(!histories["ordinary"].assignment_invalidated(
        &["get", "nested"],
        ordinary[0],
        None,
        false
    ));
    assert!(histories["ordinary"].assignment_invalidated(
        &["get", "nested"],
        ordinary[1],
        None,
        false
    ));
    assert!(histories["captured"].assignment_invalidated(&["get", "nested"], 0, None, false));
}
