use super::*;

#[test]
fn prepared_routes_are_real_files_at_the_vouchington_scale() {
    let fixture = prepared_resolve_check_fixture();
    // The resolver uses is_file() to select the first tsconfig candidate.
    assert_eq!(fixture.files.len(), VOUCHINGTON_RESOLVE_CHECK_FILE_COUNT);
    assert!(fixture.files.iter().all(|file| file.is_file()));
    assert_eq!(fixture.visible.len(), VOUCHINGTON_VISIBLE_FILE_COUNT);
}
