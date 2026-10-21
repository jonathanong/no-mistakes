use super::*;
use crate::diagnostics::{with_observer, InvocationObserver};
mod fixture;
mod legacy;

#[test]
fn memoized_maps_match_saved_last_write_argument_cycle_and_sparse_cases() {
    for name in [
        "last_shared_write",
        "argument_writes_not_replayed",
        "reentrant_record",
        "sparse_deleted_slots",
        "all_container_kinds",
        "reference_free_and_unidentified",
        "arena_before",
        "arena_after",
        "distinct_prefix_chain",
    ] {
        let case = fixture::load(name);
        let expected = legacy::collect(&case.root, case.arena(), &case.definite);
        let optimized = collect::<false>(&case.root, case.arena(), &case.definite, &mut None);
        assert_eq!(optimized, expected, "{name}");
        let owned = optimized
            .into_iter()
            .map(|(id, (text, complete))| (id, (text.to_owned(), complete)))
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(owned, case.expected, "{name}");
    }
}

#[test]
fn shared_identity_keeps_before_and_after_arena_results_separate() {
    let before = fixture::load("arena_before");
    let after = fixture::load("arena_after");
    // Exactly the same immutable record must see two distinct mutable arenas.
    let previous = collect::<false>(&before.root, before.arena(), &before.definite, &mut None);
    let current = collect::<false>(&before.root, after.arena(), &after.definite, &mut None);
    assert_eq!(previous[&1], ("before", true));
    assert_eq!(current[&1], ("after", false));
}

#[test]
fn observed_collection_reports_real_work_and_disabled_collection_keeps_no_ledger() {
    let case = fixture::load("last_shared_write");
    let observer = InvocationObserver::new(true);
    with_observer(Some(observer.clone()), || {
        let mut work = progress::Walk::new();
        collect::<true>(&case.root, case.arena(), &case.definite, &mut work);
    });
    assert!(observer.snapshot().work["postgres.annotation.prefix_nodes"] > 0);
    let disabled = InvocationObserver::new(false);
    with_observer(Some(disabled.clone()), || {
        collect::<false>(&case.root, case.arena(), &case.definite, &mut None);
    });
    assert!(disabled.snapshot().work.is_empty());
}

#[test]
fn saved_shared_dags_visit_records_instead_of_expanding_every_container_path() {
    for name in ["shared_prefix_depth30", "shared_arguments_depth30"] {
        let case = fixture::load(name);
        let observer = InvocationObserver::new(true);
        let actual = with_observer(Some(observer.clone()), || {
            let mut work = progress::Walk::new();
            collect::<true>(&case.root, case.arena(), &case.definite, &mut work)
        });
        assert_eq!(actual[&1], ("SELECT ", true));
        let visits = observer.snapshot().work["postgres.annotation.prefix_nodes"];
        // Thirty duplicate edges used to expand billions of paths.
        assert!(visits <= 2 * 31 + 2, "{name}: {visits}");
    }
}
