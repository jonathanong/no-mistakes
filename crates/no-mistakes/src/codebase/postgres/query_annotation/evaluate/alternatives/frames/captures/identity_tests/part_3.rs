#[test]
fn saved_empty_capture_domain_avoids_identity_guard_visits() {
    let fixture = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("capture-compaction-work.json");
    let case: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(fixture).unwrap()).unwrap();
    let preserved = case["preserved"].as_u64().unwrap() as usize;
    let count = case["old_scope_count"].as_u64().unwrap() as usize;
    let indices = (preserved..count).map(|id| (id, id)).collect();
    let (mut old, mut new) = (Captures::default(), Captures::default());
    let (mut old_visits, mut new_visits) = (0, 0);
    let before = previous_remap_after_compaction(
        &mut old,
        count,
        preserved,
        &indices,
        false,
        &mut std::mem::drop::<Environment>,
        &mut |_| old_visits += 1,
    );
    let after = remap_after_compaction(
        &mut new,
        count,
        preserved,
        &indices,
        false,
        &mut std::mem::drop::<Environment>,
        &mut |_| new_visits += 1,
    );
    assert_eq!((before, &old), (after, &new));
    assert!(!after);
    assert_eq!((old_visits, new_visits), (count - preserved, 0));
    assert!(
        old_visits > 0,
        "saved identity domain must exercise removed work"
    );
}
