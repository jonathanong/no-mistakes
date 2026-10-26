use super::is_nested_key;

#[test]
fn dash_comment_and_empty_are_not_nested_keys() {
    assert!(!is_nested_key(""));
    assert!(!is_nested_key("  # comment"));
    assert!(!is_nested_key("- uses: actions/checkout@v4"));
}
