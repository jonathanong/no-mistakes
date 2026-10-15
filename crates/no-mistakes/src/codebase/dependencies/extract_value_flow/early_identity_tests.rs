use super::*;

#[test]
fn early_identity_intervals_preserve_the_actual_destructuring_read_phase() {
    let source = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-cases/codebase-analysis/import-facts/fixture/nested-receiver-late-union.mts",
        ),
    )
    .unwrap();
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
    let histories = &facts.value_flow.receiver_mutations.as_ref().unwrap()[&0];
    for (name, member, first, later) in [
        ("left", "get", "[left.get", "take(left.get"),
        ("right", "set", "[right.set", "take(right.set"),
    ] {
        let assigned = histories[name].assigned.as_ref().unwrap();
        let early = assigned.early.as_ref().unwrap();
        let before = source.find(first).unwrap() as u32 + 1;
        let after = source.find(later).unwrap() as u32 + 5;
        assert!(!early.invalidated(member, &|_| Some(before), assigned));
        assert!(early.invalidated(member, &|_| Some(after), assigned));
        assert!(early.identity_uncertain(member, before, after, assigned));
        assert!(early.identity_uncertain(member, after, before, assigned));
        assert!(!early.identity_uncertain(member, after, after, assigned));
        assert!(!early.identity_uncertain("unrelated", before, after, assigned));
        assert!(!early.identity_uncertain(member, before, source.len() as u32, assigned));
    }
}
