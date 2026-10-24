use super::*;

#[test]
fn phase_catalogs_preserve_alias_projection_rest_and_nested_rhs_boundaries() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/import-facts/fixture/destructuring-phase-edges.mts",
    );
    let source = std::fs::read_to_string(fixture).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    );
    let histories = &facts.value_flow.receiver_mutations.as_ref().unwrap()[&0];
    assert!(std::sync::Arc::ptr_eq(
        &histories["merged"],
        &histories["alias"]
    ));
    for (receiver, projection, marker) in [
        ("merged", vec!["get"], "later: unused = read(merged)"),
        ("merged", vec!["set"], "later: unused = read(merged)"),
        (
            "nested",
            vec!["get", "client"],
            "later: unused = read(nested)",
        ),
        (
            "dynamic",
            vec!["get", "client"],
            "later: unused = read(dynamic)",
        ),
        ("shorthand", vec!["get"], "unused = read(shorthand)"),
        ("ordinary", vec!["get"], "field: unused = read(ordinary)"),
        (
            "updated",
            vec!["get", "client"],
            "field: unused = read(updated)",
        ),
        ("deferred", vec!["get"], "after: unused = read(deferred)"),
    ] {
        let offset = (source.find(marker).unwrap() + marker.find("read(").unwrap()) as u32;
        assert!(
            histories[receiver].assignment_invalidated(&projection, offset, None, false),
            "{receiver} {projection:?} at {marker}"
        );
        assert!(!histories[receiver].assignment_invalidated(&projection, 0, None, false));
    }
    let owner = facts
        .callable_scope_ids
        .iter()
        .find(|(_, name)| name.as_ref() == "saved")
        .unwrap()
        .0;
    let before_owned = source.find("read(owned)").unwrap() as u32;
    let after_owned = source.rfind("read(owned)").unwrap() as u32;
    assert!(!histories["owned"].assignment_invalidated(&["get"], before_owned, Some(owner), false));
    assert!(histories["owned"].assignment_invalidated(&["get"], after_owned, Some(owner), false));
    for receiver in ["arrayRest", "objectRest"] {
        let marker = format!("read({receiver})");
        let before = source.find(&marker).unwrap() as u32;
        let after = source.rfind(&marker).unwrap() as u32;
        assert!(!histories[receiver].assignment_invalidated(&["get"], before, None, false));
        assert!(histories[receiver].assignment_invalidated(&["get"], after, None, false));
    }
    assert!(!histories["nested"].assignment_invalidated(&["set", "client"], u32::MAX, None, false));
    assert!(!histories["dynamic"].assignment_invalidated(
        &["set", "client"],
        u32::MAX,
        None,
        false
    ));
}
