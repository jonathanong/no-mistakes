use super::*;

#[test]
fn aligns_exported_and_member_scopes_and_keeps_lexical_shadows_local() {
    let facts = facts(include_str!(
        "../../../../../../fixtures/test-plan/resource-impact/extractor-semantics.ts"
    ));
    let scope_for = |path: &str| {
        facts
            .calls
            .iter()
            .find(|call| call.path.value == path)
            .and_then(|call| call.function_scope.as_deref())
    };
    assert_eq!(scope_for("default-arrow.json"), Some("default"));
    assert_eq!(scope_for("object-method.json"), Some("api/load"));
    assert_eq!(scope_for("class-method.json"), Some("Service/load"));
    assert_eq!(scope_for("after-block.json"), Some("lexicalScopes"));
    for call in &facts.calls {
        assert!(!matches!(
            call.path.value.as_str(),
            "hidden-by-block.json" | "hidden-by-catch.json" | "hidden-by-var.json"
        ));
    }
    assert!(facts.calls.iter().any(|call| call.path.value
        == "outer-after-inner-reassignment.json"
        && call.kind == ResourceCallKind::ReadFile));
}

#[test]
fn supports_commonjs_member_aliases_and_conservative_glob_cwd_options() {
    let facts = facts(include_str!(
        "../../../../../../fixtures/test-plan/resource-impact/extractor-semantics.ts"
    ));
    for path in [
        "member-alias.json",
        "promises-alias.json",
        "promises-destructure.json",
    ] {
        assert!(facts
            .calls
            .iter()
            .any(|call| call.path.value == path && call.kind == ResourceCallKind::ReadFile));
    }
    let glob = facts
        .calls
        .iter()
        .find(|call| call.path.value == "templates/**/*.txt")
        .unwrap();
    assert_eq!(
        glob.cwd.as_ref().map(|cwd| cwd.value.as_str()),
        Some("last")
    );
    assert_eq!(
        facts
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.kind == ResourceDiagnosticKind::DynamicCwd)
            .count(),
        2
    );
}

#[test]
fn respects_hoisted_and_loop_lexical_shadows_without_leaking_them() {
    let facts = facts(include_str!(
        "../../../../../../fixtures/test-plan/resource-impact/extractor-final-semantics.ts"
    ));
    let paths = facts
        .calls
        .iter()
        .map(|call| call.path.value.as_str())
        .collect::<Vec<_>>();
    for expected in [
        "exported-object.json",
        "exported-class.json",
        "after-loops.json",
    ] {
        assert!(paths.contains(&expected));
    }
    for hidden in [
        "hidden-by-program-require.json",
        "hidden-by-program-url.json",
        "hidden-by-nested-var.json",
        "hidden-by-for.json",
        "hidden-by-for-of.json",
        "hidden-by-for-in.json",
    ] {
        assert!(!paths.contains(&hidden), "{hidden} must be shadowed");
    }
    assert_eq!(
        facts
            .calls
            .iter()
            .find(|call| call.path.value == "exported-object.json")
            .and_then(|call| call.function_scope.as_deref()),
        Some("api/load")
    );
    assert_eq!(
        facts
            .calls
            .iter()
            .find(|call| call.path.value == "exported-class.json")
            .and_then(|call| call.function_scope.as_deref()),
        Some("Service/load")
    );
}
