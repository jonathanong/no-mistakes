use super::*;

type Captures = FxHashMap<Environment, FxHashMap<String, Environment>>;

fn captures(reader: usize, origin: usize) -> Captures {
    FxHashMap::from_iter([(reader, FxHashMap::from_iter([("name".into(), origin)]))])
}

fn compare(
    input: Captures,
    count: usize,
    preserved: usize,
    indices: FxHashMap<usize, usize>,
    moved: bool,
) -> (Captures, usize, usize) {
    let mut expected = input.clone();
    let mut actual = input;
    let mut old_probes = 0;
    let mut new_probes = 0;
    let old_changed = remap(&mut expected, preserved, &indices, &mut |_| old_probes += 1);
    let new_changed = remap_after_compaction(
        &mut actual,
        count,
        preserved,
        &indices,
        moved,
        &mut |_| new_probes += 1,
        &mut |_| {},
    );
    assert_eq!(new_changed, old_changed);
    assert_eq!(actual, expected, "complete forward capture relation");
    (actual, old_probes, new_probes)
}

#[test]
fn identity_domains_skip_actual_origin_probes_and_keep_name_buffers() {
    let input = FxHashMap::from_iter([
        (0, (0..512).map(|n| (format!("name{n}"), 2)).collect()),
        (1, FxHashMap::default()),
    ]);
    let mut actual = input.clone();
    // This pointer check uses the actual map passed through the new path.
    let actual_pointer = actual[&0].get_key_value("name0").unwrap().0.as_ptr();
    let mut probes = 0;
    assert!(!remap_after_compaction(
        &mut actual,
        3,
        1,
        &FxHashMap::from_iter([(1, 1), (2, 2)]),
        false,
        &mut |_| probes += 1,
        &mut |_| {},
    ));
    assert_eq!(
        actual[&0].get_key_value("name0").unwrap().0.as_ptr(),
        actual_pointer
    );
    let (_, old, new) = compare(input, 3, 1, FxHashMap::from_iter([(1, 1), (2, 2)]), false);
    assert_eq!((old, new), (512, 0));
    assert_eq!(probes, 0);
}

#[test]
fn empty_maps_empty_origins_and_preserved_prefix_match_original() {
    compare(Captures::default(), 0, 0, FxHashMap::default(), false);
    compare(
        FxHashMap::from_iter([(0, FxHashMap::default())]),
        1,
        1,
        FxHashMap::default(),
        false,
    );
    compare(captures(0, 1), 2, 2, FxHashMap::default(), false);
}

#[test]
fn dropped_trailing_reader_and_moving_mapping_use_original_fallback() {
    compare(captures(2, 0), 3, 1, FxHashMap::from_iter([(1, 1)]), false);
    let (_, old, new) = compare(captures(0, 2), 3, 1, FxHashMap::from_iter([(2, 1)]), true);
    assert_eq!((old, new), (1, 1));
}

#[test]
fn stale_reader_key_preserves_fallback_even_with_empty_origins() {
    compare(
        captures(9, 0),
        2,
        0,
        FxHashMap::from_iter([(0, 0), (1, 1)]),
        false,
    );
    compare(
        FxHashMap::from_iter([(9, FxHashMap::default())]),
        2,
        0,
        FxHashMap::from_iter([(0, 0), (1, 1)]),
        false,
    );
}

#[test]
fn outside_origin_ids_remain_unchanged_when_reader_is_valid() {
    let (actual, old, new) = compare(captures(0, 99), 1, 0, FxHashMap::from_iter([(0, 0)]), false);
    assert_eq!(actual[&0]["name"], 99);
    assert_eq!((old, new), (1, 0));
}

#[test]
fn malformed_domain_or_wrong_movement_hint_never_skips_changed_origins() {
    for (indices, preserved) in [
        (FxHashMap::from_iter([(0, 0), (2, 2)]), 1),
        (FxHashMap::from_iter([(1, 1), (9, 9)]), 1),
        (FxHashMap::from_iter([(1, 2), (2, 1)]), 1),
    ] {
        compare(captures(2, 2), 3, preserved, indices, false);
    }
    compare(captures(0, 2), 1, 2, FxHashMap::default(), false);
}

#[test]
fn sparse_forward_storage_keeps_original_identity_capacity_behavior() {
    let mut input = captures(0, 0);
    input.reserve(1024);
    input.get_mut(&0).unwrap().reserve(1024);
    let expected = (input.capacity(), input[&0].capacity());
    let mut probes = 0;
    assert!(!remap_after_compaction(
        &mut input,
        1,
        0,
        &FxHashMap::from_iter([(0, 0)]),
        false,
        &mut |_| probes += 1,
        &mut |_| {},
    ));
    assert_eq!((input.capacity(), input[&0].capacity()), expected);
}

#[test]
fn conservative_true_movement_hint_uses_original_scan_for_identity_domain() {
    let (_, old, new) = compare(
        captures(0, 1),
        2,
        0,
        FxHashMap::from_iter([(0, 0), (1, 1)]),
        true,
    );
    assert_eq!((old, new), (1, 1));
}

// Exact c585 pre-fix guard, with only visit probes added for the regression.
// Keep this reference independent of the corrected production entrypoint.
fn previous_remap_after_compaction(
    captured: &mut FxHashMap<Environment, FxHashMap<String, Environment>>,
    old_scope_count: usize,
    preserved: Environment,
    indices: &FxHashMap<Environment, Environment>,
    environments_changed: bool,
    origin_probe: &mut impl FnMut(Environment),
    identity_probe: &mut impl FnMut(Environment),
) -> bool {
    let unchanged = !environments_changed
        && old_scope_count.checked_sub(preserved) == Some(indices.len())
        && indices.iter().all(|(old, new)| {
            identity_probe(*old);
            *old >= preserved && *old < old_scope_count && old == new
        })
        && captured.keys().all(|reader| *reader < old_scope_count);
    if unchanged {
        return false;
    }
    remap(captured, preserved, indices, origin_probe)
}

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
