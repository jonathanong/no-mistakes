use super::*;
#[test]
fn all_membership_and_owned_promotion_paths() {
    let mut roots = ChangedRoots::default();
    let mut oracle = fx_set();
    assert!(!roots.contains("first"));
    let first = "first".to_owned();
    let pointer = first.as_ptr();
    for name in [
        first,
        "first".into(),
        "second".into(),
        "second".into(),
        "third".into(),
        "".into(),
        "λ".into(),
    ] {
        oracle.insert(name.clone());
        roots.insert(name);
        for name in ["first", "second", "third", "", "λ", "missing"] {
            assert_eq!(roots.contains(name), oracle.contains(name));
        }
    }
    let ChangedRoots::Many(names) = &roots else {
        panic!("second distinct name must promote");
    };
    assert_eq!(
        names.get("first").unwrap().as_ptr(),
        pointer,
        "move the existing name instead of cloning"
    );
}
#[test]
fn empty_and_repeated_singleton_do_not_promote() {
    drop(ChangedRoots::default());
    let mut roots = ChangedRoots::default();
    roots.insert("one".into());
    roots.insert("one".into());
    assert!(matches!(&roots, ChangedRoots::One(name) if name == "one"));
    assert!(roots.contains("one"));
    assert!(!roots.contains("two"));
    if let Ok(old) = std::env::var("LEGACY_PHASE_WALK_SIZE") {
        assert!(size_of::<phase_walk::PhaseWalk<'_>>() <= old.parse::<usize>().unwrap() + 8);
    }
    eprintln!(
        "ACTUAL_NEW_LAYOUT field={} phase_walk={}",
        size_of::<Option<ChangedRoots>>(),
        size_of::<phase_walk::PhaseWalk<'_>>()
    );
}
