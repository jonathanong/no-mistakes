#[test]
fn legacy_file_analysis_does_not_read_the_filesystem_directly() {
    let source = include_str!("legacy.rs");
    assert!(!source.contains("std::fs::read_to_string"));
    assert!(source.contains("read_prepared_or_open"));
    assert!(source.contains("read_source"));
}
