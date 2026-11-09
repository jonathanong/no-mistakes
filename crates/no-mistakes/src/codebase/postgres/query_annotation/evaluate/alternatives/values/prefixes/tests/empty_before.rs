use super::{fixture, legacy, FxHashSet};
use crate::diagnostics::{with_observer, InvocationObserver};
use std::collections::BTreeMap;

fn saved(name: &str) -> fixture::Case {
    fixture::load_path(
        &crate::test_support::rule_fixture_root("postgres-require-query-annotation")
            .join("empty-before-prefix")
            .join(format!("{name}.json")),
    )
}

fn nodes(case: &fixture::Case) -> u64 {
    let observer = InvocationObserver::new(true);
    with_observer(Some(observer.clone()), || {
        let mut work = super::super::progress::Walk::new();
        super::super::collect::<true>(&case.root, case.arena(), &case.definite, &mut work);
    });
    observer.snapshot().work["postgres.annotation.prefix_nodes"]
}

fn compare(
    before: &fixture::Case,
    after: &fixture::Case,
    verbose: bool,
) -> (FxHashSet<u64>, crate::diagnostics::DiagnosticsSnapshot) {
    let observer = InvocationObserver::new(verbose);
    let mut changed = FxHashSet::default();
    with_observer(Some(observer.clone()), || {
        let mut work = super::super::progress::Walk::new();
        super::super::super::changes(
            &before.root,
            &after.root,
            before.arena(),
            after.arena(),
            &mut changed,
            &after.definite,
            &mut work,
        );
    });
    (changed, observer.snapshot())
}

#[test]
fn masked_cycle_has_no_old_identity_and_does_not_walk_new_dense_sparse_dag() {
    let before = saved("masked-before");
    let after = saved("after");
    assert!(legacy::collect(&before.root, before.arena(), &after.definite).is_empty());
    assert!(!legacy::collect(&after.root, after.arena(), &after.definite).is_empty());
    let (changed, diagnostics) = compare(&before, &after, true);
    assert!(changed.is_empty());
    let work = diagnostics.work;
    assert_eq!(work["postgres.annotation.prefix_after_empty_skips"], 1);
    assert_eq!(work["postgres.annotation.prefix_walks"], 1);
    assert_eq!(work["postgres.annotation.prefix_walks_completed"], 1);
    assert_eq!(work["postgres.annotation.prefix_nodes"], nodes(&before));
    assert!(nodes(&after) > 0);
    let (disabled, diagnostics) = compare(&before, &after, false);
    assert_eq!(changed, disabled);
    assert!(diagnostics.work.is_empty());
}

#[test]
fn unmasked_dense_sparse_mutations_keep_both_walks_and_exact_taint() {
    let before = saved("unmasked-before");
    let after = saved("after");
    for case in [&before, &after, &saved("masked-before")] {
        let actual = legacy::collect(&case.root, case.arena(), &case.definite)
            .into_iter()
            .map(|(id, (text, done))| (id, (text.to_owned(), done)))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(actual, case.expected);
    }
    let (changed, diagnostics) = compare(&before, &after, true);
    assert_eq!(changed, FxHashSet::from_iter([1, 2]));
    let work = diagnostics.work;
    assert_eq!(work["postgres.annotation.prefix_after_empty_skips"], 0);
    assert_eq!(work["postgres.annotation.prefix_walks"], 1);
    assert_eq!(work["postgres.annotation.prefix_walks_completed"], 1);
    assert_eq!(
        work["postgres.annotation.prefix_nodes"],
        nodes(&before) + nodes(&after)
    );
    let (disabled, diagnostics) = compare(&before, &after, false);
    assert_eq!(changed, disabled);
    assert!(diagnostics.work.is_empty());
}
