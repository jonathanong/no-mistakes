use super::*;

#[test]
fn expand_unspecified_is_empty() {
    // Defensive arm: callers never pass Unspecified to expand().
    assert!(expand(&PermissionSpec::Unspecified).is_empty());
}
