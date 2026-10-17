use super::*;
use std::cell::Cell;
thread_local! {
    pub(in super::super) static OLD: Cell<bool> = const { Cell::new(false) };
    pub(in super::super) static QUERIES: Cell<usize> = const { Cell::new(0) };
    pub(in super::super) static DESCRIPTORS: Cell<usize> = const { Cell::new(0) };
    static OWNERS: Cell<[usize; 7]> = const { Cell::new([0; 7]) };
}
#[test]
fn root_existence_is_distinct_from_missing_selected_snapshot() {
    let mut origins = ReceiverBindingOrigins::default();
    origins.record((0, "box"), (10, None), ValueExpression::Unknown, true);
    let key = (0, "box".to_string());
    let before = origins.version(&key, (0, None)).unwrap();
    assert!(before.selected.is_none());
    assert!(origins.contains(0, "box"));
    assert_eq!(origins.version(&key, (10, None)).unwrap().selected, Some(0));
    assert!(origins
        .version(&(1, "box".to_string()), (10, None))
        .is_none());
    assert!(origins
        .version(&(0, "missing".to_string()), (10, None))
        .is_none());
}
#[test]
fn fused_lookup_preserves_fullfacts_and_removes_duplicate_work() {
    let root = crate::test_support::rule_fixture_root("query-reached-per-item")
        .with_file_name("provenance-fixture")
        .join("src/nested-receiver-containers");
    let mut saved_hot = None;
    for file in [
        "mutable-root-scale.mts",
        "mutable-pattern-phase-scale.mts",
        "mutable-pattern-eager-class-scale.mts",
        "mutable-pattern-cold-class-scale.mts",
        "mutable-pattern-deferred-diamond.mts",
        "mutable-let-pattern-rhs-rebind.mts",
        "mutable-let-pattern-lhs-rebind.mts",
    ] {
        let source = std::fs::read_to_string(root.join(file)).unwrap();
        let allocator = Allocator::default();
        let parsed = oxc_parser::Parser::new(&allocator, &source, SourceType::ts()).parse();
        assert!(parsed.diagnostics.is_empty());
        for demand in [false, true] {
            let run = |old| {
                OLD.with(|n| n.set(old));
                QUERIES.with(|n| n.set(0));
                DESCRIPTORS.with(|n| n.set(0));
                OWNERS.with(|n| n.set([0; 7]));
                let facts = crate::codebase::dependencies::extract::extract_import_facts_from_program_with_source_and_demands(
                    &parsed.program, &source, false, false, demand);
                OLD.with(|n| n.set(false));
                (
                    facts,
                    QUERIES.with(Cell::get),
                    DESCRIPTORS.with(Cell::get),
                    OWNERS.with(Cell::get),
                )
            };
            let (mut old, old_queries, old_descriptors, old_owners) = run(true);
            let (new, new_queries, new_descriptors, new_owners) = run(false);
            assert_eq!(old_owners, new_owners);
            println!("OWNERS {file} demand={demand} {new_owners:?} (roots/empty/one/many/owners/capacity/max)");
            // These two cfg-only work metrics intentionally change; retain the
            // actual values and compare every other fact without normalization.
            println!("LOOKUPS {file} demand={demand} table={old_queries}->{new_queries} descriptors={old_descriptors}->{new_descriptors} membership={}->{} version_probes={}->{}",
                old.value_flow.receiver_only_membership_probes, new.value_flow.receiver_only_membership_probes,
                old.value_flow.reference_root_probes, new.value_flow.reference_root_probes);
            old.value_flow.receiver_only_membership_probes =
                new.value_flow.receiver_only_membership_probes;
            old.value_flow.reference_root_probes = new.value_flow.reference_root_probes;
            assert_eq!(old, new, "{file}: demand={demand}");
            assert!(new_queries <= old_queries && new_descriptors <= old_descriptors);
            if file == "mutable-root-scale.mts" && demand {
                assert_eq!(old_queries - new_queries, 128);
                assert_eq!(old_descriptors - new_descriptors, 128);
                assert_eq!(new.value_flow.receiver_only_origins, 129);
                saved_hot = Some((old_queries, new_queries));
            }
        }
    }
    assert!(saved_hot.is_some());
}

pub(super) fn observe_owners(roots: &FxHashMap<usize, FxHashMap<String, RootOrigins>>) {
    let mut totals = [0; 7];
    for root in roots.values().flat_map(|names| names.values()) {
        let len = root.owners.len();
        totals[0] += 1;
        totals[if len == 0 {
            1
        } else if len == 1 {
            2
        } else {
            3
        }] += 1;
        totals[4] += len;
        totals[5] += root.owners.capacity();
        totals[6] = totals[6].max(len);
    }
    OWNERS.with(|cell| cell.set(totals));
}
