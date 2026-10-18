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

