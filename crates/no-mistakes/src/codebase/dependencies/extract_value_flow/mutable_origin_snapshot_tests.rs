use super::*;

fn assert_queries(snapshots: &RootSnapshots, expected: &BTreeMap<u32, usize>) {
    #[cfg(feature = "test-instrumentation")]
    assert_eq!(snapshots.last_key_value(), expected.last_key_value());
    for offset in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, u32::MAX] {
        assert_eq!(
            snapshots.before(offset),
            expected.range(..offset).next_back()
        );
        assert_eq!(snapshots.at(offset), expected.range(..=offset).next_back());
    }
}
#[test]
fn increasing_and_equal_offsets_retain_exact_ordered_queries() {
    let mut snapshots = RootSnapshots::default();
    let mut expected = BTreeMap::new();
    assert_queries(&snapshots, &expected);
    for (offset, selected) in [(0, 0), (2, 1), (2, 7), (4, 2), (9, 3), (u32::MAX, 4)] {
        snapshots.insert(offset, selected);
        expected.insert(offset, selected);
        assert_queries(&snapshots, &expected);
        assert!(matches!(snapshots, RootSnapshots::Ordered(_)));
    }
}
#[test]
fn earlier_completion_promotes_once_without_losing_or_duplicating_entries() {
    let mut snapshots = RootSnapshots::default();
    let mut expected = BTreeMap::new();
    for (offset, selected) in [(2, 1), (4, 2), (8, 3), (3, 4), (3, 5), (0, 6), (9, 7)] {
        snapshots.insert(offset, selected);
        expected.insert(offset, selected);
        assert_queries(&snapshots, &expected);
    }
    #[cfg(feature = "test-instrumentation")]
    assert_eq!(snapshots.tree_buffer(), 1);
    let RootSnapshots::Tree(actual) = snapshots else {
        panic!("earlier insert must promote")
    };
    assert_eq!(actual, expected);
    assert_eq!(actual.len(), 6);
}
