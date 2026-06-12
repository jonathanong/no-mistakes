use super::*;

#[test]
fn glob_normalization_preserves_parent_segments_after_wildcards() {
    let wildcard_parent_glob = build_globset(&["*/../foo".to_string()]).unwrap();

    assert!(wildcard_parent_glob.is_match("pkg/../foo"));
    assert!(!wildcard_parent_glob.is_match("foo"));
}
