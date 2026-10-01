#[test]
fn a_blank_snippet_is_not_located() {
    assert!(super::find_snippet("SELECT 1", " \t").is_none());
}
