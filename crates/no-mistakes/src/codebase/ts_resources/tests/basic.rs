use super::super::paths::decode_module_url_path;
use super::*;

#[test]
fn extracts_bound_fs_glob_and_url_calls() {
    let facts = facts(include_str!(
        "../../../../../../fixtures/test-plan/resource-impact/extractor-bindings.ts"
    ));
    assert_eq!(facts.calls.len(), 7);
    assert_eq!(facts.calls[0].kind, ResourceCallKind::ReadFile);
    assert_eq!(facts.calls[1].kind, ResourceCallKind::ReadFileSync);
    assert_eq!(facts.calls[2].kind, ResourceCallKind::ReadDirectory);
    assert_eq!(facts.calls[3].kind, ResourceCallKind::Glob);
    assert_eq!(facts.calls[3].cwd.as_ref().unwrap().value, "src");
    assert_eq!(facts.calls[6].path.base, ResourcePathBase::SourceModule);
}

#[test]
fn supports_require_aliases_and_rejects_dynamic_calls() {
    let facts = facts(include_str!(
        "../../../../../../fixtures/test-plan/resource-impact/extractor-dynamic.ts"
    ));
    assert_eq!(facts.calls.len(), 1);
    assert_eq!(facts.calls[0].kind, ResourceCallKind::ReadDirectory);
    assert_eq!(facts.diagnostics.len(), 2);
    assert_eq!(
        facts.diagnostics[0].kind,
        ResourceDiagnosticKind::DynamicPath
    );
    assert_eq!(
        facts.diagnostics[1].kind,
        ResourceDiagnosticKind::DynamicCwd
    );
}

#[test]
fn ignores_shadowed_and_reassigned_bindings() {
    let facts = facts(include_str!(
        "../../../../../../fixtures/test-plan/resource-impact/extractor-shadowed.ts"
    ));
    assert!(facts.calls.is_empty());
}

#[test]
fn module_url_paths_decode_but_reject_encoded_separators() {
    assert_eq!(
        decode_module_url_path("plain.json"),
        Some("plain.json".into())
    );
    assert_eq!(
        decode_module_url_path("./data%20file.json"),
        Some("./data file.json".into())
    );
    assert_eq!(
        decode_module_url_path("./lower%ab.json?ignored#fragment"),
        None
    );
    assert_eq!(decode_module_url_path("./upper%AB.json"), None);
    assert_eq!(decode_module_url_path("./nested%2Ffile.json"), None);
    assert_eq!(decode_module_url_path("./bad%2.json"), None);
    assert_eq!(decode_module_url_path("//example.test/data.json"), None);
    assert_eq!(decode_module_url_path("file:data.json"), None);
    assert_eq!(decode_module_url_path("C:data.json"), None);
    assert_eq!(
        decode_module_url_path("https://example.test/data.json"),
        None
    );
}

#[test]
fn unnamed_default_function_uses_the_default_scope() {
    let facts = facts(include_str!(
        "../../../../../../fixtures/test-plan/resource-impact/extractor-unnamed-default.ts"
    ));
    assert_eq!(facts.calls.len(), 1);
    assert_eq!(facts.calls[0].path.value, "unnamed-default.json");
    assert_eq!(facts.calls[0].function_scope.as_deref(), Some("default"));
}

#[test]
fn parenthesized_default_functions_keep_the_default_scope() {
    for (source, path) in [
        (
            include_str!(
                "../../../../../../fixtures/test-plan/resource-impact/extractor-parenthesized-default-arrow.ts"
            ),
            "parenthesized-default-arrow.json",
        ),
        (
            include_str!(
                "../../../../../../fixtures/test-plan/resource-impact/extractor-parenthesized-default-function.ts"
            ),
            "parenthesized-default-function.json",
        ),
    ] {
        let facts = facts(source);
        assert_eq!(facts.calls.len(), 1, "{path}");
        assert_eq!(facts.calls[0].path.value, path);
        assert_eq!(facts.calls[0].function_scope.as_deref(), Some("default"));
    }
}
