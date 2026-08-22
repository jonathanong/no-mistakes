use super::*;

#[test]
fn records_static_url_forms_and_scoped_dynamic_diagnostics() {
    let facts = facts(include_str!(
        "../../../../../../fixtures/test-plan/resource-impact/extractor-coverage.ts"
    ));
    let kinds = facts.calls.iter().map(|call| call.kind).collect::<Vec<_>>();
    for kind in [
        ResourceCallKind::ReadFile,
        ResourceCallKind::ReadFileSync,
        ResourceCallKind::ReadDirectory,
        ResourceCallKind::ReadDirectorySync,
        ResourceCallKind::Glob,
        ResourceCallKind::GlobSync,
    ] {
        assert!(kinds.contains(&kind));
    }
    let diagnostic_kinds = facts
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.kind)
        .collect::<Vec<_>>();
    for kind in [
        ResourceDiagnosticKind::DynamicPath,
        ResourceDiagnosticKind::DynamicPattern,
        ResourceDiagnosticKind::DynamicCwd,
    ] {
        assert!(diagnostic_kinds.contains(&kind));
    }
    for path in [
        "./url-resource.json",
        "./file-url-resource.json",
        "./namespace-url-resource.json",
        "./inline-url-resource.json",
    ] {
        let call = facts
            .calls
            .iter()
            .find(|call| call.path.value == path)
            .unwrap();
        assert_eq!(call.path.base, ResourcePathBase::SourceModule, "{path}");
        assert_eq!(call.function_scope.as_deref(), Some("resourceScope"));
    }
    assert!(facts
        .calls
        .iter()
        .any(|call| call.path.value == "after-var-binding.json"
            && call.path.base == ResourcePathBase::AnalysisRoot
            && call.function_scope.as_deref() == Some("resourceScope")));
    for path in [
        "direct-import.json",
        "direct-sync-import.json",
        "direct-directory-import.json",
        "promises-import.json",
        "promises-namespace-directory.json",
        "inline-require.json",
        "inline-promises-require.json",
        "nested-fs-promises.json",
        "anonymous-arrow.json",
        "switch-resource.json",
        "named-default.json",
        "namespace/**/*.txt",
        "fast-glob/**/*.txt",
        "inline-default-glob/**/*.txt",
        "inline-glob-sync/**/*.txt",
    ] {
        assert!(
            facts.calls.iter().any(|call| call.path.value == path),
            "{path}"
        );
    }
    assert!(facts.calls.iter().any(|call| {
        call.path.value == "templates/**/*.txt"
            && call
                .cwd
                .as_ref()
                .is_some_and(|cwd| cwd.value == "static-cwd")
    }));
    assert!(facts.calls.iter().any(|call| {
        call.path.value == "templates/**/*.txt"
            && call
                .cwd
                .as_ref()
                .is_some_and(|cwd| cwd.base == ResourcePathBase::SourceModule)
    }));
    assert!(facts
        .calls
        .iter()
        .all(|call| call.path.value != "must-not-be-recorded.json"));
    assert_eq!(
        facts
            .diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.kind, diagnostic.function_scope.as_deref()))
            .collect::<Vec<_>>(),
        vec![
            (ResourceDiagnosticKind::DynamicPath, Some("resourceScope")),
            (
                ResourceDiagnosticKind::DynamicPattern,
                Some("resourceScope")
            ),
            (ResourceDiagnosticKind::DynamicCwd, Some("resourceScope")),
        ]
    );
}

#[test]
fn nested_assignment_targets_are_walked() {
    let facts = facts(include_str!(
        "../../../../../../fixtures/test-plan/resource-impact/extractor-nested-assign.ts"
    ));
    assert!(
        facts.calls.is_empty(),
        "nested assignment targets must invalidate rebound fs bindings: {facts:#?}"
    );
}

#[test]
fn default_export_and_argumentless_resource_calls_are_walked() {
    let prefix = "import * as fs from 'node:fs';\nimport { glob } from 'glob';\n";
    for source in [
        "export default function () { fs.readFile('anon-default.json'); }",
        "export default () => { fs.readFile('arrow-default.json'); }",
        "export default class C { read() { fs.readFile('class-default.json'); } }",
        "export default (function () { fs.readFile('paren-fn.json'); });",
        "export default (() => { fs.readFile('paren-arrow.json'); });",
        "export default (1);\n(fs.readFile)('paren-callee.json');",
        "fs.readFile();\nglob();",
    ] {
        let facts = facts(&format!("{prefix}{source}"));
        assert!(
            !facts.calls.is_empty() || !facts.diagnostics.is_empty(),
            "{source}: {facts:#?}"
        );
    }
}
