use super::*;
fn name(value: &str) -> Name {
    ("entry.mts".into(), 0, value.into())
}
#[test]
fn nullish_guard_receipts_validate_positive_negative_and_transitive_checks() {
    let cache = ParameterCache::default();
    let entry = [name("active")].into_iter().collect();
    let mut reads = GuardReads::default();
    reads.append(
        [name("active"), name("absent"), name("active")].into_iter(),
        &cache,
    );
    let receipt = reads.capture(0, &entry, &cache);
    assert!(matches(&receipt, &entry, &cache));
    assert!(!matches(&receipt, &fx_set(), &cache));
    let mut different = entry.clone();
    different.insert(name("absent"));
    assert!(!matches(&receipt, &different, &cache));
    // Cache hits must propagate even checks which did not cut a cycle.
    let start = reads.len();
    reads.append(receipt.iter().map(|(name, _)| name.clone()), &cache);
    assert_eq!(reads.capture(start, &entry, &cache), receipt);
    let empty = reads.capture(reads.len(), &entry, &cache);
    assert!(empty.is_empty());
    assert!(matches(&empty, &different, &cache));
}
