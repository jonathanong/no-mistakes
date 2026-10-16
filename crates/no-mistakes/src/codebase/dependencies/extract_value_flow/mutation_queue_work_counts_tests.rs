//! Diagnostic-only work counts; no production implementation or budgets change.
use super::super::super::*;
thread_local! {
    static ENABLED: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
    static COUNTS: std::cell::Cell<[usize; 8]> = const { std::cell::Cell::new([0; 8]) };
}
pub(in super::super) fn record(index: usize) {
    if ENABLED.with(std::cell::Cell::get) {
        COUNTS.with(|counts| {
            let mut value = counts.get();
            value[index] += 1;
            counts.set(value);
        });
    }
}
fn collect(name: &str, demand: bool, enabled: bool) -> (ImportFacts, [usize; 8]) {
    let root = crate::test_support::rule_fixture_root("query-reached-per-item")
        .with_file_name("provenance-fixture");
    let file = root.join("src/nested-receiver-containers").join(name);
    let source = std::fs::read_to_string(&file).unwrap();
    let allocator = Allocator::default();
    let parsed =
        oxc_parser::Parser::new(&allocator, &source, SourceType::from_path(&file).unwrap()).parse();
    assert!(parsed.diagnostics.is_empty());
    COUNTS.with(|counts| counts.set([0; 8]));
    ENABLED.with(|flag| flag.set(enabled));
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        demand,
    );
    ENABLED.with(|flag| flag.set(true));
    (facts, COUNTS.with(std::cell::Cell::get))
}
#[test]
fn queue_reserve_saved_controls_preserve_facts_and_record_actual_work() {
    for name in [
        "mutable-root-scale.mts",
        "mutable-projection-wide.ts",
        "mutable-pattern-deferred-diamond.mts",
        "mutable-let-cycle.mts",
        "mutable-pattern-phase-scale.mts",
        "mutable-pattern-eager-class-scale.mts",
        "mutable-pattern-cold-class-scale.mts",
    ] {
        for demand in [false, true] {
            let (measured, counts) = collect(name, demand, true);
            let (plain, disabled) = collect(name, demand, false);
            assert_eq!(
                measured, plain,
                "instrumentation changed public facts: {name}/{demand}"
            );
            assert_eq!(disabled, [0; 8]);
            println!("QUEUE_WORK {name} demand={demand} reserve/push/extend/insert/append/promote/pop/new={counts:?}");
            if !demand {
                assert_eq!(counts, [0; 8], "imports-only must keep mutation queue cold");
            } else {
                assert!(
                    counts[4] > 0 && counts[6] > 0,
                    "no actual queue traversal: {name}"
                );
                assert!(measured.value_flow.receiver_mutations.as_ref().unwrap()[&0]
                    .contains_key("client"));
            }
            if name == "mutable-root-scale.mts" && demand {
                assert_eq!(measured.value_flow.receiver_only_origins, 129);
                assert_eq!(counts[5], 0, "hot root must retain zero heap promotions");
                assert!(
                    measured.value_flow.receiver_mutations.as_ref().unwrap()[&0]["client"]
                        .assignment_invalidated(&["get"], u32::MAX, None, false)
                );
            }
        }
    }
}
#[test]
fn queue_reserve_hot_root_single_capacity_decision() {
    let (_, counts) = collect("mutable-root-scale.mts", true, true);
    println!("QUEUE_RESERVE_RED actual={counts:?}");
    // The retained a022 diagnostic was RED at 512 reserve calls /128 appends.
    // Preserve one capacity decision per append without dropping traversals.
    assert!(
        counts[0] <= counts[4],
        "redundant reserve decisions: {counts:?}"
    );
}
