use super::*;

#[test]
fn semver_range_is_allowed() {
    assert!(!is_blocked_specifier("^4.17.21"));
}

#[test]
fn workspace_is_allowed() {
    assert!(!is_blocked_specifier("workspace:*"));
}

#[test]
fn catalog_is_allowed() {
    assert!(!is_blocked_specifier("catalog:"));
}

#[test]
fn npm_alias_allowed() {
    assert!(!is_blocked_specifier("npm:lodash@^4"));
}

#[test]
fn file_specifier_blocked() {
    assert!(is_blocked_specifier("file:../my-lib"));
}

#[test]
fn link_specifier_blocked() {
    assert!(is_blocked_specifier("link:../other"));
}

#[test]
fn portal_specifier_blocked() {
    assert!(is_blocked_specifier("portal:../pkg"));
}

#[test]
fn patch_specifier_blocked() {
    assert!(is_blocked_specifier("patch:lodash@4.17.21"));
}

#[test]
fn git_specifier_blocked() {
    assert!(is_blocked_specifier("git+https://github.com/foo/bar.git"));
}

#[test]
fn github_shorthand_blocked() {
    assert!(is_blocked_specifier("github:owner/repo"));
}

#[test]
fn remaining_blocked_prefixes_and_npm_alias_versions() {
    for spec in [
        "git://github.com/foo/bar.git",
        "git+ssh://git@github.com/foo/bar.git",
        "bitbucket:owner/repo",
        "gitlab:owner/repo",
        "gist:abc123",
        "npm:lodash@file:../bad",
    ] {
        assert!(is_blocked_specifier(spec), "{spec}");
    }
}

#[test]
fn owner_slash_repo_shorthand_blocked() {
    assert!(is_blocked_specifier("owner/repo"));
}

#[test]
fn scoped_package_allowed() {
    assert!(!is_blocked_specifier("@scope/pkg"));
}

#[test]
fn scoped_package_with_extra_slash_blocked() {
    assert!(is_blocked_specifier("@scope/pkg/sub"));
}

#[test]
fn npm_alias_scoped_package_blocked() {
    // npm:@scope/pkg@version — exercises lines 78-79 (scoped npm: alias path).
    assert!(is_blocked_specifier("npm:@scope/pkg@github:owner/repo"));
}

#[test]
fn npm_alias_scoped_package_without_version_is_safe() {
    // npm:@scope/pkg with no version — after_at.find('@') = None → version = "" → false.
    assert!(!is_blocked_specifier("npm:@scope/pkg"));
}

#[test]
fn http_url_blocked() {
    assert!(is_blocked_specifier("http://example.com/pkg.tgz"));
}

#[test]
fn https_url_blocked() {
    assert!(is_blocked_specifier("https://example.com/pkg.tgz"));
}

// ── fixture tests ────────────────────────────────────────────────────────────
