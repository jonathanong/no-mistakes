use super::*;

#[test]
fn sparse_retention_releases_oversized_reader_storage() {
    let captured = (0..1024)
        .map(|frame| (frame, FxHashMap::from_iter([("shared".into(), 0)])))
        .collect::<Captures>();
    let indices = FxHashMap::from_iter([(0, 0)]);
    let (expected, _) = rebuild(&forward(captured.clone(), 0, &indices));
    let actual = compare(captured, 0, indices);
    let new_capacity = actual[&0]["shared"].capacity();
    let old_capacity = expected[&0]["shared"].capacity();
    eprintln!(
        "READER_SPARSE retained=1 remapped_capacity={new_capacity} legacy_capacity={old_capacity}"
    );
    assert_eq!(actual, expected);
    assert_eq!(new_capacity, old_capacity);
}

#[test]
fn sparse_name_and_origin_tables_shrink_without_replacing_name_buffers() {
    for distinct_origins in [false, true] {
        let captured = (0..1024)
            .map(|frame| {
                (
                    frame,
                    FxHashMap::from_iter([(
                        format!("name{frame}"),
                        if distinct_origins { frame } else { 0 },
                    )]),
                )
            })
            .collect::<Captures>();
        let actual = compare(captured, 0, FxHashMap::from_iter([(0, 0)]));
        assert_eq!(actual.capacity(), 3);
        assert_eq!(actual[&0].capacity(), 3);
        let mut discarded = actual;
        discarded.reserve(1024);
        remap::<true>(&mut discarded, 0, &FxHashMap::default());
        assert!(discarded.is_empty());
        assert_eq!(discarded.capacity(), 0);
    }
}

#[test]
fn ordinary_compactions_keep_table_capacity_and_sparse_boundary_is_explicit() {
    assert!(!substantially_sparse(0, 16));
    assert!(!substantially_sparse(8, 32));
    assert!(substantially_sparse(8, 33));
    assert!(!substantially_sparse(usize::MAX, usize::MAX));
    let captured = (0..32)
        .map(|frame| (frame, FxHashMap::from_iter([("shared".into(), 0)])))
        .collect::<Captures>();
    let (mut readers, _) = rebuild(&captured);
    let capacity = readers[&0]["shared"].capacity();
    remap::<true>(&mut readers, 32, &FxHashMap::default());
    assert_eq!(readers[&0]["shared"].capacity(), capacity);
    assert_eq!(readers, rebuild(&captured).0);
}
