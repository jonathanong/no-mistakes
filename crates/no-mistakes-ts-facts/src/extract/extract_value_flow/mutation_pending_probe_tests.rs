use super::*;

#[test]
fn saved_hot_root_uses_no_pending_heap_buffers() {
    let root = crate::test_support::rule_fixture_root("query-reached-per-item")
        .with_file_name("provenance-fixture");
    let source =
        std::fs::read_to_string(root.join("src/nested-receiver-containers/mutable-root-scale.mts"))
            .unwrap();
    let allocator = Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    QUEUE_COUNTS.with(|counts| counts.set([0; 3]));
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    )
    .value_flow;
    let counts = QUEUE_COUNTS.with(std::cell::Cell::get);
    println!("PENDING_QUEUE_COUNTS {counts:?}");
    assert_eq!(facts.receiver_only_origins, 129);
    assert_eq!(counts[1], 2);
    assert_eq!(counts[0], 0, "pending heap allocations and reallocations");
}

#[test]
fn actual_mutation_queue_layout_is_bounded() {
    type Entry = super::super::PendingMutation;
    let entry = std::mem::size_of::<Entry>();
    let queue = std::mem::size_of::<super::super::queue::Queue<Entry>>();
    let old = std::mem::size_of::<Vec<Entry>>();
    println!("PENDING_QUEUE_LAYOUT entry={entry} old={old} new={queue}");
    assert!(queue <= 232);
}
