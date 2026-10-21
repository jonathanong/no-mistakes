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
        "aliased_distinct_prefix_chain",
        "repeated_aliased_distinct_prefix_chain",
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
        if name == "shared_prefix_depth30" {
            let work = observer.snapshot().work;
            assert!(work["postgres.annotation.prefix_summary_entries"] > 0);
            assert!(work["postgres.annotation.prefix_replayed_entries"] > 0);
        }
    }
}

#[test]
fn unchanged_argument_scope_still_compares_distinct_mutable_arenas() {
    use crate::codebase::postgres::query_annotation::evaluate::Scope;
    let before = fixture::load("arena_before");
    let after = fixture::load("arena_after");
    let scope = Scope::from(FxHashMap::from_iter([(
        "arguments".into(),
        before.root.clone(),
    )]));
    let snapshot = scope.clone();
    assert!(std::ptr::eq(&*scope, &*snapshot));
    assert!(scope.contains_reference());
    let mut changed = FxHashSet::default();
    // Scope identity proves nothing about the externally stored argument arena.
    super::super::changes(
        &before.root,
        &before.root,
        before.arena(),
        after.arena(),
        &mut changed,
        &FxHashSet::default(),
    );
    assert_eq!(changed, FxHashSet::from_iter([1]));
}

#[test]
fn reference_free_entry_cannot_mutate_an_arm_created_builder() {
    use crate::codebase::postgres::query_annotation::evaluate::Scope;
    let before = fixture::load("reference_free_before");
    let after = fixture::load("rebound_builder_after");
    let scope = Scope::from(FxHashMap::from_iter([(
        "binding".into(),
        before.root.clone(),
    )]));
    assert!(!scope.contains_reference());
    assert!(after.root.contains_reference());
    assert!(legacy::collect(&before.root, before.arena(), &before.definite).is_empty());
    let mut changed = FxHashSet::default();
    super::super::changes(
        &before.root,
        &after.root,
        before.arena(),
        after.arena(),
        &mut changed,
        &before.definite,
    );
    assert!(changed.is_empty());
}

#[test]
fn snapshot_aliases_retain_only_inline_cold_summaries() {
    let case = fixture::load("aliased_distinct_prefix_chain");
    // The unreachable arena retains one alias per layer, like entry snapshots.
    // It must not turn a single chain traversal into quadratic summary copies.
    let observer = InvocationObserver::new(true);
    let actual = with_observer(Some(observer.clone()), || {
        collect::<true>(
            &case.root,
            case.arena(),
            &case.definite,
            &mut progress::Walk::new(),
        )
    });
    assert_eq!(
        actual,
        legacy::collect(&case.root, case.arena(), &case.definite)
    );
    assert_eq!(actual.len(), 128);
    let work = observer.snapshot().work;
    assert_eq!(work["postgres.annotation.prefix_nodes"], 256);
    assert_eq!(work["postgres.annotation.prefix_summary_entries"], 1);
    assert!(!work.contains_key("postgres.annotation.prefix_replayed_entries"));
}

#[test]
fn observed_large_summary_replays_restore_intervening_overwrites() {
    let case = fixture::load("repeated_aliased_distinct_prefix_chain");
    let observer = InvocationObserver::new(true);
    let actual = with_observer(Some(observer.clone()), || {
        collect::<true>(
            &case.root,
            case.arena(),
            &case.definite,
            &mut progress::Walk::new(),
        )
    });
    assert_eq!(
        actual,
        legacy::collect(&case.root, case.arena(), &case.definite)
    );
    assert_eq!(actual[&1], ("SELECT 0", true));
    assert_eq!(actual[&2], ("SELECT 1", true));
    let work = observer.snapshot().work;
    assert!(work["postgres.annotation.prefix_summary_entries"] >= 128);
    assert!(work["postgres.annotation.prefix_replayed_entries"] >= 128);
}

#[test]
fn reference_free_walk_adds_no_summary_counter_updates() {
    let case = fixture::load("reference_free_before");
    let observer = InvocationObserver::new(true);
    with_observer(Some(observer.clone()), || {
        assert!(collect::<true>(
            &case.root,
            case.arena(),
            &case.definite,
            &mut progress::Walk::new()
        )
        .is_empty());
    });
    let work = observer.snapshot().work;
    assert_eq!(work["postgres.annotation.prefix_nodes"], 1);
    assert!(!work.contains_key("postgres.annotation.prefix_summary_entries"));
    assert!(!work.contains_key("postgres.annotation.prefix_replayed_entries"));
}
