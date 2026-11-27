use super::*;

#[test]
fn identity_events_merge_intervals_without_inventing_cross_writer_order() {
    let read = |offset| Some((offset, None));
    let foreign = |offset| Some((offset, Some(CallableId(4))));
    let mut events = IdentityEvents::default();
    assert!(!events.changed(read(1), read(9)));
    assert!(events.changed(None, read(9)));
    events.record(5, ValueAssignedWriter::Module);
    events.record(5, ValueAssignedWriter::Module);
    assert!(events.changed(read(1), read(5)));
    assert!(events.changed(read(5), read(1)));
    assert!(!events.changed(read(5), read(9)));
    assert!(!events.changed(read(5), read(5)));
    assert!(events.changed(read(1), foreign(9)));
    let mut same = IdentityEvents::default();
    same.record(8, ValueAssignedWriter::Module);
    events.merge(same);
    assert!(events.changed(read(5), read(9)));
    let mut other = IdentityEvents::default();
    other.record(20, ValueAssignedWriter::Callable(CallableId(4)));
    events.merge(other);
    assert!(events.changed(read(30), read(40)));
    assert!(events.changed(foreign(30), foreign(40)));
    assert_eq!(events.offsets.len(), 3);
    assert_eq!(events.writer, Some(ValueAssignedWriter::Mixed));
    events.merge(IdentityEvents::default());
    let mut mixed = IdentityEvents::default();
    mixed.record(1, ValueAssignedWriter::Module);
    mixed.record(2, ValueAssignedWriter::Callable(CallableId(4)));
    assert!(mixed.changed(read(10), read(11)));
}

#[test]
fn capture_merges_preserve_equal_lineage_and_poison_ambiguous_lineage() {
    let mut interner = capture_reads::CaptureInterner::default();
    let capture = FreshCapture {
        origin: Some((2, None)),
        reads: interner.capture(&[Some((1, None))]),
        epoch: 0,
        phase: None,
        completed_at: 3,
    };
    let mut first = IdentityMetadata::default();
    first.capture(3, capture.clone());
    first.capture(3, capture.clone());
    first.preserve(Some("client"), 5);
    first.preserve(None, 7);
    first.unknown.record(11, ValueAssignedWriter::Module);
    let mut joined = IdentityMetadata::default();
    joined.merge(first.clone());
    joined.merge(first);
    assert_eq!(joined.captures[&3], Some(capture.clone()));
    assert!(joined.changed("client", 4, 5));
    assert!(!joined.changed("sibling", 4, 5));
    assert!(joined.changed("sibling", 6, 7));
    assert!(!joined.changed("client", 7, 8));
    assert!(joined.unknown.changed(Some((10, None)), Some((11, None))));
    let mut different = capture.clone();
    different.epoch = 1;
    let mut conflicting = IdentityMetadata::default();
    conflicting.capture(3, different.clone());
    joined.merge(conflicting);
    assert_eq!(joined.captures[&3], None);
    // Once ambiguous, another visit cannot restore certainty at the same offset.
    joined.capture(3, capture);
    assert_eq!(joined.captures[&3], None);
    let mut direct_conflict = IdentityMetadata::default();
    direct_conflict.capture(3, different.clone());
    different.origin = None;
    direct_conflict.capture(3, different);
    assert_eq!(direct_conflict.captures[&3], None);
}
