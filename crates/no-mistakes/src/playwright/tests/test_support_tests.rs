use crate::playwright::test_support::fixture_path;

#[test]
fn fixture_path_with_single_part_extends_directly() {
    let path = fixture_path(&["scan-config"]);
    assert!(path.ends_with("test-cases/scan-config"));
}
