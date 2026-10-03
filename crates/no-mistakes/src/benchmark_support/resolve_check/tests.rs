use super::*;

#[test]
fn prepared_routes_are_real_files_for_a_large_route_batch() {
    let fixture = prepared_resolve_check_fixture();
    // The resolver uses is_file() to select the first tsconfig candidate.
    assert_eq!(fixture.files.len(), LARGE_ROUTE_BATCH_FILE_COUNT);
    assert!(fixture.files.iter().all(|file| file.is_file()));
    assert_eq!(fixture.visible.len(), LARGE_ROUTE_BATCH_VISIBLE_FILE_COUNT);
    // Keep prepared facts and persisted route sources on the same synthetic import.
    for file in &fixture.files {
        let facts = fixture
            .facts
            .get(file)
            .expect("route facts should be prepared");
        assert_eq!(facts.imports[0].specifier, "@example/shared");
        let source = fixture
            .source_store
            .read_path(file)
            .expect("route source should exist");
        assert!(source.contains("from \"@example/shared\""));
    }
}
