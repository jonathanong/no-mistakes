use super::super::super::*;
use super::*;
use std::cell::Cell;
thread_local! {
    pub(super) static OLD_COPY: Cell<bool> = const { Cell::new(false) };
    pub(super) static MATERIALIZATIONS: Cell<usize> = const { Cell::new(0) };
}
#[test]
fn shared_names_preserve_all_facts_and_reduce_owned_copies() {
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
                OLD_COPY.with(|n| n.set(old));
                MATERIALIZATIONS.with(|n| n.set(0));
                let facts = crate::codebase::dependencies::extract::extract_import_facts_from_program_with_source_and_demands(
                    &parsed.program, &source, false, false, demand);
                OLD_COPY.with(|n| n.set(false));
                (facts, MATERIALIZATIONS.with(Cell::get))
            };
            let (old, old_copies) = run(true);
            let (new, new_copies) = run(false);
            assert_eq!(old, new, "{file}: demand={demand}");
            assert!(new_copies <= old_copies);
            if !demand {
                assert_eq!((old_copies, new_copies), (0, 0));
            }
            println!("NAME_MATERIALIZATIONS {file} demand={demand} {old_copies}->{new_copies}");
            if file == "mutable-pattern-phase-scale.mts" && demand {
                assert!(old_copies > new_copies);
                assert!(new_copies > 0);
            }
        }
    }
}
#[test]
fn adjacent_names_share_while_interleaved_names_keep_their_values() {
    println!(
        "PRIVATE_NAME_LAYOUT root_write={} phase_walk={} changed_roots={}",
        size_of::<phase_walk::RootPhaseWrite>(),
        size_of::<phase_walk::PhaseWalk<'_>>(),
        size_of::<Option<phase_changed_roots::ChangedRoots>>()
    );
    let first = share_previous(None, "first");
    assert!(Rc::ptr_eq(&first, &share_previous(Some(&first), "first")));
    let second = share_previous(Some(&first), "second");
    assert!(!Rc::ptr_eq(&first, &second));
    let first_again = share_previous(Some(&second), "first");
    assert_eq!(first, first_again);
    assert!(!Rc::ptr_eq(&first, &first_again));
    let empty = share_previous(None, "");
    assert!(Rc::ptr_eq(&empty, &share_previous(Some(&empty), "")));
}
