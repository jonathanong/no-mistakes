use super::SqlByteOrigins;

#[test]
fn plain_literal_stays_one_inline_run() {
    let values: Vec<u32> = (0..8_000).map(|index| 40 + index).collect();
    let origins = SqlByteOrigins::from(values.clone());
    assert_eq!(origins.len(), values.len());
    assert_eq!(origins.stored_runs(), 1);
    assert_eq!(origins.stored_heap_bytes(), 0);
    assert_eq!(origins.first(), Some(40));
    assert_eq!(origins.at(0), 40);
    assert_eq!(origins.at(7_999), 40 + 7_999);
    assert_eq!(origins.get(8_000), None);
    assert!(values
        .iter()
        .enumerate()
        .all(|(index, origin)| origins.get(index) == Some(*origin)));
}

#[test]
fn repeated_and_jumped_origins_stay_few_runs() {
    let mut values = vec![7; 4];
    values.extend(20..23);
    values.extend(std::iter::repeat_n(90, 3));
    values.push(4);
    let origins = SqlByteOrigins::from(values.clone());
    assert_eq!(origins.stored_runs(), 4);
    assert!(origins.stored_heap_bytes() < 128);
    assert!(values
        .iter()
        .enumerate()
        .all(|(index, origin)| origins.at(index) == *origin));
    assert_eq!(origins.get(values.len()), None);
    let mut cleared = origins.clone();
    cleared.clear();
    assert!(cleared.is_empty());
    assert_eq!(cleared.stored_runs(), 0);
    assert_ne!(origins, cleared);
    assert_eq!(origins, SqlByteOrigins::from(values));
}

#[test]
fn empty_constant_and_singleton_runs_round_trip() {
    assert!(SqlByteOrigins::default().is_empty());
    assert_eq!(SqlByteOrigins::default().first(), None);
    assert_eq!(SqlByteOrigins::empty().len(), 0);
    assert_eq!(SqlByteOrigins::from(vec![0, 0, 0]).at(2), 0);
    let single = SqlByteOrigins::from(vec![15]);
    assert_eq!(single.stored_runs(), 1);
    assert_eq!(single.stored_heap_bytes(), 0);
    assert_eq!(single.first(), Some(15));
    assert_eq!(SqlByteOrigins::from(vec![3, 5, 7]).at(2), 7);
}
