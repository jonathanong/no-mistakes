use super::*;

#[test]
fn nested_full_summaries_preserve_parent_replay_after_an_overwrite() {
    let case = fixture::load("nested_full_summary_replay");
    let expected = legacy::collect(&case.root, case.arena(), &case.definite);
    let observer = InvocationObserver::new(true);
    let actual = with_observer(Some(observer.clone()), || {
        collect::<true>(
            &case.root,
            case.arena(),
            &case.definite,
            &mut progress::Walk::new(),
        )
    });
    // Both cached and newly completed child maps must reach the parent's map.
    // Otherwise its final replay cannot undo the intervening prefix overwrite.
    assert_eq!(actual, expected);
    assert_eq!(actual[&1], ("first", true));
    assert_eq!(actual[&2], ("second", true));
    assert!(observer.snapshot().work["postgres.annotation.prefix_replayed_entries"] > 0);
    let unobserved = collect::<false>(&case.root, case.arena(), &case.definite, &mut None);
    assert_eq!(unobserved, expected);
    // The observed specialization also tolerates an absent diagnostic ledger.
    let without_work = collect::<true>(&case.root, case.arena(), &case.definite, &mut None);
    assert_eq!(without_work, expected);
}
