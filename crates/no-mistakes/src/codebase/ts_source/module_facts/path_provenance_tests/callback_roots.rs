use super::*;

#[test]
fn callback_roots_require_local_factories_cleanup_and_direct_uses() {
    let read = read("callback-branches.mts");
    for case in [
        "function-expression",
        "root-type-reference",
        "callback-type-reference",
        "cleanup-extra-option",
    ] {
        let origin = read.origin(&format!(r#"readdirSync(root, "{case}")"#));
        assert_eq!(origin.anchor, PathAnchor::Temp, "{case}: {origin:?}");
        assert!(origin.param.is_none(), "{case}: {origin:?}");
    }
    for case in [
        "spread-supply",
        "generator-callback",
        "nonfunction-helper",
        "generator-helper",
        "default-callback",
        "destructured-callback",
        "callback-write",
        "nested-invocation",
        "root-member-call",
        "root-parameter",
        "root-destructure",
        "root-global",
        "root-block",
        "root-late",
        "root-literal",
        "root-write",
        "resource-member-other",
        "cleanup-no-finally",
        "cleanup-noncall",
        "cleanup-first-expression",
        "cleanup-namespace",
        "cleanup-global",
        "cleanup-local",
        "cleanup-written",
        "cleanup-type-import",
        "cleanup-inline-type-import",
        "cleanup-wrong-export",
        "cleanup-wrong-module",
        "cleanup-no-argument",
        "cleanup-member-argument",
        "cleanup-no-options",
        "cleanup-options-variable",
        "cleanup-spread-options",
        "cleanup-computed-options",
        "cleanup-async-unawaited",
        "cleanup-other-root",
        "resource-other-read",
    ] {
        let origin = read.origin(&format!(r#"readdirSync(root, "{case}")"#));
        assert_eq!(origin.anchor, PathAnchor::Unknown, "{case}: {origin:?}");
    }
}
