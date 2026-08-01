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
fn is_node_builtin_matches_a_bare_module_name() {
    assert!(is_node_builtin("fs"));
    assert!(is_node_builtin("path"));
    assert!(!is_node_builtin("chalk"));
}

#[test]
fn is_node_builtin_matches_subpath_imports() {
    assert!(is_node_builtin("fs/promises"));
    assert!(!is_node_builtin("chalk/subpath"));
}

#[test]
fn is_scheme_prefixed_recognizes_a_colon_before_the_first_path_segment() {
    assert!(is_scheme_prefixed("virtual:app-config"));
    assert!(is_scheme_prefixed("data:text/plain;base64,aGVsbG8="));
    assert!(is_scheme_prefixed("node:fs"));
    assert!(!is_scheme_prefixed("left-pad"));
    assert!(!is_scheme_prefixed("left-pad/lib/helper"));
    assert!(!is_scheme_prefixed("@acme/lib"));
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
