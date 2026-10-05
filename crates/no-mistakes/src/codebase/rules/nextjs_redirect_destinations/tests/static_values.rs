use super::*;

const OPTIONS: &str = "configPath: web/next.config.ts\nappRoot: web/app\nincludeRewrites: false";

#[test]
fn mixed_literal_and_tuple_destinations_are_all_checked() {
    let findings = run("static-tuples-fail", OPTIONS);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("'/missing-route'"));
    assert_eq!(findings[0].line, 8);
    assert!(run("static-tuples-pass", OPTIONS).is_empty());
}

#[test]
fn typed_local_hoisted_constants_destructuring_and_templates_are_supported() {
    for fixture in ["static-variants", "static-shadowing", "static-decoys"] {
        let findings = run(fixture, OPTIONS);
        assert!(findings.is_empty(), "{fixture}: {findings:?}");
    }
}

#[test]
fn partial_dynamic_reassigned_and_aliased_mutations_fail_closed() {
    for fixture in [
        "partial-dynamic",
        "static-reassignment",
        "static-alias-mutation",
        "static-local-mutation",
        "static-conditional-mutation",
        "static-initializer-mutation",
        "static-wrapper-mutation",
        "static-callback-mutation",
        "static-helper-mutation",
        "static-object-initializer-mutation",
        "static-map-initializer-mutation",
        "static-constructor-mutation",
        "static-initialization-order",
        "static-computed-unknown",
        "static-nested-object-mutation",
        "static-wrapper-return-call",
    ] {
        let findings = run(fixture, OPTIONS);
        assert_eq!(findings.len(), 1, "{fixture}: {findings:?}");
        assert!(
            findings[0].message.contains("extraction is incomplete"),
            "{fixture}: {findings:?}"
        );
        assert!(!findings[0].message.contains("missing-route"));
    }
}

#[test]
fn rewrite_groups_are_checked_only_when_requested() {
    assert!(run("static-rewrites", OPTIONS).is_empty());
    let findings = run(
        "static-rewrites",
        "configPath: web/next.config.ts\nappRoot: web/app\nincludeRewrites: true",
    );
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("'/missing-route'"));
}

#[test]
fn unsupported_forms_and_evaluation_limits_fail_closed() {
    for fixture in [
        "static-edge-cases",
        "static-bounds",
        "static-string-bound",
        "static-object-bound",
        "static-shared-array-bound",
        "static-deep-array-bound",
        "static-alias-traversal-bound",
        "static-alias-depth-bound",
    ] {
        let findings = run(fixture, OPTIONS);
        assert_eq!(findings.len(), 1, "{fixture}: {findings:?}");
        assert!(findings[0].message.contains("extraction is incomplete"));
    }
    assert!(!run("static-dynamic-only", OPTIONS).is_empty());
    for fixture in [
        "static-no-return",
        "static-scoped-function",
        "static-object-destructure",
        "static-class-decoys",
        "static-unused-helper",
    ] {
        assert!(run(fixture, OPTIONS).is_empty(), "{fixture}");
    }
}

#[test]
fn unknown_rewrite_groups_are_incomplete() {
    let findings = run(
        "static-rewrite-extra",
        "configPath: web/next.config.ts\nappRoot: web/app\nincludeRewrites: true",
    );
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("extraction is incomplete"));
}
