use super::*;

#[test]
fn unchanged_ownership_skips_rebuilding_even_when_unrelated_frames_move() {
    let mut captured = FxHashMap::from_iter([
        (0, FxHashMap::from_iter([("preserved".into(), 0)])),
        (1, FxHashMap::from_iter([("inherited".into(), 0)])),
    ]);
    let snapshot = captured.clone();
    assert!(!remap(
        &mut captured,
        1,
        &FxHashMap::from_iter([(1, 1), (3, 2)])
    ));
    assert_eq!(captured, snapshot);
}

#[test]
fn ownership_changes_include_removed_readers_and_moved_origins() {
    let mut captured = FxHashMap::from_iter([
        (0, FxHashMap::from_iter([("owner".into(), 2)])),
        (2, FxHashMap::from_iter([("cyclic".into(), 0)])),
        (3, FxHashMap::from_iter([("dead".into(), 3)])),
    ]);
    assert!(remap(&mut captured, 1, &FxHashMap::from_iter([(2, 1)])));
    assert_eq!(
        captured,
        FxHashMap::from_iter([
            (0, FxHashMap::from_iter([("owner".into(), 1)])),
            (1, FxHashMap::from_iter([("cyclic".into(), 0)])),
        ])
    );
}

#[test]
fn preserved_reader_still_rebuilds_when_its_owner_moves() {
    let mut captured = FxHashMap::from_iter([(0, FxHashMap::from_iter([("owner".into(), 2)]))]);
    assert!(remap(&mut captured, 1, &FxHashMap::from_iter([(2, 1)])));
    assert_eq!(captured[&0]["owner"], 1);
}
