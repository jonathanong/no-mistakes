use super::*;

// Byte-for-byte original lookup body remains an independent pre-cursor oracle.
fn original_at(segments: &RegionSegments, owner: Option<CallableId>, offset: u32) -> Option<usize> {
    let segments = segments.owners.get(&owner)?;
    let index = segments
        .partition_point(|(start, _, _)| *start <= offset)
        .checked_sub(1)?;
    let (_, end, region) = segments[index];
    (offset < end).then_some(region)
}

#[test]
fn exact_segments_owners_boundaries_gaps_and_reverse_queries() {
    // These are the disjoint segments after nesting: region 0 resumes after 1.
    let mut owners = fx_map();
    owners.insert(
        None,
        vec![(10, 20, 0), (20, 30, 1), (30, 40, 0), (50, 60, 2)],
    );
    owners.insert(Some(CallableId(1)), vec![(10, 40, 3)]);
    let segments = RegionSegments { owners };
    let mut cursor = RegionCursor::default();
    for owner in [None, Some(CallableId(1)), None, Some(CallableId(2))] {
        for offset in [
            0, 10, 19, 20, 29, 30, 39, 40, 45, 50, 59, 60, 59, 50, 39, 30, 29, 20, 19, 10, 0,
        ] {
            assert_eq!(
                cursor.at(&segments, owner, offset),
                original_at(&segments, owner, offset)
            );
        }
    }
    // Returning from a missing owner must not reuse its predecessor's segment.
    assert_eq!(cursor.at(&segments, None, 15), Some(0));
    assert_eq!(cursor.at(&segments, Some(CallableId(2)), 15), None);
    assert!(cursor.last.is_none());
    assert_eq!(cursor.at(&segments, None, 15), Some(0));
}

#[test]
fn independent_cursors_do_not_share_history() {
    let mut owners = fx_map();
    owners.insert(None, vec![(1, 10, 7)]);
    let segments = RegionSegments { owners };
    let mut first = RegionCursor::default();
    let mut second = RegionCursor::default();
    assert_eq!(first.at(&segments, None, 2), Some(7));
    assert!(second.last.is_none());
    assert_eq!(second.at(&segments, None, 9), Some(7));
    assert_eq!(first.at(&segments, None, 0), None);
    assert_eq!(second.at(&segments, None, 3), Some(7));
}
