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

