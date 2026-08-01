use super::*;

#[test]
fn is_relative_recognizes_dot_prefixed_specifiers() {
    assert!(is_relative("./foo"));
    assert!(is_relative("../foo/bar"));
    assert!(!is_relative("foo"));
    assert!(!is_relative("@acme/foo"));
    assert!(!is_relative("#internal"));
}

#[test]
fn is_node_builtin_matches_bare_and_prefixed_forms() {
    assert!(is_node_builtin("fs"));
    assert!(is_node_builtin("node:fs"));
    assert!(is_node_builtin("node:path"));
    assert!(!is_node_builtin("chalk"));
    assert!(!is_node_builtin("node:not-a-builtin"));
}

#[test]
fn is_node_builtin_matches_subpath_imports() {
    assert!(is_node_builtin("fs/promises"));
    assert!(is_node_builtin("node:fs/promises"));
    assert!(!is_node_builtin("chalk/subpath"));
}

#[test]
fn package_name_returns_none_for_relative_absolute_and_hash_specifiers() {
    assert_eq!(package_name("./foo"), None);
    assert_eq!(package_name("../foo"), None);
    assert_eq!(package_name("/abs/path"), None);
    assert_eq!(package_name("#internal"), None);
}

#[test]
fn package_name_returns_none_for_empty_specifier() {
    assert_eq!(package_name(""), None);
}

#[test]
fn package_name_strips_subpath_for_unscoped_packages() {
    assert_eq!(package_name("left-pad"), Some("left-pad".to_string()));
    assert_eq!(
        package_name("left-pad/lib/helper"),
        Some("left-pad".to_string())
    );
}

#[test]
fn package_name_keeps_scope_and_strips_subpath_for_scoped_packages() {
    assert_eq!(package_name("@acme/lib"), Some("@acme/lib".to_string()));
    assert_eq!(
        package_name("@acme/lib/sub/path"),
        Some("@acme/lib".to_string())
    );
}
