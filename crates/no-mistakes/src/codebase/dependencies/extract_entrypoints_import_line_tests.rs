use super::{import_line_at, import_line_starts};

#[test]
fn empty_source_uses_line_one() {
    assert!(import_line_starts("").is_empty());
    assert_eq!(import_line_at(&[], 0), 1);
    assert_eq!(import_line_at(&[], 12), 1);
}

#[test]
fn binary_search_maps_offsets_onto_one_based_lines() {
    let starts = import_line_starts("a\nbc\n");
    assert_eq!(starts, vec![0, 2, 5]);
    assert_eq!(import_line_at(&starts, 0), 1);
    assert_eq!(import_line_at(&starts, 2), 2);
    assert_eq!(import_line_at(&starts, 4), 2);
    assert_eq!(import_line_at(&starts, 5), 3);
}
