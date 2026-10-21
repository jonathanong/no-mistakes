use super::*;

#[test]
fn unchanged_ownership_skips_rebuilding_even_when_unrelated_frames_move() {
    let mut captured = FxHashMap::from_iter([
        (0, FxHashMap::from_iter([("preserved".into(), 0)]).into()),
        (1, FxHashMap::from_iter([("inherited".into(), 0)]).into()),
    ]);
    let snapshot = captured.clone();
    assert!(!remap(
        &mut captured,
        1,
        &FxHashMap::from_iter([(1, 1), (3, 2)]),
        &mut std::mem::drop::<Environment>
    ));
    assert_eq!(captured, snapshot);
}

#[test]
fn ownership_changes_include_removed_readers_and_moved_origins() {
    let mut captured = FxHashMap::from_iter([
        (0, FxHashMap::from_iter([("owner".into(), 2)]).into()),
        (2, FxHashMap::from_iter([("cyclic".into(), 0)]).into()),
        (3, FxHashMap::from_iter([("dead".into(), 3)]).into()),
    ]);
    assert!(remap(
        &mut captured,
        1,
        &FxHashMap::from_iter([(2, 1)]),
        &mut std::mem::drop::<Environment>
    ));
    assert_eq!(
        captured,
        FxHashMap::from_iter([
            (0, FxHashMap::from_iter([("owner".into(), 1)]).into()),
            (1, FxHashMap::from_iter([("cyclic".into(), 0)]).into()),
        ])
    );
}

#[test]
fn preserved_reader_still_rebuilds_when_its_owner_moves() {
    let mut captured =
        FxHashMap::from_iter([(0, FxHashMap::from_iter([("owner".into(), 2)]).into())]);
    assert!(remap(
        &mut captured,
        1,
        &FxHashMap::from_iter([(2, 1)]),
        &mut std::mem::drop::<Environment>
    ));
    assert_eq!(captured[&0]["owner"], 1);
}

#[test]
fn saved_reader_moves_keep_unchanged_origin_maps_shared() {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("capture-snapshot-sharing.json");
    let saved: Vec<(usize, Vec<(String, usize)>)> =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let original: CapturedBindings = saved
        .into_iter()
        .map(|(reader, origins)| (reader, origins.into_iter().collect()))
        .collect();
    let mut arm = original.clone();
    assert!(remap(
        &mut arm,
        1,
        &FxHashMap::from_iter([(2, 1), (8, 8)]),
        &mut std::mem::drop::<Environment>
    ));
    // Reader IDs move independently of their immutable capture-origin maps.
    assert!(std::ptr::eq(&*arm[&1], &*original[&2]));
    assert!(std::ptr::eq(&*arm[&8], &*original[&8]));
    assert!(!std::ptr::eq(&*arm[&0], &*original[&0]));
    assert_eq!(arm[&0]["moving"], 1);
    assert_eq!(original[&0]["moving"], 2);
    assert_eq!(arm[&0]["outside"], 99);
}
