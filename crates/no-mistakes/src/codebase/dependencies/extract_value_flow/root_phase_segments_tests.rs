use super::super::super::*;
use super::*;
use std::cell::Cell;
thread_local! {
    pub(super) static OLD_CLONE: Cell<bool> = const { Cell::new(false) };
    pub(super) static COUNTS: Cell<[usize; 4]> = const { Cell::new([0; 4]) };
}
#[test]
fn empty_root_catalogs_preserve_fullfacts_without_cloning_segments() {
    let root = crate::test_support::rule_fixture_root("query-reached-per-item")
        .with_file_name("provenance-fixture")
        .join("src/nested-receiver-containers");
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
                OLD_CLONE.with(|n| n.set(old));
                COUNTS.with(|n| n.set([0; 4]));
                let facts = crate::codebase::dependencies::extract::extract_import_facts_from_program_with_source_and_demands(
                    &parsed.program, &source, false, false, demand);
                OLD_CLONE.with(|n| n.set(false));
                (facts, COUNTS.with(Cell::get))
            };
            let (old, before) = run(true);
            let (new, after) = run(false);
            assert_eq!(old, new, "{file}: demand={demand}");
            assert_eq!(before[..2], after[..2]);
            assert!(after[2] <= before[2] && after[3] <= before[3]);
            println!("ROOT_SEGMENT_CLONES {file} demand={demand} {before:?}->{after:?} (captures/with-roots/clone-calls/cloned-segments)");
            if !demand {
                assert_eq!((before, after), ([0; 4], [0; 4]));
            }
            if demand
                && matches!(
                    file,
                    "mutable-pattern-eager-class-scale.mts"
                        | "mutable-pattern-cold-class-scale.mts"
                )
            {
                assert!(before[0] > 0 && before[3] > 0);
                assert_eq!(before[1], 0);
                assert_eq!(after[2..], [0, 0]);
            }
            if demand && file == "mutable-pattern-phase-scale.mts" {
                assert!(after[1] > 0 && after[3] > 0);
            }
        }
    }
}
#[test]
fn required_segments_keep_exact_order_and_empty_segments_remain_empty() {
    let segments = [(9, 12, 0), (1, 8, 1)];
    assert_eq!(capture(&segments, true), segments);
    assert!(capture(&segments, false).is_empty());
    assert!(capture(&[], true).is_empty());
    assert!(capture(&[], false).is_empty());
}
