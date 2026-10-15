use super::*;
fn name(value: &str) -> Name {
    ("entry.mts".into(), 0, value.into())
}
fn captured(
    reads: &GuardReads,
    start: usize,
    entry: &FxHashSet<Name>,
    cache: &ParameterCache,
) -> BTreeMap<Name, bool> {
    reads.capture(start, entry, cache).into_iter()
        .map(|(id, active)| (reads.arena[id].as_ref().clone(), active)).collect()
}
fn oracle(log: &[Name], start: usize, entry: &FxHashSet<Name>) -> BTreeMap<Name, bool> {
    log[start..].iter().map(|name| (name.clone(), entry.contains(name))).collect()
}
#[test]
fn nullish_guard_receipts_validate_positive_negative_and_transitive_checks() {
    let cache = ParameterCache::default();
    // This active name is not yet interned when the entry snapshot is taken.
    let entry = [name("active")].into_iter().collect();
    let mut reads = GuardReads::default();
    reads.append([name("active"), name("absent"), name("active")].iter(), &cache);
    let receipt = reads.capture(0, &entry, &cache);
    assert!(reads.matches(&receipt, &entry, &cache));
    assert!(!reads.matches(&receipt, &fx_set(), &cache));
    let mut different = entry.clone();
    different.insert(name("absent"));
    assert!(!reads.matches(&receipt, &different, &cache));
    // Replay names, never the inner computation's entry-membership booleans.
    let start = reads.len();
    reads.append_ids(receipt.iter().map(|(id, _)| *id), &cache);
    assert_eq!(reads.capture(start, &entry, &cache), receipt);
    let outer = reads.capture(start, &fx_set(), &cache);
    assert!(outer.iter().all(|(_, active)| !active));
    assert!(reads.matches(&outer, &fx_set(), &cache));
    let empty = reads.capture(reads.len(), &entry, &cache);
    assert!(empty.is_empty());
    assert!(reads.matches(&empty, &different, &cache));
}
#[test]
fn latest_read_index_matches_every_legacy_suffix_across_repeats_and_nested_replays() {
    let cache = ParameterCache::default();
    let mut reads = GuardReads::default();
    let mut log = Vec::new();
    let entries = [fx_set(), [name("a")].into_iter().collect(),
        [name("a"), name("b")].into_iter().collect()];
    for value in ["a", "b", "a", "c", "b", "b", "a", "c"] {
        let value = name(value);
        reads.append(std::iter::once(&value), &cache);
        log.push(value);
        for start in 0..=log.len() {
            for entry in &entries {
                assert_eq!(captured(&reads, start, entry, &cache), oracle(&log, start, entry));
            }
        }
    }
    let inner = reads.capture(3, &entries[2], &cache);
    let replay = inner.iter().map(|(id, _)| reads.arena[*id].as_ref().clone()).collect::<Vec<_>>();
    reads.append_ids(inner.iter().map(|(id, _)| *id), &cache);
    log.extend(replay);
    for start in 0..=log.len() {
        for entry in &entries {
            assert_eq!(captured(&reads, start, entry, &cache), oracle(&log, start, entry));
        }
    }
    assert_eq!(reads.ordered.len(), 3);
    assert_eq!(reads.arena.len(), 3);
}
#[test]
fn latest_capture_visits_only_unique_names_in_the_requested_suffix() {
    let cache = ParameterCache::default();
    let mut reads = GuardReads::default();
    for index in 0..128 {
        let start = reads.len();
        let value = name(&format!("independent{index}"));
        reads.append(std::iter::once(&value), &cache);
        assert_eq!(reads.capture(start, &fx_set(), &cache).len(), 1);
    }
    let start = reads.len();
    let value = name("repeated");
    for _ in 0..128 { reads.append(std::iter::once(&value), &cache); }
    assert_eq!(reads.capture(start, &fx_set(), &cache).len(), 1);
    assert_eq!(reads.ordered.len(), 129);
    #[cfg(feature = "test-instrumentation")]
    {
        let work = cache.nullish_work();
        assert_eq!(work[7], 129); // actual capture candidates, not all interned names
        assert_eq!(work[11], 383); // 256 inserts plus 127 replacements
        assert_eq!(work[12], 129);
        assert_eq!(work[13], 256);
        assert_eq!(work[14], 129);
    }
}

#[test]
#[should_panic(expected = "guard read clock exhausted")]
fn nullish_guard_clock_never_wraps_an_active_capture_boundary() {
    let cache = ParameterCache::default();
    let mut reads = GuardReads {
        clock: usize::MAX,
        ..Default::default()
    };
    reads.append(std::iter::once(&name("last")), &cache);
}
