use super::*;

impl RootSnapshots {
    fn reversed(&self, mut visit: impl FnMut(u32, usize)) {
        let _ = self.try_reversed(|offset, selected| {
            visit(offset, selected);
            Some(())
        });
    }
}

#[test]
fn reverse_snapshot_visits_stop_when_projection_cancels() {
    for snapshots in [
        RootSnapshots::Ordered((0..1024).map(|i| (i, i as usize)).collect()),
        RootSnapshots::Tree((0..1024).map(|i| (i, i as usize)).collect()),
    ] {
        let mut visits = 0;
        assert_eq!(
            snapshots.try_reversed(|_, _| {
                visits += 1;
                (visits < 2).then_some(())
            }),
            None
        );
        assert_eq!(visits, 2);
    }
}

fn assert_queries(snapshots: &RootSnapshots, expected: &BTreeMap<u32, usize>) {
    #[cfg(feature = "test-instrumentation")]
    assert_eq!(snapshots.last_key_value(), expected.last_key_value());
    let mut reversed = Vec::new();
    snapshots.reversed(|offset, selected| reversed.push((offset, selected)));
    assert_eq!(
        reversed,
        expected
            .iter()
            .rev()
            .map(|(at, id)| (*at, *id))
            .collect::<Vec<_>>()
    );
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

#[test]
fn latest_qualification_preserves_strict_and_inclusive_boundaries() {
    let mut snapshots = RootSnapshots::default();
    let mut expected = BTreeMap::new();
    for offset in [0, 1, 17, u32::MAX] {
        snapshots.insert(offset, offset as usize);
        expected.insert(offset, offset as usize);
        for query in [0, 1, 2, 16, 17, 18, u32::MAX - 1, u32::MAX] {
            assert_eq!(snapshots.at(query), expected.range(..=query).next_back());
            assert_eq!(snapshots.before(query), expected.range(..query).next_back());
        }
    }
}
