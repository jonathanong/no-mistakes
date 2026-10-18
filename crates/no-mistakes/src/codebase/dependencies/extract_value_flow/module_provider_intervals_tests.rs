use super::*;

#[test]
fn module_provider_intervals_merge_possible_overlaps_and_keep_resets() {
    let mut target = ModuleProviderTarget {
        intervals: vec![(4, 7), (1, 5), (10, 11)],
        uncertain: false,
    };
    target.finish();
    assert_eq!(target.intervals, [(1, 7), (10, 11)]);
    for (offset, expected) in [
        (0, false),
        (1, true),
        (6, true),
        (7, false),
        (10, true),
        (11, false),
    ] {
        assert_eq!(target.contains(offset), expected);
    }
    target.uncertain = true;
    assert!(target.contains(0) && target.contains(u32::MAX));
    assert!(!ModuleProviderTarget::default().contains(0));
}
