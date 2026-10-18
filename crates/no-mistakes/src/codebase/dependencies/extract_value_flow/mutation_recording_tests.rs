use super::super::*;

#[test]
#[cfg(feature = "test-instrumentation")]
fn exact_member_records_descend_once_per_receiver_segment() {
    let root = crate::test_support::rule_fixture_root("query-reached-per-item")
        .with_file_name("provenance-fixture");
    let source =
        std::fs::read_to_string(root.join("src/nested-receiver-containers/mutable-root-scale.mts"))
            .unwrap();
    let allocator = Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    super::metrics::PATH_DESCENTS.with(|count| count.set(0));
    super::metrics::CHILD_KEYS.with(|counts| counts.set([0; 4]));
    super::metrics::OWNED_KEYS.with(|counts| counts.set([0; 4]));
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    );
    // Each box.client.get write records box's one-segment path and client's
    // empty path. Summary and exact-assignment recording must share the descent.
    assert_eq!(super::metrics::PATH_DESCENTS.with(|count| count.get()), 128);
    let keys = super::metrics::CHILD_KEYS.with(|counts| counts.get());
    println!("MUTATION_CHILD_KEYS {keys:?}");
    let owned = super::metrics::OWNED_KEYS.with(|counts| counts.get());
    println!("MUTATION_OWNED_KEYS {owned:?}");
    assert_eq!(owned[2], 0, "exit records move their existing owned key");
    assert_eq!(facts.value_flow.reference_terminal_paths, 128);
    assert_eq!(owned[0], 258, "only terminal names transfer ownership");
    // Existing receiver children need no new owned key for a later write.
    assert_eq!(keys[2], keys[1]);
    let history = &facts.value_flow.receiver_mutations.as_ref().unwrap()[&0]["client"];
    assert!(history.assignment_invalidated(&["get"], source.len() as u32, None, false));
}

fn saved_history_source() -> String {
    std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-cases/codebase-analysis/import-facts/fixture/single-descent-history.mts",
        ),
    )
    .unwrap()
}

#[test]
fn saved_nested_histories_keep_start_completion_dynamic_and_context_projections() {
    use mutation_history::ValueAssignedWriter;
    let source = saved_history_source();
    let first_start = source
        .find("box.outer.client.get = replacement();")
        .unwrap() as u32;
    let first_end = first_start + "box.outer.client.get = replacement()".len() as u32;
    let dynamic_start = source.find("box.outer[key].get = replacement();").unwrap() as u32;
    let dynamic_end = dynamic_start + "box.outer[key].get = replacement()".len() as u32;
    let other = CallableId(source.find("function writeAgain").unwrap() as u32);
    let other_start = source
        .rfind("box.outer.client.get = replacement();")
        .unwrap() as u32;
    let other_end = other_start + "box.outer.client.get = replacement()".len() as u32;
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
    let root = &histories["box"];
    let outer = &root.members["outer"];
    let direct = &outer.members["client"];
    let client = &histories["client"];
    for (history, terminal) in [
        (root.as_ref(), false),
        (outer, false),
        (direct, false),
        (client.as_ref(), true),
    ] {
        let summary = history.summary.as_ref().unwrap();
        assert_eq!(summary.offset, first_start);
        // Deferred module-provider writes remain guarded instead of poisoning
        // the unconditional summary before the caller phase is known.
        let expected = if terminal {
            vec![None]
        } else {
            vec![None, Some(other)]
        };
        assert_eq!(summary.contexts, expected.into_iter().collect());
        assert!(!history.has_unassigned);
    }
    assert!(root.has_assignments);
    assert!(!outer.has_assignments);
    let exact = direct.assigned.as_ref().unwrap().members["get"];
    assert_eq!(exact.offset, first_end);
    assert_eq!(exact.writer, ValueAssignedWriter::Mixed);
    assert_eq!(exact.unconditional_offset, None);
    let dynamic = outer.dynamic.as_ref().unwrap();
    assert_eq!(dynamic.summary.as_ref().unwrap().offset, dynamic_start);
    assert_eq!(
        dynamic.assigned.as_ref().unwrap().members["get"].offset,
        dynamic_end
    );
    let unknown = &outer
        .assigned
        .as_ref()
        .unwrap()
        .replacements
        .as_ref()
        .unwrap()
        .identity
        .as_ref()
        .unwrap()
        .unknown;
    assert!(!unknown.changed(Some((dynamic_start, None)), Some((dynamic_end - 1, None))));
    assert!(unknown.changed(Some((dynamic_start, None)), Some((dynamic_end, None))));
    let propagated = client
        .assigned
        .as_ref()
        .unwrap()
        .propagated
        .as_ref()
        .unwrap();
    assert_eq!(propagated.aggregate().members["get"].offset, exact.offset);
    assert_eq!(
        propagated.aggregate().members["get"].writer,
        ValueAssignedWriter::Module
    );
    assert_eq!(
        propagated.context(None).unwrap().members["get"].offset,
        first_end
    );
    assert!(propagated.context(Some(other)).is_none());
    let guarded = client
        .assigned
        .as_ref()
        .unwrap()
        .module
        .as_ref()
        .unwrap()
        .active_summary(|_, _| true);
    let guarded = guarded.propagated.as_ref().unwrap();
    let callable = guarded.context(Some(other)).unwrap().members["get"];
    assert_eq!(callable.offset, other_end);
    assert_eq!(callable.writer, ValueAssignedWriter::Callable(other));
    assert_eq!(guarded.aggregate().members["get"], callable);
}

#[path = "mutation_capture_tests.rs"]
mod captures;
